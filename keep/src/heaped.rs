use crate::Guard;
pub struct HeapedRef<'a, T>(&'a T);


#[allow(unused)]
impl<'a, T> HeapedRef<'a, T>
{
    #[inline]
    pub(crate) fn shared(&self) -> &T
    {
        self.0
    }

    #[inline]
    pub(crate) fn ptr(&self) -> *mut T
    {
        self.0 as *const T as _
    }
}


#[allow(clippy::missing_safety_doc)]
pub unsafe trait Heaped<'a, T>
{
    unsafe fn heaped(self) -> HeapedRef<'a, T>;
}


unsafe impl<'d, 'a, T> Heaped<'a, T> for &'a Guard<'d, T>
where
    'd: 'a,
{
    unsafe fn heaped(self) -> HeapedRef<'a, T>
    {
        HeapedRef(self.reference)
    }
}


unsafe impl<'d, 'a, T> Heaped<'a, T> for Guard<'d, T>
where
    'd: 'a,
{
    unsafe fn heaped(self) -> HeapedRef<'a, T>
    {
        HeapedRef(self.reference)
    }
}


unsafe impl<'a, T> Heaped<'a, T> for T
{
    unsafe fn heaped(self) -> HeapedRef<'a, T>
    {
        let ptr = Box::into_raw(Box::new(self));
        HeapedRef(unsafe { &*ptr })
    }
}


unsafe impl<'a, T> Heaped<'a, T> for Box<T>
{
    unsafe fn heaped(self) -> HeapedRef<'a, T>
    {
        let ptr = Box::into_raw(self);
        HeapedRef(unsafe { &*ptr })
    }
}
