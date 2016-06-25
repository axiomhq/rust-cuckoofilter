use std::hash::{Hasher, Hash, hash};
use ::byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};
use ::bucket::{Fingerprint, FINGERPRINT_SIZE};

pub struct FaI {
    pub fp: Fingerprint,
    pub i1: usize,
    pub i2: usize,
}

fn get_hash<T: ?Sized + Hash, H: Hasher + Default>(data: &T) -> [u8; 4] {
    let mut result = [0; 4];
    {
        let mut hasher = <H as Default>::default();
        data.hash(&mut hasher);
        let _ = (&mut result[..]).write_u32::<BigEndian>(hasher.finish() as u32);
    }
    result
}

pub fn get_alt_index<H: Hasher + Default>(fp: Fingerprint, i: usize) -> usize {
    let hash = get_hash::<_, H>(&fp.data);
    let alt_i = (&hash[..]).read_u32::<BigEndian>().unwrap() as usize;
    (i ^ alt_i) as usize
}

impl FaI {
    fn from_data<T: ?Sized + Hash, H: Hasher + Default>(data: &T) -> FaI {
        let mut hash_arr: [u8; FINGERPRINT_SIZE] = [0; FINGERPRINT_SIZE];
        let hash = get_hash::<_, H>(data);
        let mut n = 0;
        let fp;

        loop {
            for i in 0..FINGERPRINT_SIZE {
                hash_arr[i] = hash[i] + n;
            }

            if let Some(val) = Fingerprint::from_data(hash_arr) {
                fp = val;
                break;
            }
            n += 1;
        }

        let i1 = (&hash[..]).read_u32::<BigEndian>().unwrap() as usize;
        let i2 = get_alt_index::<H>(fp, i1);
        FaI {
            fp: fp,
            i1: i1,
            i2: i2,
        }
    }

    pub fn random_index<R: ::rand::Rng>(&self, r: &mut R) -> usize {
        if r.gen() {
            self.i1
        } else {
            self.i2
        }
    }
}

pub fn get_fai<T: ?Sized + Hash, H: Hasher + Default>(data: &T) -> FaI {
    FaI::from_data::<_, H>(data)
}

#[test]
fn test_fp_and_index() {
    use std::hash::SipHasher;
    let data = "seif";
    let fai = get_fai::<_, SipHasher>(data);
    let fp = fai.fp;
    let i1 = fai.i1;
    let i2 = fai.i2;
    let i11 = get_alt_index::<SipHasher>(fp, i2);
    assert_eq!(i11, i1);

    let i22 = get_alt_index::<SipHasher>(fp, i11);
    assert_eq!(i22, i2);
}
