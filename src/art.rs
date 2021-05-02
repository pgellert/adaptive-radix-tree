
use std::cmp::min;
use std::mem;
use std::hint::unreachable_unchecked;

const MAX_PREFIX_LEN: usize = 10;

type ArtError = u32;
type Result<V> = std::result::Result<V, ArtError>;

#[derive(Debug,Clone)]
enum Node<V>{
    Empty,
    Leaf(Box<ArtNodeLeaf<V>>),
    Internal(Box<ArtNodeInternal<V>>),
}

#[derive(Debug,Copy, Clone)]
struct InternalNodeHeader {
    partial_len: usize,
    num_children: u8,
    partial: [u8; MAX_PREFIX_LEN],
}

#[derive(Debug,Clone)]
pub struct ArtNodeLeaf<V>{
    pub value: V,
    key_len: usize,
    key: Box<[u8]>,
}

#[derive(Debug,Clone)]
struct ArtNodeInternal<V>{
    header: InternalNodeHeader,
    inner: ArtNodeInternalInner<V>,
}

#[derive(Debug,Clone)]
enum ArtNodeInternalInner<V> {
    Node4 {
        keys: [u8;4],
        children: [Node<V>;4],
    },
    Node16 {
        keys: [u8;16],
        children: [Node<V>;16],
    },
    Node48 {
        keys: [u8;256],
        children: [Node<V>;48],
    },
    Node256 {
        children: [Node<V>;256],
    },
}

#[derive(Debug, Clone)]
pub struct ArtTree<V>{
    root: Node<V>,
    size: u64,
}

impl<V> ArtTree<V>{
    pub fn new() -> Self{
        Self{
            root: Node::Empty,
            size: 0
        }
    }


    /// Searches for a value in the ARV tree
    /// @arg t Vhe tree
    /// @arg key Vhe key
    /// @arg key_len Vhe length of the key
    /// @return NULL if the item was not found, otherwise
    /// the value pointer is returned.
    pub fn get_mut(&mut self, key: &[u8], key_len: usize) -> Option<&mut V>{
        let mut n_iter = &mut self.root;
        let mut depth = 0;
        loop {
            match *n_iter {
                Node::Leaf (ref mut leaf) => {
                    if leaf.matches(key, key_len, depth) {
                        return Some(&mut leaf.value);
                    }
                    return None;
                }
                Node::Internal (ref mut internal) => {
                    let header = internal.header;

                    if header.partial_len != 0 {

                        let prefix_len = header.check_prefix( key, key_len, depth);
                        if prefix_len != min(MAX_PREFIX_LEN, header.partial_len) {
                            return None;
                        }
                        depth = depth + header.partial_len;
                    }


                    n_iter = internal.find_child_mut(key[depth])?;
                    depth+=1;
                }
                Node::Empty => return None,
            }
        }
    }

    pub fn minimum(&self) -> Option<(&Box<[u8]>,&V)>{
        self.root.minimum().map(|leaf| (&leaf.key, &leaf.value))
    }

    pub fn maximum(&self) -> Option<(&Box<[u8]>,&V)>{
        self.root.maximum().map(|leaf| (&leaf.key, &leaf.value))
    }

    pub fn minimum_mut(&mut self) -> Option<(&mut Box<[u8]>,&mut V)>{
        self.root.minimum_mut().map(|leaf| (&mut leaf.key, &mut leaf.value))
    }

    pub fn maximum_mut(&mut self) -> Option<(&mut Box<[u8]>,&mut V)>{
        self.root.maximum_mut().map(|leaf| (&mut leaf.key, &mut leaf.value))
    }


    pub fn pop_first(&mut self) -> Option<(Box<[u8]>,V)>{
        let (min_key, _) = self.minimum()?;
        let min_key = min_key.clone();
        let key_tmp = min_key.clone();
        let key = key_tmp.as_ref();
        let min_val = self.delete(key, key.len()).unwrap();
        return Some((min_key, min_val));
    }

    pub fn pop_last(&mut self) -> Option<(Box<[u8]>,V)>{
        let (min_key, _) = self.maximum()?;
        let min_key = min_key.clone();
        let key_tmp = min_key.clone();
        let key = key_tmp.as_ref();
        let min_val = self.delete(key, key.len()).unwrap();
        return Some((min_key, min_val));
    }

    /// inserts a new value into the art tree
    /// @arg t the tree
    /// @arg key the key
    /// @arg key_len the length of the key
    /// @arg value opaque value.
    /// @return null if the item was newly inserted, otherwise
    /// the old value pointer is returned.
    pub fn insert(&mut self, key: &[u8], key_len: usize, value: V) -> Option<V> {
        let result = self.root.recursive_insert(key, key_len, value, 0, true);
        if result.is_none() {
            self.size += 1;
        }
        result
    }

    /// Deletes a value from the ARV tree
    /// @arg t Vhe tree
    /// @arg key Vhe key
    /// @arg key_len Vhe length of the key
    /// @return NULL if the item was not found, otherwise
    /// the value pointer is returned.
    pub fn delete(&mut self, key: &[u8], key_len: usize) -> Option<V> {
        let (root, result) = mem::take(&mut self.root).recursive_delete(key, key_len, 0);
        self.root = root;
        if result.is_some(){
            self.size -= 1;
        }
        result
    }


