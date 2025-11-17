use crate::{Guard, Heaped};


pub struct Keep<T>(Guard<T>);


impl<T> Keep<T>
{
    #[inline]
    pub fn new(val: impl Heaped<T>) -> Self
    {
        Self(Guard::new(val))
    }

    #[inline]
    pub fn read(&self) -> Guard<T>
    {
        self.0.read()
    }

    #[inline]
    pub fn swap_guard(&self, other: &Guard<T>)
    {
        self.0.swap_guard(other);
    }

    #[inline]
    pub fn swap(&self, val: impl Heaped<T>) -> Guard<T>
    {
        self.0.swap(val)
    }

    #[inline]
    pub fn write(&self, val: impl Heaped<T>)
    {
        self.0.write(val);
    }
}


impl<T> From<Guard<T>> for Keep<T>
{
    fn from(value: Guard<T>) -> Self
    {
        Self(value)
    }
}


impl<T> Clone for Keep<T>
{
    fn clone(&self) -> Self
    {
        Self(self.0.clone())
    }
}
