use crate::art::ArtTree;

struct U64ArtMap<V>{
    tree: ArtTree<V>,
}


impl<V> U64ArtMap<V>{
    pub fn new() -> Self{
        Self{
            tree: ArtTree::new(),
        }
    }

    pub fn search(&self, key: u64) -> Option<&V>{
        let key_bytes = key.to_be_bytes();
        self.tree.search(&key_bytes, key_bytes.len())
    }

    pub fn minimum(&self) -> Option<&V> {
        self.tree.minimum()
    }

    pub fn maximum(&self) -> Option<&V> {
        self.tree.maximum()
    }

    pub fn insert(&mut self, key: u64, value: V) -> Option<V>{
        let key_bytes = key.to_be_bytes();
        self.tree.insert(&key_bytes, key_bytes.len(), value)
    }

    pub fn delete(&mut self, key: u64) -> Option<V>{
        let key_bytes = key.to_be_bytes();
        self.tree.delete(&key_bytes, key_bytes.len())
    }

    pub fn iter<CB>(&mut self, mut callback: CB) -> bool
        where
            CB: FnMut(&V) -> bool
    {
        self.tree.iter(&mut callback)
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bytes_representation() {
        println!("{:?}", 17u64.to_be_bytes());
    }

    #[test]
    fn test_search_works(){
        let artmap = U64ArtMap::<String>::new();
        let result = artmap.search(10);
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

}