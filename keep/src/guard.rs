use crate::domain::{Domain, GuardPtr};
use std::ops::Deref;


pub struct Guard<'a, T>
{
    pub(crate) domain: &'a Domain,
    pub(crate) guard_ptr: &'a GuardPtr,
    pub(crate) reference: &'a T,
}


impl<'a, T> Guard<'a, T>
{
    pub(crate) fn new(reference: &'a T, domain: &'a Domain, guard_ptr: &'a GuardPtr) -> Self
    {
        Self {
            reference,
            domain,
            guard_ptr,
        }
    }
}


impl<T> AsRef<T> for Guard<'_, T>
{
    fn as_ref(&self) -> &T
    {
        self.reference
    }
}


impl<T> Deref for Guard<'_, T>
{
    type Target = T;

    fn deref(&self) -> &Self::Target
    {
        self.reference
    }
}


impl<T> Drop for Guard<'_, T>
{
    fn drop(&mut self)
    {
        self.guard_ptr.drop_ptr(self.reference as *const T as _);
        self.domain.cleanup(self.reference as *const T as *mut T);
    }
}


impl<'a, T> Clone for Guard<'a, T>
{
    fn clone(&self) -> Self
    {
        let guard_ptr = self.domain.protect(self.reference as *const T as _);

        Self {
            guard_ptr,
            reference: self.reference,
            domain: self.domain,
        }
    }
}
