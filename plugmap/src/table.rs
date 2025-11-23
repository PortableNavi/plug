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

    pub fn remove(&self, key: &Key, hash: u64) -> Option<Guard<Val>>
    where
        Val: std::fmt::Debug,
    {
        let entry = self.entry_of(hash);

        'outer: loop
        {
            let entry_guard = entry.read();
            let entry_marker = entry.mark();

            // Examine the entry:
            match &*entry_guard
            {
                // If the entry is empty return none
                Entry::Empty => break 'outer None,

                // If however the entry is not empty its node and its children will have to be searched for key:
                Entry::Head(keep) =>
                {
                    let next = keep.read().next().read();
                    let guard = keep.read().value();

                    // Check whether or not the head has a next node:
                    match &*next
                    {
                        // If this head does have a next node it (the head) must first be checked for a matching key.
                        // If the key matches the head needs to be replaced by its next child node,
                        // if it does not have a next child node it needs to be replaced by a Entry::Empty.
                        //
                        // If the head does not match but has children, the list of children must be searched for a matching key,
                        // if a matching key is found that child is replaced by its own direct child.
                        Some(next) =>
                        {
                            if keep.read().key() == key
                            {
                                if entry
                                    .exchange_with(
                                        entry_marker,
                                        &Keep::new(Entry::Head(next.clone())),
                                    )
                                    .is_err()
                                {
                                    continue 'outer;
                                }

                                break 'outer Some(guard);
                            }

                            let mut current = keep.read().next().clone();

                            'inner: loop
                            {
                                let marker = current.read();

                                match &*current.read()
                                {
                                    Some(node) =>
                                    {
                                        let next_child = node.read().next().clone();

                                        if node.read().key() == key
                                        {
                                            let guard = node.read().value();

                                            if current
                                                .exchange(
                                                    &marker,
                                                    next_child.read().as_ref().clone(),
                                                )
                                                .is_err()
                                            {
                                                current = keep.read().next().clone();
                                                continue 'inner;
                                            }

                                            break 'outer Some(guard);
                                        }

                                        current = next_child;
                                    }

                                    None => break 'outer None,
                                }
                            }
                        }

                        // If the head does not have a next node, its key can simply be compared to key,
                        // if the keys match this Entry::Head(..) can simply be swapped for a Entry::Empty.
                        None =>
                        {
                            if keep.read().key() != key
                            {
                                break 'outer None;
                            }

                            let empty_entry = Keep::new(Entry::Empty);
                            if entry.exchange_with(entry_marker, &empty_entry).is_err()
                            {
                                continue;
                            }

                            break 'outer Some(guard);
                        }
                    }
                }
            }
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
