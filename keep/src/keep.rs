use crate::{Guard, Heaped, atomic_swap, tracked_atomic::TrackedAtomic};
use std::sync::atomic::{AtomicPtr, Ordering};


pub struct Keep<T>
{
    tracked_atomic: AtomicPtr<TrackedAtomic<T>>,
}


impl<T> Keep<T>
{
    pub fn new(value: impl Heaped<T>) -> Self
    {
        let tracked_atomic = TrackedAtomic::new(value).heap_ptr();
        tracked_atomic.as_ref().register_keep();

        Self {
            tracked_atomic: AtomicPtr::new(tracked_atomic.as_ptr()),
        }
    }

    /// Swaps the referenced tracked atomic of two keeps.
    ///
    /// If you need to swap the values of two keeps use `Keep::swap_with(..)`,
    /// if you want to swap the value a keep use `Keep::swap(..)` instead.
    pub fn swap_with(&self, other: &Self)
    {
        atomic_swap(&self.tracked_atomic, &other.tracked_atomic);
    }

    /// Reads the current value from this keep's tracked atomic
    pub fn read(&self) -> Guard<T>
    {
        self.load().read()
    }

    /// Stores a new value in this keep's tracked atomic
    pub fn write(&self, value: impl Heaped<T>)
    {
        self.load().write(value)
    }

    /// Swaps the current value with `value` and returns the old one.
    ///
    /// If you need to swap the values of two keeps use `Keep::swap_with(..)`,
    /// if you want to swap the value a keep use `Keep::swap(..)` instead.
    pub fn swap(&self, value: impl Heaped<T>) -> Guard<T>
    {
        self.load().swap(value)
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
        self.load().exchange(current, new)
    }

    #[inline]
    fn load(&self) -> &TrackedAtomic<T>
    {
        unsafe { &*self.tracked_atomic.load(Ordering::SeqCst) }
    }
}


impl<T> Clone for Keep<T>
{
    fn clone(&self) -> Self
    {
        let tracked_atomic = self.tracked_atomic.load(Ordering::SeqCst);
        unsafe { &*tracked_atomic }.register_keep();

        Self {
            tracked_atomic: AtomicPtr::new(tracked_atomic),
        }
    }
}


impl<T> Drop for Keep<T>
{
    fn drop(&mut self)
    {
        unsafe { &*self.tracked_atomic.load(Ordering::SeqCst) }.unregister_keep();
    }
}
