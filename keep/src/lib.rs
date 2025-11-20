mod guard;
mod heap_ptr;
mod keep;
mod tracked_atomic;


use std::sync::atomic::{AtomicPtr, Ordering};


pub use guard::Guard;
pub use heap_ptr::{HeapPtr, Heaped};
pub use keep::{Keep, KeepMarker};


pub(crate) fn atomic_swap<T>(a: &AtomicPtr<T>, b: &AtomicPtr<T>)
{
    let mut ptr_a = a.load(Ordering::SeqCst);
    let mut ptr_b = b.load(Ordering::SeqCst);

    loop
    {
        if let Err(changed) = a.compare_exchange(ptr_a, ptr_b, Ordering::SeqCst, Ordering::SeqCst)
        {
            ptr_a = changed;
            continue;
        }

        if let Err(changed) = b.compare_exchange(ptr_b, ptr_a, Ordering::SeqCst, Ordering::SeqCst)
        {
            ptr_b = changed;
            continue;
        }

        break;
    }
}
