extern crate adaptive_radix_tree;

use adaptive_radix_tree::art::*;

static DUMMY_VALUE: u32 = 17;
static DUMMY_VALUE_2: u32 = 18;

#[test]
fn test_insert_into_empty_tree() {
    let mut ds = ArtTree::<u32>::new();

    assert!(ds.get_mut(&[1, 2, 3]).is_none());
    assert!(ds.insert(&[1, 2, 3], DUMMY_VALUE).is_none());
    assert!(ds.get_mut(&[1, 2, 3]).is_some());
}

#[test]
fn test_insert_and_replace_into_empty_tree() {
    let mut ds = ArtTree::<u32>::new();

    assert!(ds.insert(&[1, 2, 3], DUMMY_VALUE).is_none());
    assert!(ds.insert(&[1, 2, 3], DUMMY_VALUE_2).is_some());
    assert_eq!(*ds.get_mut(&[1, 2, 3]).unwrap(), DUMMY_VALUE_2);
}

#[test]
fn test_insert_single_mismatch() {
    let mut ds = ArtTree::<u32>::new();

    assert!(ds.insert(&[1, 1, 1, 1, 1], DUMMY_VALUE).is_none());
    assert!(ds.insert(&[1, 1, 2, 1, 1], DUMMY_VALUE_2).is_none());

    assert_eq!(*ds.get_mut(&[1, 1, 1, 1, 1]).unwrap(), DUMMY_VALUE);
    assert_eq!(*ds.get_mut(&[1, 1, 2, 1, 1]).unwrap(), DUMMY_VALUE_2);
}

#[test]
fn art_minmax_with_two_works() {
    let mut ds = ArtTree::<u32>::new();
    ds.insert(&[1, 2, 3], 17);
    ds.insert(&[1, 3, 4], 122);

    let min_node = ds.minimum();
    assert!(min_node.is_some());
    assert_eq!(min_node.unwrap().1, &17);
    let max_node = ds.maximum();
    assert!(max_node.is_some());
    assert_eq!(max_node.unwrap().1, &122);
}

#[test]
fn art_minmax_with_four_unique_elements_works() {
    let mut ds = ArtTree::<String>::new();
    assert!(insert_kv(&mut ds, [0, 0, 0, 100], "min".to_string()).is_none());
    assert!(insert_kv(&mut ds, [0, 0, 0, 200], "middle".to_string()).is_none());
    assert!(insert_kv(&mut ds, [0, 0, 1, 44], "middle".to_string()).is_none());
    assert!(insert_kv(&mut ds, [0, 0, 1, 144], "max".to_string()).is_none());

    assert_eq!(**ds.minimum().unwrap().0, [0, 0, 0, 100]);
    assert_eq!(**ds.maximum().unwrap().0, [0, 0, 1, 144]);
}

fn insert_kv<V>(data: &mut ArtTree<V>, key_list: [u8; 4], value: V) -> Option<V> {
    return data.insert(&key_list, value);
}

#[test]
fn art_successive_insert_works() {
    let mut ds = ArtTree::<u32>::new();
    for i in 0..10 {
        let key = [i % 16, i % 8, i % 4, i % 2];
        let result = ds.insert(&key, i as u32);
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
        let result = ds.insert(&key, i as u32);
        assert!(result.is_none());
    }

    let mut counter = 0;

    ds.iter(|_val| {
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
            let keys: Vec<_> = (0..case)
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
                let result = ds.insert(key, i as u32);
                assert_eq!(
                    result, None,
                    "Error inserting value {:?} with key {:?}",
                    i, &key
                );
            }

            for (i, key) in keys.iter().enumerate() {
                let result = ds.delete(key);
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
        let result = ds.insert(key, i as u32);
        assert!(result.is_none());
    }

    let breaking_key = make_interesting_key(1600);
    let _result = ds.insert(breaking_key.as_ref(), 10u32);
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
        let result = ds.insert(&key, value);
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
        let result = ds.insert(&key, value);
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
        let result = ds.insert(&key, value);
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
