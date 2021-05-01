extern crate adaptive_radix_tree;

use adaptive_radix_tree::u64_art_map::*;

#[test]
fn test_search_works(){
    let mut artmap = U64ArtMap::<String>::new();
    let result = artmap.get_mut(10);
    assert_eq!(None, result);
}

#[test]
fn test_insert_works(){
    let mut artmap = U64ArtMap::<String>::new();
    let replaced = artmap.insert(17, String::from("Hello"));
    assert_eq!(None, replaced);

    let min_val = artmap.minimum();
    assert!(min_val.is_some());

    let max_val = artmap.maximum();
    assert!(max_val.is_some());
}

#[test]
fn test_delete_works(){
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