    /// Iterates through the entries pairs in the map,
    /// invoking a callback for each. Vhe call back gets a
    /// key, value for each and returns an integer stop value.
    /// If the callback returns non-zero, then the iteration stops.
    /// @arg t Vhe tree to iterate over
    /// @arg cb Vhe callback function to invoke
    /// @return true on success, or the return of the callback.
    pub fn iter<CB>(&mut self, mut callback: CB) -> bool
        where
            CB: FnMut(&V) -> bool
    {
        self.root.recursive_iter(&mut callback)
    }
}


impl<V> Default for Node<V> {
    fn default() -> Self {
        Self::Empty
    }
}

impl<V> Node<V>{
    fn minimum(&self) -> Option<&ArtNodeLeaf<V>> {
        match self{
            Node::Empty => None,
            Node::Leaf (leaf) => Some(leaf.as_ref()),
            Node::Internal(internal) => internal.minimum(),
        }
    }

    fn minimum_mut(&mut self) -> Option<&mut ArtNodeLeaf<V>> {
        match self{
            Node::Empty => None,
            Node::Leaf (leaf) => Some(leaf.as_mut()),
            Node::Internal(internal) => internal.minimum_mut(),
        }
    }

    fn maximum(&self) -> Option<&ArtNodeLeaf<V>> {
        match self{
            Node::Empty => None,
            Node::Leaf (leaf) => Some(leaf.as_ref()),
            Node::Internal (internal) => internal.maximum(),
        }
    }

    fn maximum_mut(&mut self) -> Option<&mut ArtNodeLeaf<V>> {
        match self{
            Node::Empty => None,
            Node::Leaf (leaf) => Some(leaf.as_mut()),
            Node::Internal (internal) => internal.maximum_mut(),
        }
    }

    fn pop_first(&mut self) -> Option<(Box<[u8]>, V)> {
        match self{
            Node::Empty => None,
            Node::Leaf (_) => {
                match mem::take(self) {
                    Node::Leaf (leaf) => return Some((leaf.key, leaf.value)),
                    _ => unreachable!(),
                };
            },
            Node::Internal(internal) => internal.pop_first(),
        }
    }

    fn pop_last(&mut self) -> Option<(Box<[u8]>, V)> {
        match self{
            Node::Empty => None,
            Node::Leaf (_) => {
                match mem::take(self) {
                    Node::Leaf (leaf) => return Some((leaf.key, leaf.value)),
                    _ => unreachable!(),
                };
            },
            Node::Internal(internal) => internal.pop_last(),
        }
    }

