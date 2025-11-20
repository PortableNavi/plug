use crate::{Guard, HeapPtr, Heaped};
use std::{
    ptr,
    sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering},
};


pub struct TrackedAtomic<T>
{
    ptr: HeapPtr<AtomicPtr<T>>,
    guard_ptr: HeapPtr<AtomicPtr<GuardNode<T>>>,
    domain: HeapPtr<GuardNode<T>>,
    keep_count: HeapPtr<AtomicUsize>,
}


impl<T> TrackedAtomic<T>
{
    pub fn new(value: impl Heaped<T>) -> Self
    {
        let value = value.heap_ptr();

        let domain = GuardNode::new(ptr::null_mut(), ptr::null_mut()).heap_ptr();
        unsafe { &mut *domain.as_ptr() }.head = domain.as_ptr();

        let guard_ptr = AtomicPtr::new(domain.as_ref().register(value.as_ptr())).heap_ptr();


        Self {
            ptr: AtomicPtr::new(value.as_ptr()).heap_ptr(),
            guard_ptr,
            domain,
            keep_count: AtomicUsize::new(0).heap_ptr(),
        }
    }

    pub fn read(&self) -> Guard<T>
    {
        let ptr = self.ptr.as_ref().load(Ordering::SeqCst);
        let guard_node = self.domain.as_ref().register(ptr);
        Guard::new(guard_node, ptr)
    }

    /// Stores a new value in this tracked atomic
    pub fn write(&self, value: impl Heaped<T>)
    {
        let value = value.heap_ptr();

        // Store the new value and create a guard node for it
        let old = self.ptr.as_ref().swap(value.as_ptr(), Ordering::SeqCst);
        let node = self.domain.as_ref().register(value.as_ptr());

        // now store the guard and unregister the old value
        let old_node = self.guard_ptr.as_ref().swap(node, Ordering::SeqCst);
        unsafe { &*old_node }.unregister(old);
    }

    /// Swaps the current value with `value` and returns the old one.
    pub fn swap(&self, value: impl Heaped<T>) -> Guard<T>
    {
        let value = value.heap_ptr();

        // Store the new value and create a guard node for it
        let old = self.ptr.as_ref().swap(value.as_ptr(), Ordering::SeqCst);
        let node = self.domain.as_ref().register(value.as_ptr());

        // Now store the guard node
        let old_node = self.guard_ptr.as_ref().swap(node, Ordering::SeqCst);

        // The old node can be reused for the returned guard
        Guard::new(old_node, old)
    }

    /// Exchanges the value with `new` if the current value is `current`.
    ///
    /// This does not check for semantic equality, instead the pointers that guarded are compared
    ///
    /// # Returns
    /// * `Ok(Guard<T>)` containing the old value on success (actual == `current`)
    /// * `Err(Guard<T>)` containing the actual current value on failure (actual != `current`)
    pub fn exchange(&self, current: &Guard<T>, new: impl Heaped<T>) -> Result<Guard<T>, Guard<T>>
    {
        let new = new.heap_ptr();

        match self.ptr.as_ref().compare_exchange(
            current.as_ptr(),
            new.as_ptr(),
            Ordering::SeqCst,
            Ordering::SeqCst,
        )
        {
            Ok(old) =>
            {
                let guard_node = self.domain.as_ref().register(new.as_ptr());
                let old_node = self.guard_ptr.as_ref().swap(guard_node, Ordering::SeqCst);
                Ok(Guard::new(old_node, old))
            }

            Err(cur) =>
            {
                let guard_node = self.domain.as_ref().register(cur);
                Err(Guard::new(guard_node, cur))
            }
        }
    }

    pub fn unregister_keep(&self)
    {
        // If there are no more keeps that reference this tracked atomic, it can be cleaned up.
        if 1 >= self.keep_count.as_ref().fetch_sub(1, Ordering::SeqCst)
        {
            // Unregister the current value from the domain
            unsafe { &*self.guard_ptr.as_ref().load(Ordering::SeqCst) }.unregister_self();

            unsafe {
                // Now free everything except guard nodes
                self.ptr.free();
                self.keep_count.free();

                // Keep a reference to the head
                let head = self.domain;

                // Now free this struct, leaving only the guard nodes alive,
                // which are cleaned up by unregistering nodes from the domain
                HeapPtr::from_ptr(self as *const _ as *mut Self).free();

                // Check if the guard list is already dead, in this case clean the head
                if head.as_ref().dead.load(Ordering::SeqCst)
                {
                    head.free();
                }
                // if its not dead, indicate that the tracked atomic is dead and the head can kill itself
                else
                {
                    head.as_ref().dead.store(true, Ordering::SeqCst);
                }
            };
        }
    }

