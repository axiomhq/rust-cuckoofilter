//pub const FINGERPRINT_SIZE: usize = 1;
pub const BUCKET_SIZE: usize = 4;

// Fingerprint Size is 1 byte so lets remove the Vec
#[derive(PartialEq, Copy, Clone)]
pub struct Fingerprint (pub u8);

pub struct Bucket (pub Vec<Fingerprint>);

impl Bucket {

    pub fn new() -> Bucket {
        return Bucket(vec![]);
    }

    pub fn insert(&mut self, fp: Fingerprint) -> bool {
        if self.0.len() < BUCKET_SIZE {
            self.0.push(fp);
            return true;
        } 
        return false;
    }

    pub fn delete(&mut self, fp: Fingerprint) -> bool {
        for i in 0..self.0.len() {
            if self.0[i] == fp {
                self.0.remove(i);
                return true;
            }
        } 
        return false;
    }

    pub fn get_fingerprint_index(&mut self, fp: Fingerprint) -> usize {
        for i in 0..self.0.len() {
            if self.0[i] == fp {
                return i;
            }
        } 
        return BUCKET_SIZE + 1;
    }
}