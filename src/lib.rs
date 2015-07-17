mod bucket;
mod util;

extern crate rand;

use bucket::{Bucket, Fingerprint, BUCKET_SIZE};
use util::{get_next_pow_2, get_fai, get_alt_index};
use rand::Rng;

pub const MAX_REBUCKET: usize = 500;

pub struct CuckooFilter {
    buckets: Vec<Bucket>,
    count: u64,
}

impl CuckooFilter {
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

    pub fn default() -> CuckooFilter {
        return CuckooFilter::new(1000000);
    }

    pub fn lookup(&mut self, data: &[u8]) -> bool {
        let fai = get_fai(data);
        let fp = fai.fp;
        let i1 = fai.i1;
        let i2 = fai.i2;
        let len = self.buckets.len();
        let b1 = self.buckets[i1%len].get_fingerprint_index(fp);
        let b2 = self.buckets[i2%len].get_fingerprint_index(fp);
        return b1 < BUCKET_SIZE || b2 < BUCKET_SIZE;
    }

    pub fn insert(&mut self, data: &[u8]) -> bool {
        let fai = get_fai(data);
        let fp = fai.fp;
        let i1 = fai.i1;
        let i2 = fai.i2;
        if self.put(fp, i1) || self.put(fp, i2) {
            return true;
        }
        return self.reinsert(fp, i2)
    }

    pub fn insert_unique(&mut self, data: &[u8]) -> bool {
        if self.lookup(data) {
            return false;
        }
        return self.insert(data);
    }

    pub fn get_count(&mut self) -> u64 {
        return self.count;
    }

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