//! Cuckoo filter probabilistic data structure for membership testing and cardinlaity counting.
//!
//! # Usage
//!
//! This crate is [on crates.io](https://crates.io/crates/cuckoofilter) and can be
//! used by adding `cuckoofilter` to the dependencies in your project's `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! cuckoofilter = "0.1"
//! ```
//!
//! And this in your crate root:
//!
//! ```rust
//! extern crate cuckoofilter;
//! ```

mod bucket;
mod util;

extern crate rand;

use bucket::{Bucket, Fingerprint, BUCKET_SIZE};
use util::{get_fai, get_alt_index, FaI};
use rand::Rng;
use std::iter::repeat;

pub const MAX_REBUCKET: usize = 500;

// A cuckoo filter class exposes a Bloomier filter interface,
// providing methods of add, delete, contains.
pub struct CuckooFilter {
    buckets: Vec<Bucket>,
    len: u64,
}

impl CuckooFilter {
    /// Constructs a Cockoo Filter with default capacity
    pub fn new() -> CuckooFilter {
        Self::default()
    }

    /// Constructs a Cuckoo Filter with a given max capacity
    pub fn with_capacity(cap: u64) -> CuckooFilter {
        let capacity = match cap.next_power_of_two()/BUCKET_SIZE as u64 {
            0 => 1,
            cap => cap,
        };

        CuckooFilter {
            buckets: repeat(Bucket::new()).take(capacity as usize).collect(),
            len: 0,
        }
    }

    /// Returns a Cuckoo Filter with a default max Capacity 1000000 items/
    pub fn default() -> CuckooFilter {
        CuckooFilter::with_capacity(1000000)
    }

    /// Checks if `data` is in the filter.
    pub fn contains(&mut self, data: &[u8]) -> bool {
        let FaI { fp, i1, i2 } = get_fai(data);
        let len = self.buckets.len();
        let b1 = self.buckets[i1%len].get_fingerprint_index(fp);
        let b2 = self.buckets[i2%len].get_fingerprint_index(fp);

        b1.or(b2).is_some()
    }

    /// Adds `data` to the filter. Returns true if the insertion was successful.
    pub fn add(&mut self, data: &[u8]) -> bool {
        let FaI { fp, i1, i2 } = get_fai(data);
        if self.put(fp, i1) || self.put(fp, i2) {
            return true;
        }
        return self.reinsert(fp, i2)
    }

    /// Adds `data` to the filter if it does not exist in the filter yet.
    /// Returns `true` if `data` was not yet present in the filter and added
    /// successfully.
    pub fn test_and_add(&mut self, data: &[u8]) -> bool {
        if self.contains(data) {
            return false;
        }
        return self.add(data);
    }

    /// Number of items in the filter.
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Deletes `data` from the filter. Returns true if `data` existed in the
    /// filter before.
    pub fn delete(&mut self, data: &[u8]) -> bool{
        let FaI { fp, i1, i2 } = get_fai(data);

        self.remove(fp, i1) || self.remove(fp, i2)
    }

    fn remove(&mut self, fp: Fingerprint, i: usize) -> bool {
        let len = self.buckets.len();
        if self.buckets[i%len].delete(fp) {
            self.len -= 1;
            true
        } else {
            false
        }
    }

    fn put(&mut self, fp: Fingerprint, i: usize) -> bool {
        let len = self.buckets.len();
        if self.buckets[i%len].insert(fp) {
            self.len += 1;
            true
        } else {
            false
        }
    }

    fn reinsert(&mut self, mut fp: Fingerprint, mut i: usize) -> bool {
        for _ in 0..MAX_REBUCKET {
            let j = rand::thread_rng().gen_range(0, BUCKET_SIZE);
            let newfp = fp;
            let len = self.buckets.len();
            fp = self.buckets[i%len].buffer[j];
            self.buckets[i%len].buffer[j] = newfp;
            i = get_alt_index(&mut fp, i);
            if self.put(fp, i) {
                return true;
            }
        }

        false
    }
}
