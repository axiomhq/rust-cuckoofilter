use std::hash::{Hasher, Hash, hash};
use ::byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};
use ::bucket::{Fingerprint};

fn get_hash<T: ?Sized + Hash, H: Hasher + Default>(data: &T) -> [u8; 4] {
  let mut result = [0; 4];
  {
    let mut hasher = <H as Default>::default();
    data.hash(&mut hasher);
    let _ = (&mut result[..]).write_u32::<BigEndian>(hasher.finish() as u32);
  }
  result
}

pub fn get_alt_index<H: Hasher + Default>(fp: &Fingerprint, i: usize) -> usize {
    let hash = get_hash::<_, H>(&[fp.0]);
    let alt_i = (&hash[..]).read_u32::<BigEndian>().unwrap() as usize;
    (i ^ alt_i) as usize
}

pub struct FaI {
    pub fp: Fingerprint,
    pub i1: usize,
    pub i2: usize
}

pub fn get_fai<T: ?Sized + Hash, H: Hasher + Default>(data: &T) -> FaI {
    let hash = get_hash::<_, H>(data);
    let f = Fingerprint(hash[0]);
    let i1 = (&hash[..]).read_u32::<BigEndian>().unwrap() as usize;
    let i2 = get_alt_index::<H>(&f, i1);
    FaI {
        fp: f, i1: i1, i2: i2
    }
}

#[test]
fn test_fp_and_index() {
    let data = &"seif".as_bytes();
    let fai = get_fai::<_, ::farmhash::FarmHasher>(data);
    let fp = &fai.fp;
    let i1 = fai.i1;
    let i2 = fai.i2;
    let i11 = get_alt_index::<::farmhash::FarmHasher>(fp, i2);
    assert_eq!(i11, i1);

    let i22 = get_alt_index::<::farmhash::FarmHasher>(fp, i11);
    assert_eq!(i22, i2);
}
