use crate::{Heaped, keep::Keep};
use std::sync::atomic::{AtomicPtr, Ordering};


pub struct Domain
{
    guardptr_list: GuardPtr,
}


impl Default for Domain
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl Domain
{
    pub const fn new() -> Self
    {
        Self {
            guardptr_list: GuardPtr::new(std::ptr::null_mut()),
        }
    }

    /// Acquires a free guard pointer, protects `ptr` with it and returns a reference to the guard pointer.
    pub(crate) fn protect(&self, ptr: *mut ()) -> &GuardPtr
    {
        let mut free = self.guardptr_list.fetch_free();

        while !free.protect(ptr)
        {
            free = self.guardptr_list.fetch_free();
        }

        free
    }

    /// QOL-Function for `Keep::new`
    #[inline]
    pub fn keep<'h, T>(&self, val: impl Heaped<'h, T>) -> Keep<'_, T>
    where
        T: 'h,
    {
        Keep::new(self, val)
    }

    pub(crate) fn cleanup<T>(&self, ptr: *mut T)
    {
        if !self.guardptr_list.search(ptr as _)
        {
            drop(unsafe { Box::from_raw(ptr) });
        }
    }
}


pub(crate) struct GuardPtr
{
    ptr: AtomicPtr<()>,
    next: AtomicPtr<Self>,
}


impl GuardPtr
{
    #[inline]
    const fn new(ptr: *mut ()) -> Self
    {
        Self {
            ptr: AtomicPtr::new(ptr),
            next: AtomicPtr::new(std::ptr::null_mut()),
        }
    }

    /// Make this guard ptr free its guard to `ptr` if it was guarding `ptr`.
    ///
    /// Returns `true` if this guard freed `ptr` otherwhise `false`
    pub(crate) fn drop_ptr(&self, ptr: *mut ()) -> bool
    {
        self.ptr
            .compare_exchange(
                ptr,
                std::ptr::null_mut(),
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            .is_ok()
    }

    /// Tries to protect `ptr`. Returns `true` on success.
    fn protect(&self, ptr: *mut ()) -> bool
    {
        self.swap(std::ptr::null_mut(), ptr)
    }


    pub(crate) fn swap(&self, old: *mut (), ptr: *mut ()) -> bool
    {
        self.ptr
            .compare_exchange(old, ptr, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    /// Searches the list for a free guard pointer
    fn search_free(&self) -> Result<&Self, &Self>
    {
        let mut current = self;

        while !current.ptr.load(Ordering::SeqCst).is_null()
        {
            current = match unsafe { current.next.load(Ordering::SeqCst).as_ref() }
            {
                Some(next) => next,
                None => return Err(current),
            };
        }

        Ok(current)
    }

    /// Finds or creates a free guard pointer and returns a reference to it.
    fn fetch_free(&self) -> &Self
    {
        // Search the list for a free guard pointer...
        match self.search_free()
        {
            // Found a free guard pointer -> return it.
            Ok(guard_ptr) => guard_ptr,

            // There is no free guard pointer -> create a new one.
            Err(tail) =>
            {
                // Create a new guard pointer, move it to the heap, destroy its drop and get a raw pointer to it.
                let guard_ptr = Box::into_raw(Box::new(Self::new(std::ptr::null_mut())));

                // Now walk the linked guard pointer list to its tail and try to append to new guard pointer...
                let mut current = tail;
                'append: loop
                {
                    match unsafe { current.next.load(Ordering::SeqCst).as_ref() }
                    {
                        // The current node is not the tail, look at the next one.
                        Some(next) => current = next,

                        // Current node is currently the tail, try to append to new node using a CAS
                        None => match current.next.compare_exchange(
                            std::ptr::null_mut(),
                            guard_ptr,
                            Ordering::SeqCst,
                            Ordering::SeqCst,
                        )
                        {
                            // CAS succeeded, break with the new node
                            Ok(_) =>
                            unsafe {
                                break 'append guard_ptr.as_ref_unchecked();
                            },

                            // CAS failed, read the current node again, since it is no longer the tail.
                            Err(_) => continue 'append,
                        },
                    }
                }
            }
        }
    }

    /// Searches this list for at least one occurence of `ptr`
    fn search(&self, ptr: *mut ()) -> bool
    {
        let mut current = self;

        while current.ptr.load(Ordering::SeqCst) != ptr
        {
            current = match unsafe { current.next.load(Ordering::SeqCst).as_ref() }
            {
                Some(next) => next,
                None => return false,
            }
        }

        true
    }
}
