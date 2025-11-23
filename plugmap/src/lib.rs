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

        assert_eq!(
            Some("Briar"),
            map.get(&39).map(|v| *v),
            "get(..) did not return the current value"
        );

        assert_eq!("Briar", *map.insert(39, "Miku").unwrap());

        assert_eq!(
            Some("Miku"),
            map.get(&39).map(|v| *v),
            "get(..) did not return the current value"
        );
    }


    #[test]
    fn remove()
    {
        let map = PlugMap::<u32, &str>::new();

        assert_eq!(None, map.remove(&39));
        map.insert(39, "Briar");
        assert_eq!(Some("Briar"), map.remove(&39).map(|g| *g));
        assert_eq!(None, map.remove(&39));
        assert_eq!(None, map.insert(39, "Other"));
        assert_eq!(Some("Other"), map.remove(&39).map(|g| *g));
        assert_eq!(None, map.remove(&39));
    }


    #[test]
    fn many_entries()
    {
        let map = PlugMap::new();

        for i in 0..100
        {
            map.insert(i, i.to_string());
        }

        assert_eq!(Some("39"), map.get(&39).as_ref().map(|g| g.as_str()));
        assert_eq!(Some("39"), map.remove(&39).as_ref().map(|g| g.as_str()));
        assert_eq!(None, map.remove(&39).as_ref().map(|g| g.as_str()));
        assert_eq!(None, map.get(&39));
        assert_eq!(Some("31"), map.get(&31).as_ref().map(|g| g.as_str()));
    }
}