    pub fn register_keep(&self)
    {
        self.keep_count.as_ref().fetch_add(1, Ordering::SeqCst);
    }
}


pub struct GuardNode<T>
{
    head: *mut Self,
    dead: AtomicBool,
    value: AtomicPtr<T>,
    next: AtomicPtr<Self>,
}


impl<T> GuardNode<T>
{
    fn new(value: *mut T, head: *mut Self) -> Self
    {
        Self {
            head,
            dead: AtomicBool::new(false),
            value: AtomicPtr::new(value),
            next: AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Registers `value` inside the domain.
    fn register(&self, value: *mut T) -> *mut Self
    {
        match self.value.compare_exchange(
            ptr::null_mut(),
            value,
            Ordering::SeqCst,
            Ordering::SeqCst,
        )
        {
            Ok(_) => self as *const Self as _,
            Err(_) =>
            {
                let node = GuardNode::new(value, self.head).heap_ptr();
                let mut next = self.next.load(Ordering::SeqCst);

                loop
                {
                    if let Some(next) = unsafe { next.as_ref() }
                    {
                        unsafe { node.free() }; // This node is no longer needed.
                        break next.register(value);
                    }

                    match self.next.compare_exchange(
                        ptr::null_mut(),
                        node.as_ptr(),
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    )
                    {
                        Ok(_) => break node.as_ptr(),
                        Err(actual) => next = actual,
                    }
                }
            }
        }
    }

    /// Unregisters this guard from the domain if its guarding `value`
    pub fn unregister(&self, value: *mut T) -> bool
    {
        if value.is_null()
        {
            return false;
        }

        let result = self.value.compare_exchange(
            value,
            ptr::null_mut(),
            Ordering::SeqCst,
            Ordering::Relaxed,
        );

        // return false if this node did not unregister successfully
        if result.is_err()
        {
            return false;
        }

        // look if there's cleanup to do and keep track if the guard_domain holds any other guards than null
        let mut domain_dead = true;
        let mut mutation_over = true;
        let mut current = unsafe { &*self.head };

        while mutation_over || domain_dead
        {
            let ptr = current.value.load(Ordering::SeqCst);

            if ptr == value
            {
                mutation_over = false;
            }

            if !ptr.is_null()
            {
                domain_dead = false;
            }

            match unsafe { current.next.load(Ordering::SeqCst).as_ref() }
            {
                Some(next) => current = next,
                None => break,
            }
        }

        // If the mutation is over, free the associated value
        if mutation_over
        {
            unsafe { HeapPtr::from_ptr(value).free() };
        }


        // If the domain is dead, free all nodes except the head if its not marked as dead.
        if domain_dead
        {
            let head = self.head;

            let mut current = unsafe { &*head }
                .next
                .swap(ptr::null_mut(), Ordering::SeqCst);

            while let Some(curr) = unsafe { current.as_ref() }
            {
                let next = curr.next.swap(ptr::null_mut(), Ordering::SeqCst);
                unsafe { HeapPtr::from_ptr(current).free() };
                current = next;
            }

            // Now check if the head is dead and kill it, while marking it as dead.
            // if its just marked as dead here, the last surviving keep will kill it on drop.
            if unsafe { &*head }.dead.swap(true, Ordering::SeqCst)
            {
                unsafe { HeapPtr::from_ptr(head).free() };
            }
        }

        true
    }

    pub fn unregister_self(&self)
    {
        self.unregister(self.value.load(Ordering::SeqCst));
    }
}


impl<T> Clone for TrackedAtomic<T>
{
    fn clone(&self) -> Self
    {
        Self {
            ptr: self.ptr,
            guard_ptr: self.guard_ptr,
            domain: self.domain,
            keep_count: self.keep_count,
        }
    }
}
