#![feature(negative_impls)]


pub(crate) mod guard;
pub(crate) mod heaped;
pub(crate) mod keep;
pub(crate) mod tracked_atomic;


pub use guard::Guard;
pub use heaped::Heaped;
pub use keep::Keep;


#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn look_and_feel()
    {
        let a = Guard::new("a");
        let b = Guard::new("b");

        let backup = a.read();

        a.swap_guard(&b);

        assert_eq!(*a, "b");
        assert_eq!(*b, "a");
        assert_eq!(*backup, "a");

        let old = backup.swap("c");

        assert_eq!(*old, "a");
        assert_eq!(*backup, "c");
    }
}
