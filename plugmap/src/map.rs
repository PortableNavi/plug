use crate::{entry::EntryNode, table::Table};
use keep::*;
use std::hash::{BuildHasher, Hash, RandomState};


pub struct PlugMap<Key, Val, S = RandomState>
{
    table: Keep<Table<Key, Val>>,
    hasher: S,
}


impl<Key, Val, S> PlugMap<Key, Val, S>
{
    pub const DEFAULT_SIZE: usize = 4;
}


impl<Key, Val, S> PlugMap<Key, Val, S>
where
    Key: Hash + Eq,
    S: BuildHasher,
{
    /// Creates a new PlugMap with a capacity of `2^size` and a `BuildHasher` provided by the caller.
    pub fn new_with_hasher(size: usize, hasher: S) -> Self
    {
        Self {
            table: Keep::new(Table::new(size)),
            hasher,
        }
    }

    /// Tries to remove an entry from the map.
    pub fn remove(&self, key: &Key) -> Option<Guard<Val>>
    where
        Val: std::fmt::Debug,
    {
        self.table.read().remove(key, self.hash(key))
    }

    /// Inserts a new key-value pair into the map or updates an existing one...
    pub fn insert(&self, key: Key, val: Val) -> Option<Guard<Val>>
    {
        let hash = self.hash(&key);
        self.table.read().insert(EntryNode::new(key, val, hash))
    }

    /// Tries to get a value associated with `key`. Returns `None` if no such value exists.
    pub fn get(&self, key: &Key) -> Option<Guard<Val>>
    {
        self.table.read().get(key, self.hash(key))
    }

    #[inline]
    fn hash(&self, val: impl Hash) -> u64
    {
        self.hasher.hash_one(val)
    }
}


impl<Key, Val> PlugMap<Key, Val, RandomState>
where
    Key: Hash + Eq,
{
    pub fn new() -> Self
    {
        Self::new_with_hasher(Self::DEFAULT_SIZE, RandomState::new())
    }
}


impl<Key, Val> Default for PlugMap<Key, Val, RandomState>
where
    Key: Hash + Eq,
{
    fn default() -> Self
    {
        Self::new()
    }
}
