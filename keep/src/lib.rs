#![feature(ptr_as_ref_unchecked)]


mod domain;
mod guard;
mod heaped;
mod keep;


pub use domain::Domain;
pub use guard::Guard;
pub use heaped::Heaped;
pub use keep::Keep;


#[cfg(test)]
mod tests
{
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};


    #[test]
    fn look_and_feel()
    {
        let domain = Domain::new();

        let keep = domain.keep("39");

        {
            let k2 = keep.clone();
            drop(k2);
        }

        let guard_a = keep.read();
        let guard_b = keep.read();

        drop(keep);

        let keep_b: Keep<&str> = guard_b.into();
        keep_b.store("Mk");
        assert_eq!("Mk", *keep_b.read());
        drop(keep_b);

        assert_eq!("39", *guard_a);
    }


    #[test]
    fn drops()
    {
        static DID_DROP: AtomicBool = AtomicBool::new(false);

        struct WithDrop(usize);

        impl Drop for WithDrop
        {
            fn drop(&mut self)
            {
                DID_DROP.store(true, Ordering::Relaxed);
            }
        }

        let domain = Domain::new();
        let with_drop = domain.keep(WithDrop(39));
        let guard = with_drop.read();

        {
            let _cloned = with_drop.clone();
            let _upgrade = Keep::from(guard.clone());
        }

        drop(with_drop);

        assert_eq!(false, DID_DROP.load(Ordering::Relaxed));
        assert_eq!(39, guard.0);

        drop(guard);

        assert_eq!(true, DID_DROP.load(Ordering::Relaxed));
    }
}
