extern crate adaptive_radix_tree;

use adaptive_radix_tree::art::*;

#[test]
fn art_new_works(){
    let mut ds = ArtTree::<u32>::new();
}

#[test]
fn art_search_works(){
    let mut ds = ArtTree::<u32>::new();

    let result = ds.get_mut(&[1,2,3], 3);
    assert!(result.is_none());
}

#[test]
fn art_insert_to_empty_works(){
    let mut ds = ArtTree::<u32>::new();
    let key = [1,2,3];
    let key_len = key.len();
    let value = 17;
    let result = ds.insert(&key, key_len, value);
    assert!(result.is_none());

    let get_back = ds.minimum();
    assert!(get_back.is_some());
}

#[test]
fn art_minmax_with_two_works(){
    let mut ds = ArtTree::<u32>::new();
    let key = [1,2,3];
    let key_len = key.len();
    let value = 17;
    let result = ds.insert(&key, key_len, value);
    println!("Result: {:?}", result);
    assert!(result.is_none());
    let key = [1,3,4];
    let key_len = key.len();
    let value = 122;
    let result = ds.insert(&key, key_len, value);
    println!("Result: {:?}", result);
    assert!(result.is_none());

    let min_node = ds.minimum();
    assert!(min_node.is_some());
    assert_eq!(min_node.unwrap().1, &17);
    let max_node = ds.maximum();
    assert!(max_node.is_some());
    assert_eq!(max_node.unwrap().1, &122);
}

#[test]
fn art_successive_insert_works(){
    let mut ds = ArtTree::<u32>::new();
    for i in 0..10{
        let key = [i%16,i%8,i%4,i%2];
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
fn art_iterator_works(){
    let mut ds = ArtTree::<u32>::new();
    for i in 0..10{
        let key = [i%16,i%8,i%4,i%2];
        let result = ds.insert(&key, key.len(), i as u32);
        assert!(result.is_none());
    }

    let mut counter = 0;

    ds.iter(|val| {
        println!("Visiting {:}", val);
        counter+=1;
        false
    });

    assert_eq!(counter, 10);
}

#[test]
fn art_delete_works(){
    let mut ds = ArtTree::new();

    let edge_cases = vec![1,3,4,5,15,16,17,47,48,49,255,256,257,3000];

    for case in edge_cases{
        for _ in 0..3{
            let mut keys: Vec<_> = (0..case).map(|i| [(i%10) as u8,(i%20) as u8,(i%50) as u8, (i%256) as u8]).collect();
            //let mut keys: Vec<_> = (0..200u32).map(|i| [(i%256) as u8]).collect();
            for (i,key) in keys.iter().enumerate(){
                //println!("Inserting: {:?}", key);
                let result = ds.insert(key, key.len(), i as u32);
                assert_eq!(result, None, "Error inserting value {:?} with key {:?}", i, &key);
            }

            //println!("Data structure: {:?}", ds);

            for (i,key) in keys.iter().enumerate(){
                //println!("Data structure: {:?}", ds);
                //println!("Deleting: {:?}", key);
                let result = ds.delete(key, key.len());
                assert_eq!(result, Some(i as u32));
            }

            //println!("Data structure: {:?}", ds);

            let min_node = ds.minimum();
            assert!(min_node.is_none());
        }
        println!("Data structure: {:?}", ds);
    }
}

#[test]
fn art_insert_debug(){
    let mut ds = ArtTree::new();
    let keys: Vec<_> = (0..16u32).map(|i| 100*i).map(|i| [(i%10) as u8,(i%20) as u8,(i%50) as u8, (i%256) as u8]).collect();
    for (i,key) in keys.iter().enumerate(){
        println!("Inserting: {:?}", key);
        let result = ds.insert(key, key.len(), i as u32);
        assert!(result.is_none());
    }

    println!("(Partial) Data structure: {:?}", ds);

    let breaking_key = make_interesting_key(1600);
    println!("Inserting: {:?}", breaking_key);
    let result = ds.insert(breaking_key.as_ref(), breaking_key.len(), 10u32);

    println!("(End) Data structure: {:?}", ds);
}

fn make_interesting_key(i: u32) -> Box<[u8;4]>{
    Box::new([(i % 10) as u8, (i % 20) as u8, (i % 50) as u8, (i % 256) as u8])
}

#[test]
fn art_pop_first_works(){
    let mut ds = ArtTree::<u32>::new();

    let data = vec![
        ([1,2,3], 17),
        ([1,2,4], 18),
    ];

    for (key,value) in data.clone().into_iter(){
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
fn art_pop_last_works(){
    let mut ds = ArtTree::<u32>::new();

    let data = vec![
        ([1,2,3], 17),
        ([1,2,4], 18),
    ];

    for (key,value) in data.clone().into_iter(){
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
fn art_pop_last_twice_works(){
    let mut ds = ArtTree::<u32>::new();


    let data = vec![
        ([1,2,3], 17),
        ([1,2,4], 18),
    ];


    for (key,value) in data.clone().into_iter(){
        let result = ds.insert(&key, key.len(), value);
        assert!(result.is_none());
    }

    let get_back = ds.pop_last();
    assert!(get_back.is_some());

    let get_back = ds.pop_last();
    assert!(get_back.is_some());
}

fn kv_pair_eq(left: (Box<[u8]>, u32), right: (&[u8], u32)) -> bool {
    left.1 == right.1 && left.0.iter().zip(right.0).all(|(k1,k2)| *k1 == *k2)
}