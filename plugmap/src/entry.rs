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
    next: Keep<Option<Keep<EntryNode<Key, Val>>>>,
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

    pub fn update(&self, node: &Keep<EntryNode<Key, Val>>) -> Option<Guard<Val>>
    {
        if self.key == node.read().key
        {
            self.val.swap_with(&node.read().val);
            return Some(node.read().value());
        }

        let next = &self.next;
        let mut next_guard = next.read();

        loop
        {
            match &*next_guard
            {
                Some(next) => return next.read().update(node),

                None =>
                {
                    match next.exchange(&next_guard, Some(node.clone()))
                    {
                        Ok(_old) => return None,
                        Err(actual) =>
                        {
                            next_guard = actual;
                        }
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
            Some(next) => next.read().search(key),
            None => None,
        }
    }
}
