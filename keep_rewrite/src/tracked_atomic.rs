use crate::{Heaped, heaped::HeapedPtr};
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};


/// Tracks readers of every mutation of a heap allocated value with the goal of concurrent memory reclamation.
///
/// A "accessor" called `Guard<T>` will hold a reference to the value held in this `TrackedAtomic<T>`.
/// A `Guard<T>` created from this `TrackedAtomic<T>` will have the pointer to the reference it is guarding
/// registered in the domain associated with this `TrackedAtomic<T>`. The referenced value will always reference
/// the value of the internal atomic pointer at the time of creating the `Guard<T>`, even if the value of this
/// `TrackedAtomic<T>` changes during the lifetime of the guard. Once a `Guard<T>` drops it will unregister
/// the value it is referencing from its domain.
///
/// The set of all equal pointers registered to this `TrackedAtomic<T>`'s domain are representing a mutation.
/// If two (or more) `Guard<T>`s are guarding the same reference they are part of the same mutation.
///
/// Once all guards of a mutation of this `TrackedAtomic<T>` have been dropped, the value that is allocated on the heap and associated
/// with this mutation can be dropped and its memory reclaimed, as there is no longer a way to access that value.
///
/// If the domain of this `TrackedAtomic<T>` is completely empty, the `TrackedAtomic<T>` is dead and its domain can be freed.
pub(crate) struct TrackedAtomic<T>
{
    ptr: AtomicPtr<T>,
    domain: DomainNode<T>,
    dead: AtomicBool,
}


impl<T> TrackedAtomic<T>
{
    /// Creates a new and empty `TrackedAtomic<T>`.
    pub(crate) fn new(ptr: impl Heaped<T>) -> Self
    {
        Self {
            ptr: AtomicPtr::new(unsafe { ptr.heaped_ptr().0 }),
            domain: DomainNode::new(std::ptr::null_mut()),
            dead: AtomicBool::new(false),
        }
    }

    /// Adds `val` to the domain.
    pub(crate) fn add_mutation(&self, val: impl Heaped<T>) -> HeapedPtr<DomainNode<T>>
    {
        HeapedPtr(self.domain.insert(unsafe { val.heaped_ptr().0 }))
    }

    pub(crate) fn swap(&self, val: impl Heaped<T>) -> (HeapedPtr<T>, HeapedPtr<DomainNode<T>>)
    {
        let val = unsafe { val.heaped_ptr() };
        let old = HeapedPtr(self.ptr.swap(val.0, Ordering::SeqCst));
        let new_node = self.add_mutation(val);

        (old, new_node)
    }

    #[allow(clippy::type_complexity)] // It looks a bit ugly, but it has to...
    pub(crate) fn exchange(
        &self,
        current: *mut T,
        new: impl Heaped<T>,
    ) -> Result<(HeapedPtr<T>, HeapedPtr<DomainNode<T>>), Box<T>>
    {
        let new = unsafe { new.heaped_ptr() };

        self.ptr
            .compare_exchange(current, new.0, Ordering::SeqCst, Ordering::SeqCst)
            .map_err(|ptr| unsafe { Box::from_raw(ptr) })
            .map(|old| (HeapedPtr(old), self.add_mutation(new)))
    }

    /// Reads the current value
    pub(crate) fn read(&self) -> HeapedPtr<T>
    {
        HeapedPtr(self.ptr.load(Ordering::SeqCst))
    }