    fn recursive_insert(&mut self, key: &[u8], key_len: usize, value: V, mut depth: usize, replace: bool) -> Option<V> {
        let mut split = false;
        let mut split_internal = false;
        let mut prefix_save = 0;

        match *self {
            Node::Leaf ( ref mut leaf ) => {
                // Check if we are updating an existing value
                if leaf.matches(key, key_len, depth){
                    // TODO: use replace here?
                    let old_value = std::mem::replace(&mut leaf.value, value);
                    return Some(old_value);
                }

                // New value, we must split the leaf into a node4
                split = true;
            }
            Node::Internal (ref mut internal) => {
                let mut n = internal.header;

                // Check if given node has a prefix
                if n.partial_len != 0 {
                    // Determine if the prefixes differ, since we need to split
                    let prefix_diff = internal.prefix_mismatch(key, key_len, depth);
                    if prefix_diff >= n.partial_len{
                        depth += n.partial_len;

                        // Find a child to recurse to
                        let child = internal.find_child_mut(key[depth]); // TODO: double check
                        if let Some(node) = child {
                            return node.recursive_insert(key, key_len, value, depth+1, replace);
                        }

                        // No child, node goes within us
                        let new_leaf = Node::Leaf (Box::new(ArtNodeLeaf::new(key, key_len, value)));
                        internal.add_child(key[depth], new_leaf);

                        return None;
                    }

                    split_internal = true;
                    prefix_save = prefix_diff;
                }

                let child = internal.find_child_mut(key[depth]); // TODO: double check
                if let Some(node) = child{
                    return node.recursive_insert(key, key_len, value, depth+1, replace);
                }

                let new_leaf = Node::Leaf(Box::new(ArtNodeLeaf::new(key, key_len, value)));
                internal.add_child(key[depth], new_leaf);

                return None;
            }
            Node::Empty => {
                let new_leaf = Box::new(ArtNodeLeaf::new(key, key_len, value));
                *self = Node::Leaf(new_leaf);
                return None;
            }
        };

        if split {
            // Create a new leaf
            let mut new_leaf = ArtNodeLeaf::new(key, key_len, value);

            // Determine longest prefix
            let longest_prefix = match self{
                Node::Leaf(ref leaf) => leaf.longest_common_prefix(&mut new_leaf, depth),
                _ => unreachable!(),
            };
            let mut partial_new = [0u8; MAX_PREFIX_LEN];
            for i in 0..min(MAX_PREFIX_LEN,longest_prefix){
                partial_new[i] = key[depth + i];
            }

            let arr = [Node::<V>::INIT;4];

            let mut internal = Node::Internal(Box::new(ArtNodeInternal {
                header: InternalNodeHeader {
                    partial_len: longest_prefix,
                    num_children: 0,
                    partial: partial_new,
                },
                inner: ArtNodeInternalInner::Node4 { keys: [0u8;4], children: arr },
            }));

            match mem::replace(self, internal){
                Node::Leaf(old_leaf) => {
                    match self {
                        Node::Internal(internal) => {
                            internal.add_child(old_leaf.as_ref().key[depth+longest_prefix], Node::Leaf(old_leaf));
                            internal.add_child(new_leaf.key[depth+longest_prefix], Node::Leaf(Box::new(new_leaf)));
                        }
                        _ => unreachable!(),
                    }
                }
                _ => unreachable!(),
            }

            return None;
        }

        if split_internal {
            let prefix_diff = prefix_save;

            // Create a new node
            let mut partial = [0u8; MAX_PREFIX_LEN];
            let mut partial_len = 0;

            {
                let n = match self{
                    Node::Internal(ref internal) => internal.header,
                    _ => unreachable!(),
                };
                for i in 0..min(MAX_PREFIX_LEN, prefix_diff){
                    partial[i] = n.partial[i];
                }
                partial_len = n.partial_len;
            }


            let mut new_node = Node::Internal(Box::new(ArtNodeInternal {
                header: InternalNodeHeader {
                    partial_len: prefix_diff,
                    num_children: 0,
                    partial,
                },
                inner: ArtNodeInternalInner::Node4 { keys: [0u8;4], children: [Node::<V>::INIT;4] },
            }));

            // Adjust the prefix of the old node
            if partial_len <= MAX_PREFIX_LEN {
                match mem::replace(self, new_node) {
                    Node::Internal(mut old_node) => {
                        old_node.header.partial_len -= prefix_diff + 1;
                        for i in (0..min(MAX_PREFIX_LEN,old_node.header.partial_len)).rev(){
                            old_node.header.partial[i] = old_node.header.partial[prefix_diff+1+i];
                        }
                        match self {
                            Node::Internal(ref mut new_internal) => {
                                new_internal.add_child(old_node.header.partial[prefix_diff], Node::Internal(old_node));

                                let new_leaf = ArtNodeLeaf::new(key, key_len, value);
                                new_internal.add_child(key[depth+prefix_diff], Node::Leaf (Box::new(new_leaf)));


                                return None;
                            },
                            _ => unreachable!(),
                        }
                    },
                    _ => unreachable!(),
                }

            } else {
                match mem::replace(self, new_node) {
                    Node::Internal(mut internal) => {
                        internal.header.partial_len -= prefix_diff + 1;
                        let l = internal.minimum().unwrap();
                        let mut temp = vec![0u8;min(MAX_PREFIX_LEN,internal.header.partial_len)];
                        let c = l.key[depth+prefix_diff];
                        for i in 0..min(MAX_PREFIX_LEN,internal.header.partial_len){
                            temp[i] = l.key[depth+prefix_diff+1+i];
                        }
                        for i in 0..min(MAX_PREFIX_LEN,internal.header.partial_len){
                            internal.header.partial[i] = temp[i];
                        }

                        match *self{
                            Node::Internal(ref mut new_internal) =>{
                                new_internal.add_child(c,Node::Internal(internal));

                                let new_leaf = ArtNodeLeaf::new(key, key_len, value);
                                new_internal.add_child(key[depth+prefix_diff], Node::Leaf (Box::new(new_leaf)));


                                return None;
                            },
                            _ => unreachable!(),
                        }
                    },
                    _ => unreachable!(),
                }
            }

            /* TODO: double check
            let new_leaf = ArtNodeLeaf::new(key, key_len, value);

            new_node.add_child(key[depth+prefix_diff], Node::Leaf (Box::new(new_leaf)));

            *self = Node::Internal(Box::new(new_node));

            return None;
             */


        }

        unreachable!()
    }


