use keep::*;

pub enum Entry<Key, Val>
{
    Empty,
    Head(Keep<EntryNode<Key, Val>>),
}


pub struct EntryNode<Key, Val>
{
    val: Keep<Val>,
    key: Key,
    hash: u64,
    next: Keep<Option<EntryNode<Key, Val>>>,
}


impl<Key, Val> EntryNode<Key, Val>
{
    #[inline]
    pub fn value(&self) -> Guard<Val>
    {
        self.val.read()
    }

    #[inline]
    pub fn hash(&self) -> u64
    {
        self.hash
    }

    pub fn new(key: Key, val: Val, hash: u64) -> Self
    {
        Self {
            val: Keep::new(val),
            key,
            hash,
            next: Keep::new(None),
        }
    }
}
