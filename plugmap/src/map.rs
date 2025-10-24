use crate::{
    entry::{Entry, EntryNode},
    table::Table,
};
use keep::{Domain, Guard, Heaped, Keep};
use std::hash::{BuildHasher, Hash, RandomState};


pub struct PlugMap<'d, K, V, S = RandomState>
{
    domain: &'d Domain,
    table: Keep<'d, Table<'d, K, V>>,
    hasher: S,
}


impl<'d, K, V> Default for PlugMap<'d, K, V, RandomState>
where
    V: 'd,
    K: Hash + Eq + 'd,
{
    fn default() -> Self
    {
        Self::new()
    }
}


impl<'d, K, V> PlugMap<'d, K, V, RandomState>
where
    V: 'd,
    K: Hash + Eq + 'd,
{
    pub fn new() -> Self
    {
        let domain = unsafe { &*Box::into_raw(Box::new(Domain::new())) };
        Self::new_with(domain, RandomState::new())
    }
}


impl<'d, K, V, S> PlugMap<'d, K, V, S>
where
    V: 'd,
    K: Hash + Eq + 'd,
    S: BuildHasher,
{
    pub fn new_with(domain: &'d Domain, hasher: S) -> Self
    {
        Self {
            domain,
            table: domain.keep(Table::new(Table::DEFAULT_SIZE, domain)),
            hasher,
        }
    }

    pub fn get(&self, key: &K) -> Option<Guard<'d, V>>
    {
        let hash = self.hash(key);
        self.table.read().bin(hash).find(key).map(|g| g.value())
    }

    pub fn insert<'h>(&self, key: K, val: impl Heaped<'h, V>) -> Option<Guard<'d, V>>
    where
        V: 'h,
    {
        let hash = self.hash(&key);
        let table = self.table.read();
        let bin_index = table.bin_index(hash);

        let entry_node = self
            .domain
            .keep(EntryNode::new(key, val, hash, self.domain));

        let mut current_entry = table.bin_at(bin_index);

        'insert: loop
        {
            match &*current_entry
            {
                Entry::Head(keep) => break 'insert keep.read().update(entry_node),

                Entry::Empty =>
                {
                    match table.exchange(
                        bin_index,
                        &current_entry,
                        Entry::Head(self.domain.keep(entry_node.read())),
                    )
                    {
                        Ok(old) => break 'insert None,

                        // current_entry became not empty while this insert was happening, update the current_entry and retry...
                        Err(curr) =>
                        {
                            current_entry = curr;
                            continue 'insert;
                        }
                    }
                }
            }

            break None;
        }
    }

    #[inline]
    fn hash(&self, val: impl Hash) -> u64
    {
        self.hasher.hash_one(val)
    }
}
