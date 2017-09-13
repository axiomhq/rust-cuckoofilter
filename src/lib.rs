//! Cuckoo filter probabilistic data structure for membership testing and cardinality counting.
//!
//! # Usage
//!
//! This crate is [on crates.io](https://crates.io/crates/cuckoofilter) and can be
//! used by adding `cuckoofilter` to the dependencies in your project's `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! cuckoofilter = "0.3"
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
extern crate byteorder;

use bucket::{Bucket, Fingerprint, BUCKET_SIZE};
use util::{get_fai, get_alt_index, FaI};
use rand::Rng;
use std::iter::repeat;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hasher, Hash};
use std::marker::PhantomData;
use std::mem;
use std::fmt;
use std::error::Error as StdError;

/// If insertion fails, we will retry this many times.
pub const MAX_REBUCKET: u32 = 500;

/// The default number of buckets.
pub const DEFAULT_CAPACITY: u64 = (1 << 20) - 1;

#[derive(Debug)]
pub enum CuckooError {
    NotEnoughSpace
}

impl fmt::Display for CuckooError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("NotEnoughSpace")
    }
}

impl StdError  for CuckooError {
    fn description(&self) -> &str {
        "Not enough space to store this item, rebucketing failed."
    }
}

/// A cuckoo filter class exposes a Bloomier filter interface,
/// providing methods of add, delete, contains.
///
/// # Examples
///
/// ```
/// extern crate cuckoofilter;
///
/// let words = vec!["foo", "bar", "xylophone", "milagro"];
/// let mut cf = cuckoofilter::CuckooFilter::new();
///
/// let mut insertions = 0;
/// for s in &words {
///     if cf.test_and_add(s).unwrap() {
///         insertions += 1;
///     }
/// }
///
/// assert_eq!(insertions, words.len());
/// assert_eq!(cf.len(), words.len());
///
/// // Re-add the first element.
/// cf.add(words[0]);
///
/// assert_eq!(cf.len(), words.len() + 1);
///
/// for s in &words {
///     cf.delete(s);
/// }
///
/// assert_eq!(cf.len(), 1);
/// assert!(!cf.is_empty());
///
/// cf.delete(words[0]);
///
/// assert_eq!(cf.len(), 0);
/// assert!(cf.is_empty());
///
/// ```
pub struct CuckooFilter<H> {
    buckets: Box<[Bucket]>,
    len: usize,
    _hasher: std::marker::PhantomData<H>,
}

impl Default for CuckooFilter<DefaultHasher> {
    fn default() -> Self {
        CuckooFilter::new()
    }
}

impl CuckooFilter<DefaultHasher> {
    /// Construct a CuckooFilter with default capacity and hasher.
    pub fn new() -> CuckooFilter<DefaultHasher> {
        Self::with_capacity(DEFAULT_CAPACITY)
    }
}

impl<H> CuckooFilter<H>
    where H: Hasher + Default
{
    /// Constructs a Cuckoo Filter with a given max capacity
    pub fn with_capacity(cap: u64) -> CuckooFilter<H> {
        let capacity = match cap.next_power_of_two() / BUCKET_SIZE as u64 {
            0 => 1,
            cap => cap,
        };

        CuckooFilter {
            buckets: repeat(Bucket::new())
                .take(capacity as usize)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            len: 0,
            _hasher: PhantomData,
        }
    }

    /// Checks if `data` is in the filter.
    pub fn contains<T: ?Sized + Hash>(&self, data: &T) -> bool {
        let FaI { fp, i1, i2 } = get_fai::<T, H>(data);
        let len = self.buckets.len();
        self.buckets[i1 % len]
            .get_fingerprint_index(fp)
            .or(self.buckets[i2 % len].get_fingerprint_index(fp))
            .is_some()
    }

    /// Adds `data` to the filter. Returns `Ok` if the insertion was successful,
    /// but could fail with a `NotEnoughSpace` error, especially when the filter
    /// is nearing its capacity.
    /// Note that while you can put any hashable type in the same filter, beware
    /// for side effects like that the same number can have diferent hashes
    /// depending on the type.
    /// So for the filter, 4711i64 isn't the same as 4711u64.
    pub fn add<T: ?Sized + Hash>(&mut self, data: &T) -> Result<(), CuckooError> {
        let fai = get_fai::<T, H>(data);
        if self.put(fai.fp, fai.i1) || self.put(fai.fp, fai.i2) {
            return Ok(());
        }
        let len = self.buckets.len();
        let mut rng = rand::thread_rng();
        let mut i = fai.random_index(&mut rng);
        let mut fp = fai.fp;
        for _ in 0..MAX_REBUCKET {
            let other_fp;
            {
                let loc = &mut self.buckets[i % len].buffer[rng.gen_range(0, BUCKET_SIZE)];
                other_fp = *loc;
                *loc = fp;
                i = get_alt_index::<H>(other_fp, i);
            }
            if self.put(other_fp, i) {
                return Ok(());
            }
            fp = other_fp;
        }
        Err(CuckooError::NotEnoughSpace)
    }

    /// Adds `data` to the filter if it does not exist in the filter yet.
    /// Returns `Ok(true)` if `data` was not yet present in the filter and added
    /// successfully.
    pub fn test_and_add<T: ?Sized + Hash>(&mut self, data: &T) -> Result<bool, CuckooError> {
        if self.contains(data) {
            Ok(false)
        } else {
            match self.add(data) {
                Ok(_) => Ok(true),
                Err(e) => Err(e),
            }
        }
    }

    /// Number of items in the filter.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Number of bytes the filter occupies in memory
    pub fn memory_usage(&self) -> usize {
        mem::size_of_val(self) + self.buckets.len() * mem::size_of::<Bucket>()
    }

    /// Check if filter is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Deletes `data` from the filter. Returns true if `data` existed in the
    /// filter before.
    pub fn delete<T: ?Sized + Hash>(&mut self, data: &T) -> bool {
        let FaI { fp, i1, i2 } = get_fai::<T, H>(data);
        self.remove(fp, i1) || self.remove(fp, i2)
    }

    /// Removes the item with the given fingerprint from the bucket indexed by i.
    fn remove(&mut self, fp: Fingerprint, i: usize) -> bool {
        let len = self.buckets.len();
        if self.buckets[i % len].delete(fp) {
            self.len -= 1;
            true
        } else {
            false
        }
    }

    fn put(&mut self, fp: Fingerprint, i: usize) -> bool {
        let len = self.buckets.len();
        if self.buckets[i % len].insert(fp) {
            self.len += 1;
            true
        } else {
            false
        }
    }
}
