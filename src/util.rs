extern crate farmhash;
extern crate byteorder;

use std::io::Cursor;
use self::byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};
use bucket::{Fingerprint};

pub fn get_hash(data: &[u8]) -> Vec<u8> {
    let hash32 = farmhash::hash32(data);
    let mut hash = vec![];
    hash.write_u32::<BigEndian>(hash32).unwrap();
    hash
}

pub fn get_alt_index(fp: &Fingerprint, i: usize) -> usize {
    let hash = get_hash(&[fp.0]);
    let mut rdr = Cursor::new(hash);
    let alt_i = rdr.read_u32::<BigEndian>().unwrap() as usize;
    (i ^ alt_i) as usize
}

pub struct FaI {
    pub fp: Fingerprint,
    pub i1: usize,
    pub i2: usize
}

pub fn get_fai(data: &[u8]) -> FaI {
    let hash = get_hash(data);
    let f = Fingerprint(hash[0]);
    let mut rdr = Cursor::new(hash);
    let i1 = rdr.read_u32::<BigEndian>().unwrap() as usize;
    let i2 = get_alt_index(&f, i1);
    FaI {
        fp: f, i1: i1, i2: i2
    }
}

#[test]
fn test_fp_and_index() {
    let data = &"seif".as_bytes();
    let fai = get_fai(data);
    let fp = &fai.fp;
    let i1 = fai.i1;
    let i2 = fai.i2;
    let i11 = get_alt_index(fp, i2);
    assert_eq!(i11, i1);

    let i22 = get_alt_index(fp, i11);
    assert_eq!(i22, i2);
}