    fn recursive_delete(mut self, key: &[u8], key_len: usize, mut depth: usize) -> (Self, Option<V>) {
        let mut delete_cleanup = false;

        match self {
            Node::Leaf (leaf) => {
                if leaf.matches(key, key_len, depth) {
                    return (Node::Empty, Some(leaf.value));
                } else {
                    return (Node::Leaf(leaf), None);
                }
            }
            Node::Internal (mut internal) => {
                // Bail if the prefix does not match
                if internal.header.partial_len != 0 {
                    let prefix_len = internal.header.check_prefix(key, key_len, depth);
                    if prefix_len != min(MAX_PREFIX_LEN, internal.header.partial_len){
                        return (Node::Internal (internal), None);
                    }
                    depth += internal.header.partial_len; // TODO: consider if mut depth is needed
                }

                // Find child node
                //let mut child = internal.find_child_mut(key[depth]);
                let child_pos = internal.find_child_index(key[depth]);
                if child_pos.is_none(){
                    return (Node::Internal (internal), None);
                }
                let mut child_pos = child_pos.unwrap();

                match *internal {
                    ArtNodeInternal { ref mut header, ref mut inner } => {
                        match inner {
                            ArtNodeInternalInner::Node4 { ref mut children, ref mut keys, .. } => {
                                let (child_res, return_val) = mem::take(&mut children[child_pos]).recursive_delete(key, key_len, depth + 1);
                                children[child_pos] = child_res;
                                if children[child_pos].is_empty() {
                                    for i in (child_pos+1)..header.num_children as usize {
                                        keys[i-1] = keys[i];
                                        children[i-1] = mem::take(&mut children[i]);
                                    }
                                    keys[(header.num_children-1) as usize] = 0;
                                    header.num_children -= 1;

                                    // Remove nodes with only a single child
                                    if header.num_children == 1 {
                                        match mem::take(&mut children[0]) {
                                            Node::Internal(mut internal) => {
                                                // Concatenate the prefixes
                                                let mut prefix = header.partial_len;
                                                if prefix < MAX_PREFIX_LEN {
                                                    header.partial[prefix] = keys[0];
                                                    prefix += 1;
                                                }
                                                if prefix < MAX_PREFIX_LEN {
                                                    let sub_prefix = min(internal.header.partial_len, MAX_PREFIX_LEN - prefix);
                                                    for i in 0..sub_prefix {
                                                        header.partial[prefix + i] = internal.header.partial[i];
                                                    }
                                                    prefix += sub_prefix;
                                                }

                                                // Store the prefix in the child
                                                for i in 0..min(prefix, MAX_PREFIX_LEN) {
                                                    internal.header.partial[i] = header.partial[i];
                                                }
                                                internal.header.partial_len += header.partial_len + 1;

                                                return (Node::Internal(internal), return_val);
                                            },
                                            Node::Leaf(leaf) => {
                                                //mem::replace(self, leaf);
                                                return (Node::Leaf(leaf), return_val);
                                            },
                                            _ => unreachable!(),
                                        }
                                    }
                                }
                                return (Node::Internal(internal), return_val);
                            },
                            ArtNodeInternalInner::Node16 { ref mut children, ref mut keys, .. } => {
                                let (child_res, return_val) = mem::take(&mut children[child_pos]).recursive_delete(key, key_len, depth + 1);
                                children[child_pos] = child_res;
                                if children[child_pos].is_empty() {
                                    for i in (child_pos+1)..header.num_children as usize {
                                        keys[i-1] = keys[i];
                                        children[i-1] = mem::take(&mut children[i]);
                                    }
                                    keys[(header.num_children-1) as usize] = 0;
                                    header.num_children -= 1;

                                    if header.num_children == 3 {
                                        let mut children_new: [Node<V>; 4] = [Node::INIT; 4];
                                        let mut keys_new: [u8; 4] = [0; 4];

                                        for i in 0..header.num_children as usize {
                                            keys_new[i] = keys[i];
                                            children_new[i] = mem::take(&mut children[i]);
                                        }

                                        let new_node = Node::Internal(Box::new(ArtNodeInternal{
                                            header: *header,
                                            inner: ArtNodeInternalInner::Node4 {
                                                keys: keys_new,
                                                children: children_new,
                                            }}));
                                        return (new_node, return_val);
                                    }
                                }
                                return (Node::Internal(internal), return_val);
                            },
                            ArtNodeInternalInner::Node48 { keys, children } => {
                                let (child_res, return_val) = mem::take(&mut children[child_pos]).recursive_delete(key, key_len, depth + 1);
                                children[child_pos] = child_res;
                                if children[child_pos].is_empty() {
                                    let c = key[depth];
                                    let pos = keys[c as usize] as usize;
                                    //let pos = child_pos + 1;
                                    keys[c as usize] = 0;
                                    children[pos - 1] = Node::Empty;

                                    header.num_children -= 1;

                                    if header.num_children == 12{

                                        let mut children_new: [Node<V>; 16] = [Node::INIT; 16];
                                        let mut keys_new: [u8; 16] = [0; 16];
                                        let mut child = 0;
                                        for i in 0..256 {
                                            let pos = keys[i] as usize;
                                            if pos != 0 {
                                                keys_new[child] = i as u8;
                                                children_new[child] = mem::take(&mut children[pos - 1]);
                                                child += 1;
                                            }
                                            //keys_new[i] = keys[i];
                                            //children_new[i] = mem::take(&mut children[i]);
                                        }

                                        let new_node  = Node::Internal(Box::new(ArtNodeInternal{ header: *header, inner: ArtNodeInternalInner::Node16 {
                                            keys: keys_new,
                                            children: children_new,
                                        } }));
                                        return (new_node, return_val);
                                    }
                                }
                                return (Node::Internal(internal), return_val);
                            },
                            ArtNodeInternalInner::Node256 { children } => {
                                let (child_res, return_val) = mem::take(&mut children[child_pos]).recursive_delete(key, key_len, depth + 1);
                                children[child_pos] = child_res;
                                if children[child_pos].is_empty() {
                                    header.num_children -= 1;

                                    // Resize to a node48 on underflow, not immediately to prevent
                                    // thrashing if we sit on the 48/49 boundary
                                    if header.num_children == 37 {
                                        let mut children_new = [Node::INIT; 48];
                                        let mut keys_new: [u8; 256] = [0; 256];

                                        let mut pos = 0;
                                        for i in 0..256 {
                                            if !children[i].is_empty() {
                                                children_new[pos] = mem::take(&mut children[i]);
                                                keys_new[i] = (pos + 1) as u8;
                                                pos += 1;
                                            }
                                        }

                                        let new_node = Node::Internal(Box::new(ArtNodeInternal{
                                            header: *header,
                                            inner: ArtNodeInternalInner::Node48 {
                                                keys: keys_new,
                                                children: children_new,
                                            },
                                        }));

                                        return (new_node, return_val);
                                    }
                                }

                                return (Node::Internal(internal), return_val);

                            }
                        }
                    }
                }
            }
            Node::Empty => return (self, None),
        };
        unreachable!()
    }


