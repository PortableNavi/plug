use crate::entry::{Entry, EntryNode};
use keep::*;
use std::sync::atomic::AtomicUsize;


pub struct Table<Key, Val>
{
    size: usize,
    capacity: usize,
    entry_count: AtomicUsize,
    entries: Box<[Keep<Entry<Key, Val>>]>,
}


impl<Key, Val> Table<Key, Val>
where
    Key: Eq,
{
    pub fn new(size: usize) -> Self
    {
        let mut entries = Box::new_uninit_slice(1 << size);

        for entry in &mut entries
        {
            entry.write(Keep::new(Entry::Empty));
        }

        Self {
            size,
            capacity: 1 << size,
            entry_count: AtomicUsize::new(0),
            entries: unsafe { entries.assume_init() },
        }
    }

    pub fn get(&self, key: &Key, hash: u64) -> Option<Guard<Val>>
    {
        self.entry_of(hash).read().search(key)
    }

    pub fn insert(&self, entry_node: EntryNode<Key, Val>) -> Option<Guard<Val>>
    {
        let hash = entry_node.hash();
        let entry_node = Keep::new(entry_node);
        let entry = self.entry_of(hash);

        loop
        {
            let entry_guard = entry.read();
            let entry_marker = entry.mark();

            match &*entry_guard
            {
                Entry::Empty =>
                {
                    let new = Keep::new(Entry::Head(entry_node.clone()));

                    if entry.exchange_with(entry_marker, &new).is_err()
                    {
                        continue;
                    }

                    break None;
                }

                Entry::Head(keep) => break keep.read().update(&entry_node),
            }
        }
    }

    #[inline]
    fn index_of(&self, hash: u64) -> usize
    {
        hash as usize & ((1 << self.size) - 1)
    }

    #[inline]
    fn entry_at(&self, index: usize) -> &Keep<Entry<Key, Val>>
    {
        &self.entries[index]
    }

    #[inline]
    fn entry_of(&self, hash: u64) -> &Keep<Entry<Key, Val>>
    {
        &self.entries[self.index_of(hash)]
    }
}
