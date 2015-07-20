#![feature(test)]

extern crate cuckoofilter;
extern crate test;

use self::cuckoofilter::*;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::error::Error;

fn get_words() -> String {
  let path = Path::new("/usr/share/dict/words");
  let display = path.display();

  // Open the path in read-only mode, returns `io::Result<File>`
  let mut file = match File::open(&path) {
    // The `description` method of `io::Error` returns a string that
    // describes the error
    Err(why) => panic!("couldn't open {}: {}", display,
                                               Error::description(&why)),
    Ok(file) => file,
  };

  let mut contents = String::new();
  if let Err(why) = file.read_to_string(&mut contents) {
    panic!("couldn't read {}: {}", display, Error::description(&why));
  }
  contents
}

#[test]
fn test_insertion() {
  let contents = get_words();
  let split: Vec<&str> = contents.split("\n").collect();

  let mut cf = CuckooFilter::new();
  let mut insertions = 0;
  for s in &split {
      if cf.test_and_add(s) {
        insertions += 1;
      }
  }
  assert_eq!(cf.len(), insertions);
  assert_eq!(cf.len(), 234185 as u64);

  for s in &split {
      cf.delete(s);
  }
  assert_eq!(cf.len(), 0);
  assert!(cf.is_empty());
}
