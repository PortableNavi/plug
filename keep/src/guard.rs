use crate::tracked_atomic::GuardNode;
use std::ops::Deref;


pub struct Guard<T>
{
    guard_node: *mut GuardNode<T>,
    reference: *mut T,
}


impl<T> Guard<T>
{
    pub(crate) fn new(guard_node: *mut GuardNode<T>, reference: *mut T) -> Self
    {
        Self {
            guard_node,
            reference,
        }
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *mut T
    {
        self.reference
    }
}


impl<T> Clone for Guard<T>
{
    fn clone(&self) -> Self
    {
        Self {
            guard_node: self.guard_node,
            reference: self.reference,
        }
    }
}


impl<T> Deref for Guard<T>
{
    type Target = T;

    fn deref(&self) -> &T
    {
        unsafe { &*self.reference }
    }
}


impl<T> Drop for Guard<T>
{
    fn drop(&mut self)
    {
        unsafe { &*self.guard_node }.unregister(self.reference);
    }
}


impl<T> std::fmt::Debug for Guard<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("Guard")
            .field("guard_node", &self.guard_node)
            .field("reference", &self.reference)
            .finish()
    }
}
