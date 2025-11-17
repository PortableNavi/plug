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

    pub fn insert(&self, entry_node: EntryNode<Key, Val>) -> Option<Guard<EntryNode<Key, Val>>>
    {
        let entry = self.entry_of(entry_node.hash());
        let entry_node = Keep::new(entry_node);

        loop
        {
            match &*entry
            {
                Entry::Empty =>
                {
                    // If the entry has changed while exchanging, reload it and try again...
                    if let Err(_) = entry.exchange(Entry::Head(entry_node.clone()))
                    {
                        entry.reload();
                        continue;
                    }

                    return None;
                }

                Entry::Head(keep) => todo!(),
            }
        }

        None
    }

    #[inline]
    fn index_of(&self, hash: u64) -> usize
    {
        hash as usize & ((1 << self.size) - 1)
    }

    #[inline]
    fn entry_at(&self, index: usize) -> Guard<Entry<Key, Val>>
    {
        self.entries[index].read()
    }

    #[inline]
    fn entry_of(&self, hash: u64) -> Guard<Entry<Key, Val>>
    {
        self.entries[self.index_of(hash)].read()
    }
}
