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
use util::{get_next_pow_2, get_fai, get_alt_index};
use rand::Rng;

pub const MAX_REBUCKET: usize = 500;

// A cuckoo filter class exposes a Bloomier filter interface,
// providing methods of add, delete, contains.
pub struct CuckooFilter {
    buckets: Vec<Bucket>,
    count: u64,
}

impl CuckooFilter {
    /**
     * Returns a Cuckoo Filter with a given max Capacity
     */
    pub fn new(cap: u64) -> CuckooFilter {
        let mut capacity = get_next_pow_2(cap);
        capacity = capacity/BUCKET_SIZE as u64;
        if capacity == 0 {
            capacity = 1;
        }
        let mut buckets: Vec<Bucket> = vec![];
        for _ in 0..capacity {
            buckets.push(Bucket::new())
        }
        return CuckooFilter{buckets: buckets, count: 0,}
    }

    /**
     * Returns a Cuckoo Filter with a default max Capacity 1000000 items/
     */
    pub fn default() -> CuckooFilter {
        return CuckooFilter::new(1000000);
    }

    /**
     * Returns if data is in filter.
     */
    pub fn contains(&mut self, data: &[u8]) -> bool {
        let fai = get_fai(data);
        let fp = fai.fp;
        let i1 = fai.i1;
        let i2 = fai.i2;
        let len = self.buckets.len();
        let b1 = self.buckets[i1%len].get_fingerprint_index(fp);
        let b2 = self.buckets[i2%len].get_fingerprint_index(fp);
        return b1 < BUCKET_SIZE || b2 < BUCKET_SIZE;
    }

    /**
     * Add data to the filter.
     * Returns true if successful.
     */
    pub fn add(&mut self, data: &[u8]) -> bool {
        let fai = get_fai(data);
        let fp = fai.fp;
        let i1 = fai.i1;
        let i2 = fai.i2;
        if self.put(fp, i1) || self.put(fp, i2) {
            return true;
        }
        return self.reinsert(fp, i2)
    }
    /**
     * If data is in filter and add an item to the filter if not exists.
     * This is like using lookup and adding if it returns true.
     * Returns true if data did not exist in filter an is added successfuly
     */
    pub fn test_and_add(&mut self, data: &[u8]) -> bool {
        if self.contains(data) {
            return false;
        }
        return self.add(data);
    }

    /**
     * Returns number of current inserted items;
     */
    pub fn get_count(&mut self) -> u64 {
        return self.count;
    }

    /**
     * Delete an data from the filter.
     * Returns true if successful (data exists in filter and was deleted).
     */
    pub fn delete(&mut self, data: &[u8]) -> bool{
        let fai = get_fai(data);
        let fp = fai.fp;
        let i1 = fai.i1;
        let i2 = fai.i2;

        return self.remove(fp, i1) || self.remove(fp, i2)
    }

    fn remove (&mut self, fp: Fingerprint, i: usize) -> bool {
        let len = self.buckets.len();
        if self.buckets[i%len].delete(fp) {
            self.count -= 1;
            return true;
        }
        return false;
    }

    fn put(&mut self, fp: Fingerprint, i: usize) -> bool {
        let len = self.buckets.len();
        let mut b = &mut self.buckets[i%len];
        if b.insert(fp) {
            self.count+=1;
            return true;
        }
        return false;
    }

    fn reinsert(&mut self, mut fp: Fingerprint, mut i: usize) -> bool {
        for _ in 0..MAX_REBUCKET {
            let j = rand::thread_rng().gen_range(0.0, BUCKET_SIZE as f64) as usize;
            let newfp = fp;
            let len = self.buckets.len();
            fp = self.buckets[i%len].0[j];
            self.buckets[i%len].0[j] = newfp;
            i = get_alt_index(&mut fp, i);
            if self.put(fp, i) {
                return true;
            }
        }
        return false;
    }
}