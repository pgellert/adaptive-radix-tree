extern crate adaptive_radix_tree;

use adaptive_radix_tree::u64_art_map::*;
use rand::distributions::Standard;
use rand::prelude::Distribution;
use rand::Rng;
use std::collections::BTreeMap;

#[test]
fn test_search_works() {
    let mut artmap = U64ArtMap::<String>::new();
    let result = artmap.get_mut(&10);
    assert_eq!(None, result);
}

#[test]
fn test_insert_works() {
    let mut artmap = U64ArtMap::<String>::new();
    let replaced = artmap.insert(17, String::from("Hello"));
    assert_eq!(None, replaced);

    let min_val = artmap.minimum();
    assert!(min_val.is_some());

    let max_val = artmap.maximum();
    assert!(max_val.is_some());
}

#[test]
fn test_delete_works() {
    let mut artmap = U64ArtMap::<String>::new();
    let replaced = artmap.insert(17, String::from("Hello"));
    assert_eq!(None, replaced);

    let min_val = artmap.minimum();
    assert!(min_val.is_some());

    let removed = artmap.delete(17);
    assert!(removed.is_some());

    let min_val = artmap.minimum();
    assert!(min_val.is_none());
}

#[test]
fn test_minmax_works() {
    let mut artmap = U64ArtMap::<String>::new();
    artmap.insert(100, "min".to_string());
    artmap.insert(200, "middle".to_string());
    artmap.insert(300, "middle".to_string());
    artmap.insert(400, "max".to_string());

    assert_eq!(artmap.minimum().unwrap().0, 100);
    assert_eq!(artmap.maximum().unwrap().0, 400);
}

#[test]
fn test_pop_first_and_pop_last_work() {
    let mut artmap = U64ArtMap::<String>::new();
    artmap.insert(100, "min".to_string());
    artmap.insert(200, "middle".to_string());
    artmap.insert(300, "middle".to_string());
    artmap.insert(400, "max".to_string());

    assert_eq!(artmap.pop_first().unwrap().0, 100);
    assert_eq!(artmap.pop_last().unwrap().0, 400);
}

enum TestOperation {
    Insert,
    Delete,
}

impl Distribution<TestOperation> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TestOperation {
        match rng.gen_range(0..=3) {
            // 25% Delete, 75% Insert
            0 => TestOperation::Delete,
            _ => TestOperation::Insert,
        }
    }
}

#[test]
fn test_when_executing_random_operations_min_max_returns_same_as_btree() {
    let mut artmap = U64ArtMap::<String>::new();
    let mut btree = BTreeMap::<u64, String>::new();

    let mut rng = rand::thread_rng();
    for _ in 0..10000 {
        let random_op = rng.gen::<TestOperation>();
        match random_op {
            TestOperation::Insert => {
                let random_key = rng.gen::<u64>();

                artmap.insert(random_key, random_key.to_string());
                btree.insert(random_key, random_key.to_string());
            }
            TestOperation::Delete => {
                let random_key = rng.gen::<u64>();

                artmap.delete(random_key);
                btree.remove(&random_key);
            }
        }

        assert_eq!(artmap.minimum().unwrap().0, *btree.iter().next().unwrap().0);
        assert_eq!(artmap.maximum().unwrap().0, *btree.iter().last().unwrap().0);
    }
}
