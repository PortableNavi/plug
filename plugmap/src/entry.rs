use keep::*;


pub enum Entry<Key, Val>
{
    Empty,
    Head(Keep<EntryNode<Key, Val>>),
}


impl<Key, Val> Entry<Key, Val>
where
    Key: Eq,
{
    pub fn search(&self, key: &Key) -> Option<Guard<Val>>
    {
        match self
        {
            Entry::Empty => None,
            Entry::Head(keep) => keep.read().search(key),
        }
    }
}


pub struct EntryNode<Key, Val>
{
    val: Keep<Val>,
    key: Key,
    hash: u64,
    next: Keep<Option<Guard<EntryNode<Key, Val>>>>,
}


impl<Key, Val> EntryNode<Key, Val>
where
    Key: Eq,
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

    pub fn update(&self, node: Guard<EntryNode<Key, Val>>) -> Option<Guard<Val>>
    {
        if self.key == node.read().key
        {
            let guard = node.read().val.read();
            self.val.swap_guard(&guard);
            return Some(guard);
        }

        let next = self.next.read();

        loop
        {
            match &*next
            {
                Some(next) => return next.update(node),

                None =>
                {
                    if next.exchange(Some(node.clone())).is_ok()
                    {
                        next.reload();
                        return None;
                    }
                }
            }
        }
    }

    pub fn search(&self, key: &Key) -> Option<Guard<Val>>
    {
        if &self.key == key
        {
            return Some(self.value());
        }

        match &*self.next.read()
        {
            Some(next) => next.search(key),
            None => None,
        }
    }
}
