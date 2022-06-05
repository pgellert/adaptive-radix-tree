extern crate adaptive_radix_tree;

use adaptive_radix_tree::art::*;

#[test]
fn test_get_mut_returns_none_when_art_is_empty() {
    let mut ds = ArtTree::<u32>::new();

    let result = ds.get_mut(&[1, 2, 3], 3);
    assert!(result.is_none());
}

#[test]
fn test_art_insert_inserts_single_element() {
    let mut ds = ArtTree::<u32>::new();
    let key = [1, 2, 3];
    let value = 17;
    let result = ds.insert(&key, key.len(), value);
    assert!(result.is_none());

    let minimum = ds.minimum();
    assert_eq!(value, *minimum.unwrap().1);
}

#[test]
fn art_minmax_with_two_works() {
    let mut ds = ArtTree::<u32>::new();
    let key = [1, 2, 3];
    let key_len = key.len();
    let value = 17;
    let result = ds.insert(&key, key_len, value);
    assert!(result.is_none());
    let key = [1, 3, 4];
    let key_len = key.len();
    let value = 122;
    let result = ds.insert(&key, key_len, value);
    assert!(result.is_none());

    let min_node = ds.minimum();
    assert!(min_node.is_some());
    assert_eq!(min_node.unwrap().1, &17);
    let max_node = ds.maximum();
    assert!(max_node.is_some());
    assert_eq!(max_node.unwrap().1, &122);
}

#[test]
fn art_successive_insert_works() {
    let mut ds = ArtTree::<u32>::new();
    for i in 0..10 {
        let key = [i % 16, i % 8, i % 4, i % 2];
        let result = ds.insert(&key, key.len(), i as u32);
        assert!(result.is_none());
    }

    let min_node = ds.minimum();
    assert!(min_node.is_some());
    assert_eq!(min_node.unwrap().1, &0);
    let max_node = ds.maximum();
    assert!(max_node.is_some());
    assert_eq!(max_node.unwrap().1, &9);
}

#[test]
fn art_iterator_works() {
    let mut ds = ArtTree::<u32>::new();
    for i in 0..10 {
        let key = [i % 16, i % 8, i % 4, i % 2];
        let result = ds.insert(&key, key.len(), i as u32);
        assert!(result.is_none());
    }

    let mut counter = 0;

    ds.iter(|val| {
        counter += 1;
        false
    });

    assert_eq!(counter, 10);
}

#[test]
fn art_delete_works() {
    let mut ds = ArtTree::new();

    let edge_cases = vec![1, 3, 4, 5, 15, 16, 17, 47, 48, 49, 255, 256, 257, 3000];

    for case in edge_cases {
        for _ in 0..3 {
            let mut keys: Vec<_> = (0..case)
                .map(|i| {
                    [
                        (i % 10) as u8,
                        (i % 20) as u8,
                        (i % 50) as u8,
                        (i % 256) as u8,
                    ]
                })
                .collect();
            for (i, key) in keys.iter().enumerate() {
                let result = ds.insert(key, key.len(), i as u32);
                assert_eq!(
                    result, None,
                    "Error inserting value {:?} with key {:?}",
                    i, &key
                );
            }

            for (i, key) in keys.iter().enumerate() {
                let result = ds.delete(key, key.len());
                assert_eq!(result, Some(i as u32));
            }

            let min_node = ds.minimum();
            assert!(min_node.is_none());
        }

        // TODO: assert on data structure
    }
}

#[test]
fn art_insert_debug() {
    let mut ds = ArtTree::new();
    let keys: Vec<_> = (0..16u32)
        .map(|i| 100 * i)
        .map(|i| {
            [
                (i % 10) as u8,
                (i % 20) as u8,
                (i % 50) as u8,
                (i % 256) as u8,
            ]
        })
        .collect();
    for (i, key) in keys.iter().enumerate() {
        let result = ds.insert(key, key.len(), i as u32);
        assert!(result.is_none());
    }

    let breaking_key = make_interesting_key(1600);
    let _result = ds.insert(breaking_key.as_ref(), breaking_key.len(), 10u32);
}

fn make_interesting_key(i: u32) -> Box<[u8; 4]> {
    Box::new([
        (i % 10) as u8,
        (i % 20) as u8,
        (i % 50) as u8,
        (i % 256) as u8,
    ])
}

#[test]
fn art_pop_first_works() {
    let mut ds = ArtTree::<u32>::new();

    let data = vec![([1, 2, 3], 17), ([1, 2, 4], 18)];

    for (key, value) in data.clone().into_iter() {
        let result = ds.insert(&key, key.len(), value);
        assert!(result.is_none());
    }

    let get_back = ds.pop_first();
    assert!(get_back.is_some());

    let kv = get_back.unwrap();
    let expected_kv = (data[0].0.as_ref(), data[0].1);
    assert!(kv_pair_eq(kv, expected_kv));
}

#[test]
fn art_pop_last_works() {
    let mut ds = ArtTree::<u32>::new();

    let data = vec![([1, 2, 3], 17), ([1, 2, 4], 18)];

    for (key, value) in data.clone().into_iter() {
        let result = ds.insert(&key, key.len(), value);
        assert!(result.is_none());
    }

    let get_back = ds.pop_last();
    assert!(get_back.is_some());

    let kv = get_back.unwrap();
    let expected_kv = (data[1].0.as_ref(), data[1].1);
    assert!(kv_pair_eq(kv, expected_kv));
}

#[test]
fn art_pop_last_twice_works() {
    let mut ds = ArtTree::<u32>::new();

    let data = vec![([1, 2, 3], 17), ([1, 2, 4], 18)];

    for (key, value) in data.clone().into_iter() {
        let result = ds.insert(&key, key.len(), value);
        assert!(result.is_none());
    }

    let get_back = ds.pop_last();
    assert!(get_back.is_some());

    let get_back = ds.pop_last();
    assert!(get_back.is_some());
}

fn kv_pair_eq(left: (Box<[u8]>, u32), right: (&[u8], u32)) -> bool {
    left.1 == right.1 && left.0.iter().zip(right.0).all(|(k1, k2)| *k1 == *k2)
}
