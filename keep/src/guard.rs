use std::{cell::RefCell, ops::Deref};

use crate::{
    Heaped,
    heaped::HeapedPtr,
    tracked_atomic::{DomainNode, TrackedAtomic},
};


pub struct Guard<T>
{
    value: RefCell<HeapedPtr<T>>,
    node: RefCell<HeapedPtr<DomainNode<T>>>,
    tracked_atomic: RefCell<HeapedPtr<TrackedAtomic<T>>>,
}


//TODO: This is redundant, Guard cannot be sync because RefCell is !Sync.
//      Should i remove this, or should i keep this to absolutely make sure
//      that Guard is only Send and never !Sync ???
impl<T> !Sync for Guard<T> {}


impl<T> Guard<T>
{
    pub fn new(val: impl Heaped<T>) -> Self
    {
        let value = unsafe { val.heaped_ptr() };
        let tracked_atomic = unsafe { TrackedAtomic::new(value).heaped_ptr() };
        let node = tracked_atomic.add_mutation(value);

        Self {
            value: RefCell::new(value),
            node: RefCell::new(node),
            tracked_atomic: RefCell::new(tracked_atomic),
        }
    }

    pub fn read(&self) -> Self
    {
        let value = self.tracked_atomic.borrow().read();
        let node = self.tracked_atomic.borrow().add_mutation(value);

        Self {
            value: RefCell::new(value),
            node: RefCell::new(node),
            tracked_atomic: self.tracked_atomic.clone(),
        }
    }

    #[inline]
    pub fn reload(&self)
    {
        self.swap_guard(&self.read());
    }

    pub fn swap_guard(&self, other: &Guard<T>)
    {
        self.value.swap(&other.value);
        self.node.swap(&other.node);
        self.tracked_atomic.swap(&other.tracked_atomic);
    }

    pub fn swap(&self, val: impl Heaped<T>) -> Guard<T>
    {
        let val = unsafe { val.heaped_ptr() };
        let (_old, new_node) = self.tracked_atomic.borrow().swap(val);

        let new_guard = Guard {
            value: RefCell::new(val),
            node: RefCell::new(new_node),
            tracked_atomic: self.tracked_atomic.clone(),
        };

        self.swap_guard(&new_guard);
        new_guard
    }


    #[inline]
    pub fn write(&self, val: impl Heaped<T>)
    {
        drop(self.swap(val));
    }

    /// Exchanges the tracked atomics value with `val` only if the tracked atomics value still
    /// is the same value as is guarded by this guard.
    pub fn exchange(&self, val: impl Heaped<T>) -> Result<Guard<T>, Box<T>>
    {
        let val = unsafe { val.heaped_ptr() };
        let current = self.value.borrow().0;

        match self.tracked_atomic.borrow().exchange(current, val)
        {
            Ok((_old, new_node)) =>
            {
                let new_guard = Guard {
                    value: RefCell::new(val),
                    node: RefCell::new(new_node),
                    tracked_atomic: self.tracked_atomic.clone(),
                };

                self.swap_guard(&new_guard);
                Ok(new_guard)
            }

            Err(e) => Err(e),
        }
    }
}


impl<T> Clone for Guard<T>
{
    fn clone(&self) -> Self
    {
        Self {
            value: self.value.clone(),
            node: RefCell::new(
                self.tracked_atomic
                    .borrow()
                    .add_mutation(*self.value.borrow()),
            ),
            tracked_atomic: self.tracked_atomic.clone(),
        }
    }
}


impl<T> AsRef<T> for Guard<T>
{
    fn as_ref(&self) -> &T
    {
        unsafe { &*self.value.borrow().0 }
    }
}


impl<T> Deref for Guard<T>
{
    type Target = T;

    fn deref(&self) -> &Self::Target
    {
        self.as_ref()
    }
}


impl<T> Drop for Guard<T>
{
    fn drop(&mut self)
    {
        self.node.borrow().unregister(self.value.borrow().0);

        if let Some(boxed) = self
            .tracked_atomic
            .borrow()
            .try_clean_mutation_of(self.value.borrow().0)
        {
            drop(boxed);
        }

        if self.tracked_atomic.borrow().is_dead()
        {
            drop(unsafe { Box::from_raw(self.tracked_atomic.borrow().0) });
        }
    }
}
