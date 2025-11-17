use crate::table::Table;
use keep::{Domain, Guard, Heaped, Keep};
use parking_lot::Mutex;


pub enum Entry<'d, K, V>
{
    Head(Keep<'d, EntryNode<'d, K, V>>),
    Moved(Guard<'d, Table<'d, K, V>>),
    Empty,
}


impl<'d, K, V> Entry<'d, K, V>
where
    K: Eq,
{
    pub fn find(&self, key: &K) -> Option<Guard<'d, EntryNode<'d, K, V>>>
    {
        match self
        {
            Entry::Head(keep) => keep.read().find(key),
            Self::Moved(_) => todo!(),
            Entry::Empty => None,
        }
    }
}


pub struct EntryNode<'d, K, V>
{
    key: K,
    value: Keep<'d, V>,
    hash: u64,
    next: Keep<'d, Option<Keep<'d, Self>>>,
    lock: Mutex<()>,
}


impl<'d, K, V> EntryNode<'d, K, V>
where
    K: Eq,
{
    pub fn new<'h>(key: K, val: impl Heaped<'h, V>, hash: u64, domain: &'d Domain) -> Self
    where
        V: 'h,
    {
        Self {
            key,
            value: domain.keep(val),
            hash,
            next: domain.keep(None),
            lock: Mutex::new(()),
        }
    }

    pub fn find(self: Guard<'d, Self>, key: &K) -> Option<Guard<'d, Self>>
    {
        let mut current = self;

        loop
        {
            if current.key == *key
            {
                break Some(current);
            }

            current = match current.next.read().as_ref()
            {
                Some(next) => next.read(),
                None => break None,
            }
        }
    }

    pub fn pop() -> _ {}

    pub fn update(self: Guard<'d, Self>, entry: Keep<'d, Self>) -> (usize, Option<Guard<'d, V>>)
    {
        let mut current = self.clone();
        let mut depth = 0;
        let _lock = self.lock.lock();
        let guard = entry.read();

        loop
        {
            if current.key == guard.key
            {
                break (depth, Some(current.value.swap(guard.value.read())));
            }

            match &*current.next.read()
            {
                Some(next) => current = next.read(),
                None =>
                {
                    current.next.store(Some(entry));
                    break (depth, None);
                }
            }

            depth += 1;
        }
    }

    #[inline]
    pub fn value(&self) -> Guard<'d, V>
    {
        self.value.read()
    }
}
