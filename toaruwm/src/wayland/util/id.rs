use std::sync::atomic::{AtomicU64, Ordering};

/// A counter that returns unique IDs, counting up from 1.
// idea shamelessly stolen from Niri.
#[derive(Debug)]
pub struct IdCounter {
    value: AtomicU64,
}

impl IdCounter {
    pub const fn new() -> Self {
        // Start from 1 to reduce the possibility that some other code that uses these IDs will
        // get confused.
        Self {
            value: AtomicU64::new(1)
        }
    }

    pub fn next(&self) -> u64 {
        self.value.fetch_add(1, Ordering::Relaxed)
    }
}