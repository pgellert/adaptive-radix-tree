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
}
