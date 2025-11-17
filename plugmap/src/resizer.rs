use crate::table::Table;
use keep::*;
use std::sync::atomic::{AtomicUsize, Ordering};


pub struct Resizer<'d, K, V>
{
    original_table: Guard<'d, Table<'d, K, V>>,
    new_table: Keep<'d, Table<'d, K, V>>,
    helpers: AtomicUsize,
    stride: usize,
    next_index: AtomicUsize,
}


impl<'d, K, V> Resizer<'d, K, V>
{
    pub fn new(
        original_table: Guard<'d, Table<'d, K, V>>,
        new_table: Keep<'d, Table<'d, K, V>>,
        stride: usize,
    ) -> Self
    {
        Self {
            original_table,
            new_table,
            helpers: AtomicUsize::new(0),
            stride,
            next_index: AtomicUsize::new(0),
        }
    }


    /// Make this thread help with the resize.
    ///
    /// Blocks until the resize is complete
    pub fn help(&self)
    {
        let table_length = self.original_table.length();

        // Get a section of bins that this thread should transfer to the new table.
        // If the start index is overlapping the end index, the transfer is finished.
        let mut start_index = self.next_index.fetch_add(self.stride, Ordering::SeqCst);
        let mut end_index = (start_index + self.stride).min(table_length);

        while start_index < end_index
        {
            for i in start_index..end_index
            {
                let entry = self.original_table.bin_at(i);
            }

            start_index = self.next_index.fetch_add(self.stride, Ordering::SeqCst);
            end_index = (start_index + self.stride).min(table_length);
        }

        // This helper is finished now, decrease the number of helpers by 1
        let mut helpers = self.helpers.fetch_sub(1, Ordering::SeqCst);

        // Now wait until all helpers are finished...
        while helpers >= 1
        {
            helpers = self.helpers.load(Ordering::Relaxed);
        }
    }
}