    /// Recursively iterates over the tree
    fn recursive_iter<CB>(&mut self, callback: &mut CB) -> bool
    where
        CB: FnMut(&V) -> bool
    {
        match self{
            Node::Leaf (leaf) => (callback)(&leaf.value),
            Node::Internal (internal) => internal.recursive_iter(callback),
            Node::Empty => true, // TODO: double check
        }
    }
}

impl<V> ArtNodeInternal<V> {
    fn find_child_mut(&mut self, c: u8) -> Option<&mut Node<V>> {
        let n = self.header;
        match &mut self.inner {
            ArtNodeInternalInner::Node4 { keys, children, .. } => {
                for i in 0..n.num_children as usize {
                    if keys[i] == c {
                        return Some(&mut children[i]);
                    }
                }
            }
            ArtNodeInternalInner::Node16 { keys, children } => {
                for i in 0..n.num_children as usize {
                    if keys[i] == c {
                        return Some(&mut children[i]);
                    }
                }
            }
            ArtNodeInternalInner::Node48 { keys, children } => {
                let idx = keys[c as usize]  as usize;
                if idx != 0 {
                    return Some(&mut children[idx - 1]);
                } else {
                    return None;
                }
            }
            ArtNodeInternalInner::Node256 { children } => {
                let node = &mut children[c as usize];
                if node.is_empty() {
                    return None;
                } else {
                    return Some(node);
                }
            }
        }
        return None;
    }

    fn find_child(&self, c: u8) -> Option<&Node<V>> {
        let n = self.header;
        match &self.inner {
            ArtNodeInternalInner::Node4 { keys, children, .. } => {
                for i in 0..n.num_children as usize {
                    if keys[i] == c {
                        return Some(&children[i]);
                    }
                }
            }
            ArtNodeInternalInner::Node16 { keys, children } => {
                for i in 0..n.num_children as usize {
                    if keys[i] == c {
                        return Some(&children[i]);
                    }
                }
            }
            ArtNodeInternalInner::Node48 { keys, children } => {
                let idx = keys[c as usize]  as usize;
                if idx != 0 {
                    return Some(&children[idx - 1]);
                } else {
                    return None;
                }
            }
            ArtNodeInternalInner::Node256 { children } => {
                return children.get(c as usize)
            }
        }
        return None;
    }

    fn find_child_index(&self, c: u8) -> Option<usize>{
        let n = self.header;
        match &self.inner {
            ArtNodeInternalInner::Node4 { keys,  .. } => {
                for i in 0..n.num_children as usize {
                    if keys[i] == c {
                        return Some(i);
                    }
                }
            }
            ArtNodeInternalInner::Node16 { keys, .. } => {
                for i in 0..min(16, n.num_children as usize) {
                    if keys[i] == c {
                        return Some(i);
                    }
                }
            }
            ArtNodeInternalInner::Node48 { keys, .. } => {
                let idx = keys[c as usize]  as usize;
                if idx != 0 {
                    return Some(idx - 1);
                }
            }
            ArtNodeInternalInner::Node256 { .. } => {
                return Some(c as usize);
            }
        }
        return None;
    }

