use std::sync::atomic::{AtomicUsize, Ordering};

use crate::entry::Entry;
use keep::{Domain, Guard, Heaped, Keep};


pub struct Table<'d, K, V>
{
    size: usize,
    capacity: usize,
    entry_count: AtomicUsize,
    entries: Box<[Keep<'d, Entry<'d, K, V>>]>,
}


impl Table<'_, (), ()>
{
    pub const DEFAULT_SIZE: usize = 4;
    pub const MAX_SIZE: usize = 32;
}


impl<'d, K, V> Table<'d, K, V>
{
    pub fn new(mut size: usize, domain: &'d Domain) -> Self
    {
        // Do not allow the size to be too small or too big
        size = size.clamp(Table::DEFAULT_SIZE, Table::MAX_SIZE);

        let mut entries = Box::new_uninit_slice(1 << size);

        entries.iter_mut().for_each(|e| {
            e.write(domain.keep(Entry::Empty));
        });

        Self {
            entry_count: AtomicUsize::new(0),
            capacity: 1 << size,
            size,
            entries: unsafe { entries.assume_init() },
        }
    }


    #[inline]
    /// Returns the length of this Table.
    ///
    /// This is not the capacity or entry count but the number of bins present in this table.
    pub fn length(&self) -> usize
    {
        1 << self.size
    }

    #[inline]
    pub fn inc_entry_count(&self) -> usize
    {
        self.entry_count.fetch_add(1, Ordering::SeqCst) + 1
    }

    #[inline]
    pub fn dec_entry_count(&self) -> usize
    {
        self.entry_count.fetch_sub(1, Ordering::Relaxed) - 1
    }

    #[inline]
    pub fn new_bigger(&self, domain: &'d Domain) -> Self
    {
        Self::new(self.size + 1, domain)
    }

    #[inline]
    pub fn new_smaller(&self, domain: &'d Domain) -> Self
    {
        Self::new(self.size - 1, domain)
    }

    /// Returns `true` if more than 75% of the available capacity is occupied
    #[inline]
    pub fn resize_up_needed(&self) -> bool
    {
        self.entry_count.load(Ordering::SeqCst) > 1 << (self.size - 1) + 1 << (self.size - 2)
            && self.size < Table::MAX_SIZE
    }

    /// Returns `true` if less than 25% of the available capacity is occupied
    #[inline]
    pub fn resize_down_needed(&self) -> bool
    {
        self.entry_count.load(Ordering::SeqCst) < (self.size - 2) && self.size > Table::DEFAULT_SIZE
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
