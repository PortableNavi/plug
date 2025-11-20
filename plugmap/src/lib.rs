#![allow(unused)]


mod entry;
mod map;
mod table;


pub use map::PlugMap;


#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn look_and_feel()
    {
        let map = PlugMap::<usize, &str>::new();

        assert!(
            map.get(&39).is_none(),
            "Empty map did not return None on get(..)"
        );

        map.insert(39, "Briar");

        assert_eq!("Briar", *map.insert(39, "Miku").unwrap());

        assert_eq!(
            Some("Miku"),
            map.get(&39).map(|v| *v),
            "get(..) did not return the current value"
        );
    }
}
