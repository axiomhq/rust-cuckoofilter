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

#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]

mod bucket;
mod util;

extern crate rand;
extern crate farmhash;
extern crate byteorder;

use farmhash::{FarmHasher};
use bucket::{Bucket, Fingerprint, BUCKET_SIZE};
use util::{get_fai, get_alt_index, FaI};
use rand::{Rng};
use std::iter::{repeat};
use std::hash::{Hasher, Hash};
use std::marker::{PhantomData};

pub const MAX_REBUCKET: usize = 500;

// A cuckoo filter class exposes a Bloomier filter interface,
// providing methods of add, delete, contains.
pub struct CuckooFilter<H = FarmHasher> {
    buckets: Box<[Bucket]>,
    len: u64,
    _hasher: std::marker::PhantomData<H>,
}

impl CuckooFilter<FarmHasher> {
  /// Constructs a Cockoo Filter with default capacity and hasher.
  pub fn new() -> CuckooFilter<FarmHasher> {
    Self::with_capacity(1000000)
  }
}

impl<H> CuckooFilter<H>
  where H: Hasher + Default
{
    /// Constructs a Cuckoo Filter with a given max capacity
    pub fn with_capacity(cap: u64) -> CuckooFilter<H> {
        let capacity = match cap.next_power_of_two()/BUCKET_SIZE as u64 {
            0 => 1,
            cap => cap,
        };

        CuckooFilter {
            buckets: repeat(Bucket::new())
              .take(capacity as usize)
              .collect::<Vec<_>>()
              .into_boxed_slice(),
            len: 0,
            _hasher: PhantomData
        }
    }

    /// Returns a Cuckoo Filter with a default max Capacity 1000000 items/
    pub fn default() -> CuckooFilter<H> {
        CuckooFilter::with_capacity(1000000)
    }

    /// Checks if `data` is in the filter.
    pub fn contains<T: ?Sized + Hash>(&mut self, data: &T) -> bool {
        let FaI { fp, i1, i2 } = get_fai::<T, H>(data);
        let len = self.buckets.len();
        let b1 = self.buckets[i1%len].get_fingerprint_index(fp);
        let b2 = self.buckets[i2%len].get_fingerprint_index(fp);

        b1.or(b2).is_some()
    }

    /// Adds `data` to the filter. Returns true if the insertion was successful.
    pub fn add<T: ?Sized + Hash>(&mut self, data: &T) -> bool {
        let FaI { fp, i1, i2 } = get_fai::<T, H>(data);
        self.put(fp, i1) || self.put(fp, i2) || self.reinsert(fp, i2)
    }

    /// Adds `data` to the filter if it does not exist in the filter yet.
    /// Returns `true` if `data` was not yet present in the filter and added
    /// successfully.
    pub fn test_and_add(&mut self, data: &[u8]) -> bool {
        if self.contains(data) {
            false
        } else {
          self.add(data)
        }
    }

    /// Number of items in the filter.
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Check if filter is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Deletes `data` from the filter. Returns true if `data` existed in the
    /// filter before.
    pub fn delete<T: ?Sized + Hash>(&mut self, data: &T) -> bool{
        let FaI { fp, i1, i2 } = get_fai::<T, H>(data);

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
            i = get_alt_index::<H>(&mut fp, i);
            if self.put(fp, i) {
                return true;
            }
        }

        false
    }
}
