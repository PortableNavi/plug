#![allow(unused)]
#![feature(arbitrary_self_types)]

mod entry;
mod map;
mod table;


pub use map::Plugmap;


#[cfg(test)]
mod tests
{
    use super::Plugmap;

    #[test]
    fn look_and_feel()
    {
        let map = Plugmap::<&str, usize>::new();

        assert!(map.get(&"mk").is_none());
        assert!(map.insert("mk", 39).is_none());
        assert!(map.get(&"other_key").is_none());
        assert_eq!(39, *map.get(&"mk").unwrap());
        assert_eq!(39, *map.insert("mk", 393939).unwrap());
        assert_eq!(393939, *map.get(&"mk").unwrap());
    }

    #[test]
    fn many_keys()
    {
        let map = Plugmap::new();

        for i in 1..100
        {
            map.insert(i, i * 200);
        }

        for i in 1..100
        {
            assert_eq!(i * 200, *map.get(&i).unwrap())
        }
    }
}