    /// Checks if the mutation of `val` is over.
    ///
    /// If this mutation is over, its value will be returned as `Some(Box<T>)`.
    /// Dropping this box will drop and free the value of this mutation, like the drop of a normal box would.
    ///
    /// if this mutation is not over, `None` will be returned.
    pub(crate) fn try_clean_mutation_of(&self, val: *mut T) -> Option<Box<T>>
    {
        let mut current = &self.domain;
        let mut dead = true; // Indicates if no values are registered to this domain.

        loop
        {
            let curr_val = current.value.load(Ordering::SeqCst);

            if curr_val == val
            {
                return None;
            }

            if dead && !curr_val.is_null()
            {
                // Since there is at least another value other than null or val registered to this domain it cannot be dead.
                dead = false;
            }

            match unsafe { current.next.load(Ordering::SeqCst).as_ref() }
            {
                Some(next) => current = next,

                // `val` is not registered to this domain anymore. Pack it in a box and return it.
                None =>
                {
                    // Indicate if this domain is dead to the tracked atomic
                    if dead
                    {
                        self.dead.store(true, Ordering::SeqCst);
                    }

                    return Some(unsafe { Box::from_raw(val) });
                }
            }
        }
    }

    /// Returns if self is dead.
    #[inline]
    pub(crate) fn is_dead(&self) -> bool
    {
        self.dead.load(Ordering::SeqCst)
    }

    /// Frees this domains linked list. Panics if self is not dead.
    unsafe fn destroy_domain(&self)
    {
        // Do not destroy domains that are not dead.
        assert!(self.dead.load(Ordering::SeqCst));

        let mut current = self.domain.next.load(Ordering::SeqCst);
        let mut values = vec![];

        loop
        {
            values.push(current);

            match unsafe { current.as_ref() }
            {
                Some(c) => current = c.next.load(Ordering::SeqCst),
                None => break,
            }
        }


        while let Some(val) = values.pop()
        {
            if let Some(val) = unsafe { val.as_ref() }
            {
                unsafe { val.free_child() };
            }
        }

        unsafe { self.domain.free_child() };
    }
}


impl<T> Drop for TrackedAtomic<T>
{
    fn drop(&mut self)
    {
        if self.dead.load(Ordering::SeqCst)
        {
            unsafe { self.destroy_domain() };
        }
    }
}


/// Atomically linked list node.
pub(crate) struct DomainNode<T>
{
    value: AtomicPtr<T>,
    next: AtomicPtr<Self>,
}


impl<T> DomainNode<T>
{
    /// Creates a new domain node
    pub(crate) fn new(value: *mut T) -> Self
    {
        Self {
            value: AtomicPtr::new(value),
            next: AtomicPtr::new(std::ptr::null_mut()),
        }
    }

    /// Appends `node` to the tail of this list making `node` the new tail
    pub(crate) fn append(&self, node: *mut Self)
    {
        let mut current = self;

        loop
        {
            match current.next.compare_exchange(
                std::ptr::null_mut(),
                node,
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            {
                Ok(_old) => break,
                Err(actual) => current = unsafe { &*actual },
            }
        }
    }

    /// Inserts `val` into this domain, by either finding a free node or extending this list.
    ///
    /// Returns a pointer to the node guarding `val`
    pub(crate) fn insert(&self, val: *mut T) -> *mut Self
    {
        let mut current = self;

        loop
        {
            match current.value.compare_exchange(
                std::ptr::null_mut(),
                val,
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            {
                Ok(_old) => break current as *const Self as _,
                Err(_) =>
                {
                    match unsafe { current.next.load(Ordering::SeqCst).as_ref() }
                    {
                        Some(next) => current = next,
                        None =>
                        {
                            let node = Box::into_raw(Box::new(Self::new(val)));
                            current.append(node);
                            break node;
                        }
                    }
                }
            }
        }
    }

    /// Unregisters this nodes value if it is `current`.
    ///
    /// Returns `None` if this node unregistered from `current`.
    /// Returns `Some(actual)` if this node's value is not `current` where `actual` will hold the actual value of this node.
    pub(crate) fn unregister(&self, current: *mut T) -> Option<*mut T>
    {
        self.value
            .compare_exchange(
                current,
                std::ptr::null_mut(),
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            .err()
    }

    unsafe fn free_child(&self)
    {
        let val = self.next.swap(std::ptr::null_mut(), Ordering::SeqCst);

        if !val.is_null()
        {
            drop(unsafe { Box::from_raw(val) });
        }
    }
}
