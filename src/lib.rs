mod art;

#[cfg(test)]
mod tests {
    use super::*;
    use super::art::*;
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn art_new_works(){
        let mut ds = ArtTree::new();
    }

    #[test]
    fn art_search_works(){
        let mut ds = ArtTree::new();

        let result = ds.search(&[1,2,3], 3);
        assert!(result.is_none());
    }

    #[test]
    fn art_insert_to_empty_works(){
        let mut ds = ArtTree::new();
        let key = [1,2,3];
        let key_len = key.len() as u32;
        let value = 17;
        let result = ds.insert(&key, key_len, value);
        assert!(result.is_none());

        let get_back = ds.minimum();
        assert!(get_back.is_some());
    }

    #[test]
    fn art_minmax_with_two_works(){
        let mut ds = ArtTree::new();
        let key = [1,2,3];
        let key_len = key.len() as u32;
        let value = 17;
        let result = ds.insert(&key, key_len, value);
        println!("Result: {:?}", result);
        assert!(result.is_none());
        let key = [1,3,4];
        let key_len = key.len() as u32;
        let value = 122;
        let result = ds.insert(&key, key_len, value);
        println!("Result: {:?}", result);
        assert!(result.is_none());

        let min_node = ds.minimum();
        assert!(min_node.is_some());
        assert_eq!(min_node.unwrap().value, 17);
        let max_node = ds.maximum();
        assert!(max_node.is_some());
        assert_eq!(max_node.unwrap().value, 122);
    }
}
