use crate::art::ArtTree;

/// Map indexed by u64-keys using an Adaptive Radix Tree
#[derive(Clone, Debug)]
pub struct U64ArtMap<V> {
    tree: ArtTree<V>,
}

impl<V> U64ArtMap<V> {
    pub fn new() -> Self {
        Self {
            tree: ArtTree::new(),
        }
    }

    /// Returns a mutable reference to the value stored at the given key if it exists
    pub fn get_mut(&mut self, key: &u64) -> Option<&mut V> {
        let key_bytes = key.to_be_bytes();
        self.tree.get_mut(&key_bytes, key_bytes.len())
    }

    /// Returns the key and a reference to the value of the minimum element in the map
    pub fn minimum(&self) -> Option<(u64, &V)> {
        self.tree.minimum().map(|(k, v)| (u8_list_to_u64_key(k), v))
    }

    /// Returns the key and a reference to the value of the maximum element in the map
    pub fn maximum(&self) -> Option<(u64, &V)> {
        self.tree.maximum().map(|(k, v)| (u8_list_to_u64_key(k), v))
    }

    /// Returns the key and a reference to the value of the minimum element in the map
    pub fn minimum_mut(&mut self) -> Option<(u64, &mut V)> {
        self.tree
            .minimum_mut()
            .map(|(k, v)| (u8_list_to_u64_key(k), v))
    }

    /// Returns the key and a reference to the value of the maximum element in the map
    pub fn maximum_mut(&mut self) -> Option<(u64, &mut V)> {
        self.tree
            .maximum_mut()
            .map(|(k, v)| (u8_list_to_u64_key(k), v))
    }

    /// Inserts the given value at the given key and returns the previous value stored at the key if
    /// such exists.
    pub fn insert(&mut self, key: u64, value: V) -> Option<V> {
        let key_bytes = key.to_be_bytes();
        self.tree.insert(&key_bytes, key_bytes.len(), value)
    }

    /// Deletes and returns the value stored at the given key.
    pub fn delete(&mut self, key: u64) -> Option<V> {
        let key_bytes = key.to_be_bytes();
        self.tree.delete(&key_bytes, key_bytes.len())
    }

    /// Iterates over the values stored in the map in sorted order and calls the callback on the
    /// values.
    ///
    /// If the callback returns true, the iteration stops (before continuing to any successive
    /// element).
    pub fn iter<CB>(&mut self, mut callback: CB) -> bool
    where
        CB: FnMut(&V) -> bool,
    {
        self.tree.iter(&mut callback)
    }

    /// Removes and returns the minimal key-value pair from the map
    pub fn pop_first(&mut self) -> Option<(u64, V)> {
        self.tree
            .pop_first()
            .map(|(k, v)| (u8_list_to_u64_key(&k), v))
    }

    /// Removes and returns the maximal key-value pair from the map
    pub fn pop_last(&mut self) -> Option<(u64, V)> {
        self.tree
            .pop_last()
            .map(|(k, v)| (u8_list_to_u64_key(&k), v))
    }
}

fn u8_list_to_u64_key(stored_key: &Box<[u8]>) -> u64 {
    let mut key_slice = [0; 8];
    for i in 0..8 {
        key_slice[i] = stored_key[i];
    }
    u64::from_be_bytes(key_slice)
}

#[cfg(test)]
mod tests {
    #[test]
    fn u64_mapping_and_reverse_mapping_test() {
        let u64key = 123456u64;
        assert_eq!(
            u64key,
            crate::u64_art_map::u8_list_to_u64_key(&u64key.to_be_bytes().into())
        );
    }
}