    fn add_child(&mut self, c: u8, child: Node<V>) {
        let n = &mut self.header;

        match self.inner {
            ArtNodeInternalInner::Node4 { ref mut keys, ref mut children } => {
                if n.num_children < 4 {
                    let m = n.num_children;
                    let idx = keys.iter().position(|&key| c < key).unwrap_or(m as usize);
                    for i in (idx..m as usize).rev() {
                        keys[i+1] = keys[i];
                        children[i+1] = mem::replace(&mut children[i], Node::Empty);
                    }

                    keys[idx] = c;
                    children[idx] = child;
                    n.num_children += 1;
                } else {

                    let mut children_new: [Node<V>; 16] = [Node::<V>::INIT; 16];
                    let mut keys_new: [u8; 16] = [0; 16];
                    for i in 0..4 {
                        keys_new[i] = keys[i];
                        children_new[i] = mem::replace(&mut children[i], Node::Empty);
                    }

                    self.inner = ArtNodeInternalInner::Node16 {
                        keys: keys_new,
                        children: children_new,
                    };
                    self.add_child(c, child);
                }
            }
            ArtNodeInternalInner::Node16 { ref mut keys, ref mut children } => {
                if n.num_children < 16 {
                    let m = n.num_children as usize;
                    let idx = keys[0..m].iter().position(|&key| c < key).unwrap_or(m);
                    for i in (idx..m).rev() {
                        keys[i+1] = keys[i];
                        children[i+1] = mem::replace(&mut children[i], Node::Empty);
                    }

                    keys[idx] = c;
                    children[idx] = child;
                    n.num_children += 1;
                } else {

                    let mut children_new: [Node<V>; 48] = [Node::INIT; 48];
                    let mut keys_new: [u8; 256] = [0; 256];

                    for i in 0..16 {
                        keys_new[keys[i] as usize] = (i + 1) as u8;
                        children_new[i] = mem::replace(&mut children[i], Node::Empty);
                    }

                    self.inner = ArtNodeInternalInner::Node48 {
                        keys: keys_new,
                        children: children_new,
                    };
                    self.add_child(c, child);
                }
            }
            ArtNodeInternalInner::Node48 { ref mut keys, ref mut children } => {
                if n.num_children < 48 {
                    let pos = children.iter().position(|child| child.is_empty()).unwrap();
                    children[pos] = child;
                    keys[c as usize] = (pos + 1) as u8; // TODO: double check this
                    n.num_children += 1;
                } else {
                    // TODO: consider optimising this
                    let mut children_new: [Node<V>; 256] = [Node::INIT; 256];
                    for (i, &key) in keys.iter().enumerate() {
                        if key != 0 {
                            let idx = (key - 1) as usize;
                            children_new[i] = mem::replace(&mut children[idx], Node::Empty);
                        }
                    }

                    self.inner = ArtNodeInternalInner::Node256 {
                        children: children_new,
                    };
                    self.add_child(c, child);
                }
            }
            ArtNodeInternalInner::Node256 { ref mut children } => {
                n.num_children += 1;
                children[c as usize] = child;
            }
        }
    }

    fn minimum(&self) -> Option<&ArtNodeLeaf<V>>{
        match &self.inner{
            ArtNodeInternalInner::Node4 { children, .. } => children[0].minimum(),
            ArtNodeInternalInner::Node16 { children,.. } => children[0].minimum(),
            ArtNodeInternalInner::Node48 { keys, children,.. } => {
                let idx = keys.iter().position(|&key| key != 0).unwrap_or(48);
                let idx = (keys[idx] - 1) as usize;
                children[idx].minimum()
            },
            ArtNodeInternalInner::Node256 {children,.. } => {
                let idx = children.iter().position(|child| !child.is_empty());
                match idx{
                    None => None,
                    Some(i) => children[i].minimum(),
                }
            },
        }
    }

    fn minimum_mut(&mut self) -> Option<&mut ArtNodeLeaf<V>>{
        match &mut self.inner{
            ArtNodeInternalInner::Node4 { children, .. } => children[0].minimum_mut(),
            ArtNodeInternalInner::Node16 { children,.. } => children[0].minimum_mut(),
            ArtNodeInternalInner::Node48 { keys, children,.. } => {
                let idx = keys.iter().position(|&key| key != 0).unwrap_or(48);
                let idx = (keys[idx] - 1) as usize;
                children[idx].minimum_mut()
            },
            ArtNodeInternalInner::Node256 {children,.. } => {
                let idx = children.iter().position(|child| !child.is_empty());
                match idx{
                    None => None,
                    Some(i) => children[i].minimum_mut(),
                }
            },
        }
    }

    fn pop_first(&mut self) -> Option<(Box<[u8]>, V)>{
        match self.inner{
            ArtNodeInternalInner::Node4 { ref mut children, .. } => children[0].pop_first(),
            ArtNodeInternalInner::Node16 { ref mut children,.. } => children[0].pop_first(),
            ArtNodeInternalInner::Node48 { ref mut keys, ref mut children,.. } => {
                let idx = keys.iter().position(|&key| key != 0).unwrap_or(48);
                let idx = (keys[idx] - 1) as usize;
                children[idx].pop_first()
            },
            ArtNodeInternalInner::Node256 { ref mut children,.. } => {
                let idx = children.iter().position(|child| !child.is_empty());
                match idx{
                    None => None,
                    Some(i) => children[i].pop_first(),
                }
            },
        }
    }

    fn pop_last(&mut self) -> Option<(Box<[u8]>, V)>{
        let n = &self.header;
        match self.inner{
            ArtNodeInternalInner::Node4 { ref mut children, .. } => children[(n.num_children-1) as usize].pop_last(),
            ArtNodeInternalInner::Node16 { ref mut children,.. } => children[(n.num_children-1) as usize].pop_last(),
            ArtNodeInternalInner::Node48 { ref mut keys, ref mut children,.. } => {
                let idx = keys.iter().rev().position(|&key| key != 0).unwrap_or(0);
                let idx = (keys[idx] - 1) as usize;
                children[idx].pop_last()
            },
            ArtNodeInternalInner::Node256 { ref mut children,.. } => {
                let idx = children.iter().rev().position(|child| !child.is_empty());
                match idx{
                    None => None,
                    Some(i) => children[i].pop_last(),
                }
            },
        }
    }

