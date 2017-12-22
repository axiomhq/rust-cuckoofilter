extern crate cuckoofilter;
use cuckoofilter::CuckooFilter;
use std::collections::hash_map::DefaultHasher;

#[test]
fn interoperability() {
    let total_items = 1_000_000;

    let mut filter = CuckooFilter::<DefaultHasher>::with_capacity(total_items);

    let mut num_inserted: u64 = 0;
    // Fit as many values in as possible, count how many made it in.
    for i in 0..total_items {
        match filter.add(&i) {
            Ok(_) => num_inserted += 1,
            Err(_) => break,
        }
    }

    // Export the fingerprint data stored in the filter,
    // along with the filter's current length.
    let store: Vec<u8> = filter.export();
    let length = filter.len();

    // Create a new filter using the `recover` method and the values previously exported.
    let recovered_filter = CuckooFilter::<DefaultHasher>::recover(store, length);

    // The range 0..num_inserted are all known to be in the filter.
    // The filters shouldn't return false negatives, and therefore they should all be contained.
    // Both filters should also be identical.
    for i in 0..num_inserted {
        assert!(filter.contains(&i));
        assert!(recovered_filter.contains(&i));
    }
}
