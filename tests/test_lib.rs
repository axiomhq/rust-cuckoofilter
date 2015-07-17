extern crate cuckoofilter;

use self::cuckoofilter::*;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::error::Error;

#[test]
fn test_insertion() {
    let path = Path::new("/usr/share/dict/web2");
    let display = path.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(&path) {
        // The `description` method of `io::Error` returns a string that
        // describes the error
        Err(why) => panic!("couldn't open {}: {}", display,
                                                   Error::description(&why)),
        Ok(file) => file,
    };

    let mut web2 = String::new();
    match file.read_to_string(&mut web2) {
        Err(why) => panic!("couldn't read {}: {}", display,
                                                   Error::description(&why)),
        Ok(_) => {},
    }

    let mut split: Vec<&str> = web2.split("\n").collect();

    let mut cf = CuckooFilter::new(1000000);
    for s in &mut split {
        cf.insert_unique(&s.as_bytes());
    }
    assert_eq!(cf.get_count(), 235033);

    for s in &mut split {
        cf.delete(&s.as_bytes());
    }
    assert_eq!(cf.get_count(), 0);
}