    fn maximum(&self) -> Option<&ArtNodeLeaf<V>>{
        let n = &self.header;
        match &self.inner{
            ArtNodeInternalInner::Node4 { children, .. } => children[(n.num_children-1) as usize].maximum(),
            ArtNodeInternalInner::Node16 {  children,.. } => children[(n.num_children-1) as usize].maximum(),
            ArtNodeInternalInner::Node48 { keys, children,.. } => {
                let idx = keys.iter().rev().position(|&key| key != 0).unwrap_or(0);
                let idx = (keys[idx] - 1) as usize;
                children[idx].maximum()
            },
            ArtNodeInternalInner::Node256 { children,.. } => {
                let idx = children.iter().rev().position(|child| !child.is_empty());
                match idx{
                    None => None,
                    Some(i) => children[i].maximum(),
                }
            },
        }
    }

    fn maximum_mut(&mut self) -> Option<&mut ArtNodeLeaf<V>>{
        let n = &self.header;
        match &mut self.inner{
            ArtNodeInternalInner::Node4 { children, .. } => children[(n.num_children-1) as usize].maximum_mut(),
            ArtNodeInternalInner::Node16 { children,.. } => children[(n.num_children-1) as usize].maximum_mut(),
            ArtNodeInternalInner::Node48 { keys, children,.. } => {
                let idx = keys.iter().rev().position(|&key| key != 0).unwrap_or(0);
                let idx = (keys[idx] - 1) as usize;
                children[idx].maximum_mut()
            },
            ArtNodeInternalInner::Node256 { children,.. } => {
                let idx = children.iter().rev().position(|child| !child.is_empty());
                match idx{
                    None => None,
                    Some(i) => children[i].maximum_mut(),
                }
            },
        }
    }



    fn remove_child(mut self: Box<Self>, c: u8) -> (Node<V>, Option<V>) {
        // ASSERT: node to be removed is a leaf
        let n = &mut self.header;

        match self.inner {
            ArtNodeInternalInner::Node4 { ref mut keys, ref mut children } => {
                let pos = keys.iter().position(|&key| key == c);
                if pos.is_none() {
                    return (Node::Internal(self), None);
                }
                let pos = pos.unwrap();

                let return_val = mem::take(&mut children[pos]);
                let return_val = match return_val {
                    Node::Leaf(leaf) => Some(leaf.value),
                    _ => unreachable!(),
                };

                for i in pos+1..n.num_children as usize{
                    keys[i-1] = keys[i];
                    children[i-1] = mem::take(&mut children[i]);
                }

                if pos == 3 { // TODO: double check
                    keys[pos] = 0;
                    children[pos] = Node::Empty;
                }

                n.num_children -= 1;

                // Remove nodes with only a single child
                if n.num_children == 1 {
                    match mem::take(&mut children[0]){
                        Node::Internal(ref mut internal) => {
                            // Concatenate the prefixes
                            let mut prefix = n.partial_len;
                            if prefix < MAX_PREFIX_LEN{
                                n.partial[prefix] = keys[0];
                                prefix += 1;
                            }
                            if prefix < MAX_PREFIX_LEN{
                                let sub_prefix = min(internal.header.partial_len, MAX_PREFIX_LEN - prefix);
                                for i in 0..sub_prefix{
                                    n.partial[prefix+i] = internal.header.partial[i];
                                }
                                prefix += sub_prefix;
                            }

                            // Store the prefix in the child
                            for i in 0..min(prefix, MAX_PREFIX_LEN){
                                internal.header.partial[i] = internal.header.partial[i];
                            }
                            internal.header.partial_len += n.partial_len + 1;



                            //mem::replace(Node::Internal(self), internal);
                            // TODO: double check
                        },
                        Node::Leaf(leaf) => {
                            //mem::replace(self, leaf);
                        },
                        _ => unreachable!(),
                    }


                    //mem::replace(self, child);
                    return (Node::Internal(self), return_val);
                }

                return (Node::Internal(self), return_val);
            },
            _ => return (Node::Internal(self), None),
            /*
            ArtNodeInternalInner::Node16 { keys, children } => {
                let pos = keys.iter().position(|&key| key == c);
                if pos.is_none() {return;}
                let pos = pos.unwrap();

                for i in pos+1..n.num_children as usize{
                    keys[i-1] = keys[i];
                    children[i-1] = children[i].take();
                }

                if pos == 15 { // TODO: double check
                    keys[pos] = 0;
                    children[pos] = None;
                }

                n.num_children -= 1;

                if n.num_children == 3 {

                    let mut children_new: [Node<V>; 4] = [None; 4];
                    let mut keys_new: [u8; 4] = [0; 4];

                    for i in 0..n.num_children as usize {
                        keys_new[i] = keys[i];
                        children_new[i] = children[i].take();
                    }

                    self.inner = ArtNodeInternalInner::Node4 {
                        keys: keys_new,
                        children: children_new,
                    };
                }
            }
            ArtNodeInternalInner::Node48 { keys, children } => {
                let pos = keys[c as usize] as usize;
                keys[c as usize] = 0;
                children[pos - 1] = None;
                n.num_children -= 1;

                if n.num_children == 12{

                    let mut children_new: [Node<V>; 16] = [None; 16];
                    let mut keys_new: [u8; 16] = [0; 16];
                    let mut child = 0;
                    for i in 0..256 {
                        let pos = keys[i] as usize;
                        if pos != 0{
                            keys_new[child] = i as u8;
                            children_new[child] = children[pos - 1].take();
                            child += 1;
                        }
                        keys_new[i] = keys[i];
                        children_new[i] = children[i].take();
                    }

                    self.inner = ArtNodeInternalInner::Node16 {
                        keys: keys_new,
                        children: children_new,
                    };
                }
            }
            ArtNodeInternalInner::Node256 { children } => {
                children[c as usize] = None;
                n.num_children -= 1;

                // Resize to a node48 on underflow, not immediately to prevent
                // trashing if we sit on the 48/49 boundary
                if n.num_children == 37 {
                    let mut children_new = [None; 48];
                    let mut keys_new: [u8; 256] = [0; 256];

                    let mut pos = 0;
                    for i in 0..256 {
                        if children[i].is_some() {
                            children_new[pos] = children[i].take();
                            keys_new[i] = (pos + 1) as u8;
                            pos += 1;
                        }
                    }

                    self.inner = ArtNodeInternalInner::Node48 {
                        keys: keys_new,
                        children: children_new,
                    };
                }
            }
             */
        }
    }


