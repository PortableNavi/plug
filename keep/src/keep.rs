use crate::{
    Heaped,
    domain::{Domain, GuardPtr},
    guard::Guard,
};
use std::sync::atomic::{AtomicPtr, Ordering};


pub struct Keep<'a, T>
{
    pub(crate) ptr: AtomicPtr<T>,
    pub(crate) domain: &'a Domain,
    pub(crate) guard_ptr: &'a GuardPtr,
}


impl<'a, T> Keep<'a, T>
{
    pub fn new<'h>(domain: &'a Domain, val: impl Heaped<'h, T>) -> Self
    where
        T: 'h,
    {
        let ptr = unsafe { val.heaped().ptr() };

        Self {
            domain,
            ptr: AtomicPtr::new(ptr),
            guard_ptr: domain.protect(ptr as _),
        }
    }

    pub fn read(&self) -> Guard<'a, T>
    {
        let ptr = self.ptr.load(Ordering::SeqCst);
        let guard_ptr = self.domain.protect(ptr as _);

        Guard::new(unsafe { ptr.as_ref_unchecked() }, self.domain, guard_ptr)
    }

    pub fn store<'h>(&self, val: impl Heaped<'h, T>)
    where
        T: 'h,
    {
        let ptr = unsafe { val.heaped().ptr() };
        self.domain.cleanup(self.swap_ptr(ptr));
    }

    pub fn swap<'h>(&self, val: impl Heaped<'h, T>) -> Guard<'a, T>
    where
        T: 'h,
    {
        let ptr = unsafe { val.heaped().ptr() };
        let old = self.swap_ptr(ptr);
        let guard_ptr = self.domain.protect(old as _);

        Guard::new(unsafe { old.as_ref_unchecked() }, self.domain, guard_ptr)
    }

    pub fn exchange<'h>(
        &self,
        current: &T,
        new: impl Heaped<'h, T>,
    ) -> Result<Guard<'a, T>, Guard<'a, T>>
    where
        T: 'h,
    {
        let current_ptr = current as *const T as *mut T;
        let new_ptr = unsafe { new.heaped().ptr() };

        match self
            .ptr
            .compare_exchange(current_ptr, new_ptr, Ordering::SeqCst, Ordering::SeqCst)
        {
            Ok(old_ptr) =>
            {
                self.guard_ptr.swap(old_ptr as _, new_ptr as _);
                let guard_ptr = self.domain.protect(old_ptr as _);
                Ok(Guard::new(
                    unsafe { old_ptr.as_ref_unchecked() },
                    self.domain,
                    guard_ptr,
                ))
            }

            Err(new_ptr) => Err(Guard::new(
                unsafe { &*new_ptr },
                self.domain,
                self.domain.protect(new_ptr as _),
            )),
        }
    }

    fn swap_ptr(&self, ptr: *mut T) -> *mut T
    {
        let old = self.ptr.swap(ptr, Ordering::SeqCst);
        self.guard_ptr.swap(old as _, ptr as _);

        old
    }
}


impl<T> Drop for Keep<'_, T>
{
    fn drop(&mut self)
    {
        self.domain.cleanup(self.swap_ptr(std::ptr::null_mut()));
    }
}


impl<'a, T: 'a> Clone for Keep<'a, T>
{
    fn clone(&self) -> Self
    {
        let ptr = self.ptr.load(Ordering::SeqCst);

        Self {
            ptr: AtomicPtr::new(ptr),
            domain: self.domain,
            guard_ptr: self.domain.protect(ptr as _),
        }
    }
}


impl<'a, T> From<Guard<'a, T>> for Keep<'a, T>
{
    fn from(val: Guard<'a, T>) -> Self
    {
        let ptr = val.reference as *const T as _;

        Keep {
            ptr: AtomicPtr::new(ptr),
            domain: val.domain,
            guard_ptr: val.domain.protect(ptr as _), // The guards guards_pointer cannot be reused because this guard is dropped and will free its guard_pointer on drop
        }
    }
}
