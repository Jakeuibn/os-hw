//! Stride scheduling algorithm implementation.
use core::cmp::Ordering;

const BIG_STRIDE: u64 = 1 << 32 - 1;

/// Pass measures the total stride a process has passed. The process with the smallest pass will be scheduled first.
#[derive(Debug, Clone, Copy)]
pub struct Pass(u64);

impl Pass {
    /// Create a new Pass with the given initial value.
    pub fn new(pass: u64) -> Self {
        Pass(pass)
    }
    /// Add stride to the pass according to the priority of the process. The higher the priority, the smaller the stride.
    pub fn add_stride(&mut self, priority: isize) {
        self.0 += BIG_STRIDE / priority as u64;
    }
}

impl PartialOrd for Pass {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0 < other.0 {
            if other.0 - self.0 <= BIG_STRIDE / 2 {
                Some(Ordering::Less)
            } else {
                Some(Ordering::Greater)
            }
        } else if self.0 > other.0 {
            if self.0 - other.0 <= BIG_STRIDE / 2 {
                Some(Ordering::Greater)
            } else {
                Some(Ordering::Less)
            }
        } else {
            Some(Ordering::Equal)
        }
    }
}

impl PartialEq for Pass {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Pass {}

impl Ord for Pass {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}