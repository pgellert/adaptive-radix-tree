use crate::art::ArtTree;

#[derive(Clone,Debug)]
pub struct U64ArtMap<V>{
    tree: ArtTree<V>,
}


impl<V> U64ArtMap<V>{
    pub fn new() -> Self{
        Self{
            tree: ArtTree::new(),
        }
    }

    pub fn get_mut(&mut self, key: &u64) -> Option<&mut V>{
        let key_bytes = key.to_be_bytes();
        self.tree.get_mut(&key_bytes, key_bytes.len())
    }

    pub fn minimum(&self) -> Option<(u64, &V)> {
        self.tree.minimum().map(|(k,v)| {
            let mut key_slice = [0;8];
            for i in 0..8{
                key_slice[i] = k[i];
            }
            (u64::from_be_bytes(key_slice), v)
        })
    }

    pub fn maximum(&self) -> Option<(u64, &V)> {
        self.tree.maximum().map(|(k,v)| {
            let mut key_slice = [0;8];
            for i in 0..8{
                key_slice[i] = k[i];
            }
            (u64::from_be_bytes(key_slice), v)
        })
    }

    pub fn minimum_mut(&mut self) -> Option<(u64, &mut V)> {
        self.tree.minimum_mut().map(|(k,v)| {
            let mut key_slice = [0;8];
            for i in 0..8{
                key_slice[i] = k[i];
            }
            (u64::from_be_bytes(key_slice), v)
        })
    }

    pub fn maximum_mut(&mut self) -> Option<(u64, &mut V)> {
        self.tree.maximum_mut().map(|(k,v)| {
            let mut key_slice = [0;8];
            for i in 0..8{
                key_slice[i] = k[i];
            }
            (u64::from_be_bytes(key_slice), v)
        })
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

    pub fn pop_first(&mut self) -> Option<(u64, V)>{
        self.tree.pop_first().map(|(k,v)| {
            let mut key_slice = [0;8];
            for i in 0..8{
                key_slice[i] = k[i];
            }
            (u64::from_be_bytes(key_slice), v)
        })
    }

    pub fn pop_last(&mut self) -> Option<(u64, V)>{
        self.tree.pop_last().map(|(k,v)| {
            let mut key_slice = [0;8];
            for i in 0..8{
                key_slice[i] = k[i];
            }
            (u64::from_be_bytes(key_slice), v)
        })
    }




}
