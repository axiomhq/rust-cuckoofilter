//pub const FINGERPRINT_SIZE: usize = 1;
pub const BUCKET_SIZE: usize = 4;

// Fingerprint Size is 1 byte so lets remove the Vec
#[derive(PartialEq, Copy, Clone)]
pub struct Fingerprint (pub u8);

/// Manages BUCKET_SIZE fingerprints at most.
#[derive(Clone)]
pub struct Bucket {
    pub buffer: Vec<Fingerprint>,
}

impl Bucket {
    /// Creates a new bucket with a pre-allocated buffer.
    pub fn new() -> Bucket {
        Bucket {
            buffer: Vec::with_capacity(BUCKET_SIZE)
        }
    }

    /// Inserts the fingerprint into the buffer if the buffer is not full. This
    /// Operation will be O(1), since the buffer was pre-allocated.
    pub fn insert(&mut self, fp: Fingerprint) -> bool {
        if self.buffer.len() < BUCKET_SIZE {
            self.buffer.push(fp);
            true
        } else {
            false
        }
    }

    /// Deletes the given fingerprint from the bucket. Since the order inside
    /// the bucket doesn't matter, we can use `swap_remove` to make the
    /// deletion O(1). Finding element still needs O(n).
    pub fn delete(&mut self, fp: Fingerprint) -> bool {
        match self.get_fingerprint_index(fp) {
            Some(index) => { self.buffer.swap_remove(index); true }
            None => false
        }
    }

    /// Returns the index of the given fingerprint, if its found. O(n)
    pub fn get_fingerprint_index(&mut self, fp: Fingerprint) -> Option<usize> {
        self.buffer.iter().position(|e| *e == fp)
    }
}
