use crate::{
    entry::{Entry, EntryNode},
    resizer::Resizer,
    table::Table,
};
use keep::{Domain, Guard, Heaped, Keep};
use std::hash::{BuildHasher, Hash, RandomState};


//TODO: What is the optimal stride???
const TRANSFER_STRIDE: usize = 5;


pub struct PlugMap<'d, K, V, S = RandomState>
{
    domain: &'d Domain,
    table: Keep<'d, Table<'d, K, V>>,
    resizer: Keep<'d, Option<Resizer<'d, K, V>>>,
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
            resizer: domain.keep(None),
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

        let entry_node = self
            .domain
            .keep(EntryNode::new(key, val, hash, self.domain));

        'load_table: loop
        {
            let table = self.table.read();
            let bin_index = table.bin_index(hash);
            let mut current_entry = table.bin_at(bin_index);

            let result = 'insert: loop
            {
                match &*current_entry
                {
                    Entry::Head(keep) =>
                    {
                        let resizer = self.resizer.read();

                        if let Some(ref resizer) = *resizer
                        {
                            //TODO: resizer.help(...)
                            todo!();
                            continue 'load_table;
                        }

                        let (depth, res) = keep.read().update(entry_node);

                        if res.is_none()
                        {
                            table.inc_entry_count();
                        }

                        // Only check for a resize if the bin that was just updated now contains more then one item.
                        if depth > 0
                        {
                            if table.resize_up_needed()
                            {
                                let old_table = self.table.read();
                                let new_table = self.domain.keep(old_table.new_bigger(self.domain));
                                let new_resizer =
                                    Resizer::new(old_table, new_table, TRANSFER_STRIDE);

                                match self.resizer.exchange(&resizer, Some(new_resizer))
                                {
                                    Ok(_) =>
                                    {
                                        if let Some(ref resizer) = *self.resizer.read()
                                        {
                                            //TODO: resizer.help(...)
                                            todo!()
                                        }

                                        break 'insert res;
                                    }

                                    Err(_) =>
                                    {
                                        // Some other thread already started a resize, while we were starting one...
                                        // Since the other thread is now in charge of the resize, we are all done here...
                                        break 'insert res;
                                    }
                                }
                            }
                        }

                        break 'insert res;
                    }

                    Entry::Empty =>
                    {
                        match table.exchange(
                            bin_index,
                            &current_entry,
                            Entry::Head(self.domain.keep(entry_node.read())),
                        )
                        {
                            Ok(_old) =>
                            {
                                table.inc_entry_count();
                                break 'insert None;
                            }

                            // current_entry became not empty while this insert was happening, update the current_entry and retry...
                            Err(curr) =>
                            {
                                current_entry = curr;
                                continue 'insert;
                            }
                        }
                    }

                    Entry::Moved(new_table) =>
                    {
                        //TODO: Update in new table...
                        todo!()
                    }
                }
            };

            break 'load_table result;
        }
    }

    #[inline]
    fn hash(&self, val: impl Hash) -> u64
    {
        self.hasher.hash_one(val)
    }
}
