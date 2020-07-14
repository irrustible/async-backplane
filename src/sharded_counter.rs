use std::sync::atomic::{AtomicUsize, Ordering};

const DEFAULT_LEASE: usize = 64;

/// A Sharded Counter 
pub struct ShardedCounter {
    pub lease: usize,
    start: usize,
    end: usize,
}

/// A reference is a source from which we can lease a range. Currently, AtomicUsize
pub trait Reference {
    fn lease(&self, lease: usize) -> usize;
}

impl Reference for AtomicUsize {
    fn lease(&self, lease: usize) -> usize {
        self.fetch_add(lease, Ordering::SeqCst)
    }
}

impl Default for ShardedCounter {
    fn default() -> Self {
        ShardedCounter::new(DEFAULT_LEASE)
    }
}

impl ShardedCounter {
    pub fn new(lease: usize) -> Self {
        ShardedCounter::new_with_values(0, 0, lease)
    }
    fn new_with_values(lease: usize, start: usize, end: usize) -> Self {
        ShardedCounter { lease, start, end }
    }
    /// Attempts to take the current id and replace it with the next one
    fn next_local(&mut self) -> Option<usize> {
        let current = self.start;
        if current < self.end {
            self.start += 1;
            Some(current)
        } else {
            None
        }
    }
    fn next_reference<R: Reference>(&mut self, reference: &R) -> usize {
        let new = reference.lease(self.lease);
        self.start = new + 1;
        self.end = new + self.lease;
        new
    }
    pub fn next(&mut self, reference: &AtomicUsize) -> usize {
        self.next_local().unwrap_or_else(|| self.next_reference(reference))
    }
}
