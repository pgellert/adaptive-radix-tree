mod art;
mod u64_art_map;

#[cfg(test)]
mod tests {
    use super::art::*;
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn art_new_works(){
        let mut ds = ArtTree::<u32>::new();
    }

    #[test]
    fn art_search_works(){
        let ds = ArtTree::<u32>::new();

        let result = ds.search(&[1,2,3], 3);
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
        assert_eq!(*min_node.unwrap(), 17);
        let max_node = ds.maximum();
        assert!(max_node.is_some());
        assert_eq!(*max_node.unwrap(), 122);
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
        assert_eq!(*min_node.unwrap(), 0);
        let max_node = ds.maximum();
        assert!(max_node.is_some());
        assert_eq!(*max_node.unwrap(), 9);
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
        let keys: Vec<_> = (0..3000u32).map(|i| [(i%10) as u8,(i%20) as u8,(i%50) as u8, (i%256) as u8]).collect();
        for (i,key) in keys.iter().enumerate(){
            println!("Inserting: {:?}", key);
            let result = ds.insert(key, key.len(), i as u32);
            assert!(result.is_none());
        }

        println!("Data structure: {:?}", ds);

        for (i,key) in keys.iter().enumerate(){
            println!("Deleting: {:?}", key);
            let result = ds.delete(key, key.len());
            assert_eq!(result, Some(i as u32));
        }

        let min_node = ds.minimum();
        assert!(min_node.is_none());
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
}
