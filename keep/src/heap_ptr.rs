pub struct HeapPtr<T>(*mut T);
impl<T> HeapPtr<T>
{
    #[inline]
    pub fn as_ptr(&self) -> *mut T
    {
        self.0
    }

    /// Frees the memory this `HeapPtr` is pointing at.
    ///
    /// # Safety
    /// The caller is responsible for avoiding use after free errors.
    #[inline]
    pub(crate) unsafe fn free(self)
    {
        drop(unsafe { Box::from_raw(self.0) })
    }

    #[inline]
    pub(crate) fn from_ptr(ptr: *mut T) -> Self
    {
        Self(ptr)
    }
}


impl<T> AsRef<T> for HeapPtr<T>
{
    fn as_ref(&self) -> &T
    {
        unsafe { &*self.0 }
    }
}


impl<T> Copy for HeapPtr<T> {}
#[allow(clippy::non_canonical_clone_impl)]
impl<T> Clone for HeapPtr<T>
{
    fn clone(&self) -> Self
    {
        Self(self.0)
    }
}


/// # Safety
/// The resulting `HeapPtr<T>` must point to non-null, aligned and heap allocated `T`.
pub unsafe trait Heaped<T>
{
    fn heap_ptr(self) -> HeapPtr<T>;
}


unsafe impl<T> Heaped<T> for T
{
    #[inline]
    fn heap_ptr(self) -> HeapPtr<T>
    {
        Box::new(self).heap_ptr()
    }
}


unsafe impl<T> Heaped<T> for Box<T>
{
    #[inline]
    fn heap_ptr(self) -> HeapPtr<T>
    {
        HeapPtr(Box::into_raw(self))
    }
}


unsafe impl<T> Heaped<T> for HeapPtr<T>
{
    #[inline]
    fn heap_ptr(self) -> HeapPtr<T>
    {
        self
    }
}
