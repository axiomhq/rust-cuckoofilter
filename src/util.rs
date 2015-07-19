use std::hash::{Hasher, Hash};
use ::bucket::{Fingerprint};

pub struct FaI {
    pub fp: Fingerprint,
    pub i1: usize,
    pub i2: usize
}

pub fn get_alt_index<H: Hasher + Default>(fp: Fingerprint, index: usize) -> usize {
  let mut hasher = <H as Default>::default();
  fp.hash(&mut hasher);
  hasher.finish() as usize ^ index
}

impl FaI {
  fn from_data<T: ?Sized + Hash, H: Hasher + Default>(data: &T) -> FaI {
    let i1;
    let fp;
    let mut n = 1;
    loop {
      let mut hasher = <H as Default>::default();
      for _ in 0..n {
        data.hash(&mut hasher);
      }
      let hash = hasher.finish() as usize;
      if let Some(val) = Fingerprint::from_usize(hash) {
        i1 = hash;
        fp = val;
        break;
      }
      n += 1;
    }
    let i2 = get_alt_index::<H>(fp, i1);

    FaI { fp: fp, i1: i1, i2: i2 }
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