    fn recursive_iter<CB>(&mut self, callback: &mut CB) -> bool
        where
            CB: FnMut(&V) -> bool
    {
        let n = self.header;
        match &mut self.inner{
            ArtNodeInternalInner::Node4 { children, .. } => {
                for child in children.iter_mut() {
                    if !child.is_empty(){
                        let result = child.recursive_iter(callback);
                        if result {
                            return result;
                        }
                    }
                }
            }
            ArtNodeInternalInner::Node16 { children, .. } => {
                for child in children.iter_mut() {
                    if !child.is_empty(){
                        let result = child.recursive_iter(callback);
                        if result {
                            return result;
                        }
                    }
                }
            }
            ArtNodeInternalInner::Node48 { keys, children, .. } => {
                for i in 0..256{
                    let idx = keys[i] as usize;
                    if idx != 0{
                        let result = children[idx-1].recursive_iter(callback);
                        if result{
                            return result;
                        }
                    }
                }
            }
            ArtNodeInternalInner::Node256 { children, .. } => {
                for child in children.iter_mut(){
                    if !child.is_empty() {
                        let result = child.recursive_iter(callback);
                        if result{
                            return result;
                        }
                    }
                }
            }
        }
        false
    }


}

impl<V> ArtNodeInternal<V>{
    /// Calculates the index at which the prefixes mismatch
    fn prefix_mismatch(&mut self, key: &[u8], key_len: usize, depth: usize) -> usize {
        let n = &self.header;
        let max_cmp = min(min(MAX_PREFIX_LEN, n.partial_len), key_len - depth);
        let idx = (0..max_cmp).into_iter().position(|i| n.partial[i] != key[depth+i]);
        if let Some(id) = idx {
            return id;
        }

        let idx = max_cmp;


        // If the prefix is short we can avoid finding a leaf
        if n.partial_len > MAX_PREFIX_LEN{
            // Prefix is longer than what we've checked, find a leaf
            let l = self.minimum().unwrap(); // TODO: check
            let max_cmp = min(l.key_len, key_len) - depth;
            for i in idx..max_cmp{
                if l.key[(i+depth)] != key[(depth+i)]{
                    return i;
                }
            }
        }

        return idx;
    }
}


impl InternalNodeHeader {
    /// Returns the number of prefix characters shared between
    /// the key and node.
    fn check_prefix(&self, key: &[u8], key_len: usize, depth: usize) -> usize{
        let max_cmp = min(min(self.partial_len, MAX_PREFIX_LEN), key_len -depth);
        for idx in 0..max_cmp{
            if self.partial[idx] != key[(depth + idx)]{
                return idx;
            }
        }
        return max_cmp;
    }
}

impl<V> ArtNodeLeaf<V> {
    fn new(key: &[u8], key_len: usize, value: V) -> Self{
        let mut key_clone = vec![0;key.len()];
        key_clone.copy_from_slice(key);
        Self{
            value,
            key_len,
            key: key_clone.into_boxed_slice(),
        }
    }

    /// Checks if a leaf matches
    /// @return 0 on success.
    fn matches(&self, key: &[u8], key_len: usize, _depth: usize) -> bool{
        if self.key_len != key_len {
            return false;
        }
        self.key == Box::from(key)
    }

    fn longest_common_prefix(&self, other: &mut Self, depth: usize) -> usize{
        let max_cmp = min(self.key_len, other.key_len) - depth;
        for idx in 0..max_cmp{
            if self.key[depth+idx] != other.key[depth+idx] {
                return idx;
            }
        }
        return max_cmp;
    }
}

impl<V> Node<V>{
    const INIT: Self = Node::Empty;

    fn is_empty(&self) -> bool {
        match self {
            Node::Empty => true,
            _ => false,
        }
    }
}
