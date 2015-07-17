//pub const FINGERPRINT_SIZE: usize = 1;
pub const BUCKET_SIZE: usize = 4;

// Fingerprint Size is 1 byte so lets remove the Vec
#[derive(PartialEq, Copy, Clone)]
pub struct Fingerprint (pub u8);

pub struct Bucket (pub Vec<Fingerprint>);

impl Bucket {

    pub fn new() -> Bucket {
        Bucket(Vec::with_capacity(BUCKET_SIZE))
    }

    pub fn insert(&mut self, fp: Fingerprint) -> bool {
        if self.0.len() < BUCKET_SIZE {
            self.0.push(fp);
            true
        } else {
            false
        }
    }

    /// Deletes the given fingerprint from the bucket. Since the order inside
    /// the bucket doesn't matter, we can use `swap_remove` to keep the runtime
    /// in O(1).
    pub fn delete(&mut self, fp: Fingerprint) -> bool {
        let pos = self.0.iter().position(|e| *e == fp);
        match pos {
            Some(index) => { self.0.swap_remove(index); true }
            None => false
        }
    }

    pub fn get_fingerprint_index(&mut self, fp: Fingerprint) -> usize {
        for i in 0..self.0.len() {
            if self.0[i] == fp {
                return i;
            }
        }
        BUCKET_SIZE + 1
    }
}
