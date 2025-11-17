use std::{ops::Deref, sync::atomic::AtomicPtr};


// Making the fields of this struct not pub makes this trait effectively sealed...
pub struct HeapedPtr<T>(pub(crate) *mut T);


impl<T> From<HeapedPtr<T>> for AtomicPtr<T>
{
    fn from(value: HeapedPtr<T>) -> Self
    {
        AtomicPtr::new(value.0)
    }
}


impl<T> From<&AtomicPtr<T>> for HeapedPtr<T>
{
    fn from(value: &AtomicPtr<T>) -> Self
    {
        Self(value.load(std::sync::atomic::Ordering::SeqCst))
    }
}


impl<T> Copy for HeapedPtr<T> {}
impl<T> Clone for HeapedPtr<T>
{
    fn clone(&self) -> Self
    {
        *self
    }
}


impl<T> Deref for HeapedPtr<T>
{
    type Target = T;

    fn deref(&self) -> &Self::Target
    {
        unsafe { &*self.0 }
    }
}


/// Provides a function to acquire a heap allocated pointer to self.
#[allow(clippy::missing_safety_doc)]
pub unsafe trait Heaped<T>
{
    /// Provides a pointer to a Target on the heap, that is never manually deallocated.
    #[allow(clippy::missing_safety_doc)]
    unsafe fn heaped_ptr(self) -> HeapedPtr<T>;
}


unsafe impl<T> Heaped<T> for Box<T>
{
    unsafe fn heaped_ptr(self) -> HeapedPtr<T>
    {
        HeapedPtr(Box::into_raw(self))
    }
}


unsafe impl<T> Heaped<T> for HeapedPtr<T>
{
    unsafe fn heaped_ptr(self) -> HeapedPtr<T>
    {
        HeapedPtr(self.0)
    }
}


unsafe impl<T> Heaped<T> for &HeapedPtr<T>
{
    unsafe fn heaped_ptr(self) -> HeapedPtr<T>
    {
        HeapedPtr(self.0)
    }
}


//TODO: having this might be confusing???
// unsafe impl<T: Clone> Heaped<T> for &Guard<T>
// {
//     unsafe fn heaped_ptr(self) -> HeapedPtr<T>
//     {
//         let cloned: T = self.as_ref().clone();
//         unsafe { cloned.heaped_ptr() }
//     }
// }


unsafe impl<T> Heaped<T> for T
{
    unsafe fn heaped_ptr(self) -> HeapedPtr<T>
    {
        HeapedPtr(Box::into_raw(Box::new(self)))
    }
}
