use crate::entry::Entry;
use keep::{Domain, Guard, Heaped, Keep};


pub struct Table<'d, K, V>
{
    capacity: usize,
    entries: Box<[Keep<'d, Entry<'d, K, V>>]>,
}


impl Table<'_, (), ()>
{
    pub const DEFAULT_SIZE: usize = 4;
}


impl<'d, K, V> Table<'d, K, V>
{
    pub fn new(size: usize, domain: &'d Domain) -> Self
    {
        let mut entries = Box::new_uninit_slice(1 << size);

        entries.iter_mut().for_each(|e| {
            e.write(domain.keep(Entry::Empty));
        });

        Self {
            capacity: 1 << size,
            entries: unsafe { entries.assume_init() },
        }
    }

    #[inline]
    pub fn bin_index(&self, hash: u64) -> usize
    {
        hash as usize & (self.capacity - 1)
    }

    #[inline]
    pub fn bin(&self, hash: u64) -> Guard<'d, Entry<'d, K, V>>
    {
        self.entries[self.bin_index(hash)].read()
    }

    #[inline]
    pub fn bin_at(&self, index: usize) -> Guard<'d, Entry<'d, K, V>>
    {
        self.entries[index].read()
    }

    #[allow(clippy::type_complexity)]
    #[inline]
    pub fn exchange<'h>(
        &self,
        index: usize,
        old: &Entry<'d, K, V>,
        new: impl Heaped<'h, Entry<'d, K, V>>,
    ) -> Result<Guard<'_, Entry<'d, K, V>>, Guard<'d, Entry<'d, K, V>>>
    where
        K: 'h,
        V: 'h,
        'd: 'h,
    {
        self.entries[index].exchange(old, new)
    }
}
