use std::ptr::null_mut;
use std::cmp::min;
use std::borrow::Borrow;

const MAX_PREFIX_LEN: usize = 10;

type ArtError = u32;
type Result<T> = std::result::Result<T, ArtError>;

#[derive(Clone)]
enum Node{
    Leaf{
        leaf: ArtNodeLeaf,
    },
    Internal{
        internal: ArtNodeInternal,
    }
}

#[derive(Copy, Clone)]
struct InternalNodeHeader {
    partial_len: usize,
    num_children: u8,
    partial: [u8; MAX_PREFIX_LEN],
}

#[derive(Debug,Clone)]
pub struct ArtNodeLeaf{
    pub value: u32, // TODO: make arbitrary
    key_len: usize,
    key: Box<[u8]>,
}

#[derive(Clone)]
struct ArtNodeInternal{
    header: InternalNodeHeader,
    inner: ArtNodeInternalInner,
}

#[derive(Clone)]
enum ArtNodeInternalInner {
    Node4 {
        keys: [u8;4],
        children: [MyNode;4],
    },
    Node16 {
        keys: [u8;16],
        children: [MyNode;16],
    },
    Node48 {
        keys: [u8;256],
        children: [MyNode;48],
    },
    Node256 {
        children: [MyNode;256],
    },
}

type MyNode = Option<Box<Node>>;

pub struct ArtTree{
    root: MyNode,
    size: u64,
}

impl ArtTree{
    pub fn new() -> Self{
        Self{
            root: None,
            size: 0
        }
    }


    /// Searches for a value in the ART tree
    /// @arg t The tree
    /// @arg key The key
    /// @arg key_len The length of the key
    /// @return NULL if the item was not found, otherwise
    /// the value pointer is returned.
    pub fn search(&mut self, key: &[u8], key_len: usize) -> Option<u32>{
        let mut n_iter = self.root.as_mut();
        let mut depth = 0;
        while let Some(node) = n_iter {
            match **node {
                Node::Leaf { ref leaf} => {
                    if leaf.matches(key, key_len, depth) {
                        return Some(leaf.value);
                    }
                    return None;
                }
                Node::Internal { ref mut internal } => {
                    let header = internal.header;

                    if header.partial_len != 0 {

                        let prefix_len = header.check_prefix( key, key_len, depth);
                        if prefix_len != min(MAX_PREFIX_LEN, header.partial_len) {
                            return None;
                        }
                        depth = depth + header.partial_len;
                    }

                    n_iter = internal.find_child(key[depth]);
                    depth+=1;
                }
            }
        }
        None
    }

    pub fn minimum(&mut self) -> Option<&mut ArtNodeLeaf>{
        match &mut self.root {
            None => None,
            Some(node) => node.minimum(),
        }
    }

    pub fn maximum(&mut self) -> Option<&mut ArtNodeLeaf>{
        match &mut self.root {
            None => None,
            Some(node) => node.maximum(),
        }
    }


    /// inserts a new value into the art tree
    /// @arg t the tree
    /// @arg key the key
    /// @arg key_len the length of the key
    /// @arg value opaque value.
    /// @return null if the item was newly inserted, otherwise
    /// the old value pointer is returned.
    pub fn insert(&mut self, key: &[u8], key_len: usize, value: u32) -> Option<u32> {
        match &mut self.root{
            None => {
                self.root = Some(Box::new(Node::Leaf { leaf: ArtNodeLeaf::new(key, key_len, value) }));
                self.size += 1;
                None
            }
            Some(node) => {
                let result = node.recursive_insert(key, key_len, value, 0, true);
                if result.is_none() {
                    self.size += 1;
                }
                result
            },
        }
    }

    /// Iterates through the entries pairs in the map,
    /// invoking a callback for each. The call back gets a
    /// key, value for each and returns an integer stop value.
    /// If the callback returns non-zero, then the iteration stops.
    /// @arg t The tree to iterate over
    /// @arg cb The callback function to invoke
    /// @return true on success, or the return of the callback.
    pub fn iter<CB>(&mut self, mut callback: CB) -> bool
        where
            CB: FnMut(u32) -> bool
    {
        self.root.as_mut().map_or(false, |root| root.recursive_iter(&mut callback))
    }
}



impl Node{
    fn minimum(&mut self) -> Option<&mut ArtNodeLeaf> {
        match self{
            Node::Leaf { leaf } => Some(leaf),
            Node::Internal { internal } => internal.minimum(),
        }
    }

    fn maximum(&mut self) -> Option<&mut ArtNodeLeaf> {
        match self{
            Node::Leaf { leaf } => Some(leaf),
            Node::Internal { internal } => internal.maximum(),
        }
    }

    fn recursive_insert(&mut self, key: &[u8], key_len: usize, value: u32, depth: usize, replace: bool) -> Option<u32> {
        // TODO: handle NULL case
        match self {
            Node::Leaf { leaf } => {
                // Check if we are updating an existing value
                if leaf.matches(key, key_len, depth){
                    // TODO: use replace here?
                    let old = leaf.value;
                    leaf.value = value;
                    return Some(old);
                }

                // New value, we must split the leaf into a node4


                let mut new_leaf = ArtNodeLeaf::new(key, key_len, value);

                let longest_prefix = leaf.longest_common_prefix(&mut new_leaf, depth);


                let mut partial = [0u8; MAX_PREFIX_LEN];
                let key_slice = &key[depth..(depth+min(MAX_PREFIX_LEN,longest_prefix))];
                for (i,&v ) in key_slice.iter().enumerate(){
                    partial[i] = v;
                }
                const INIT: MyNode = None;

                let mut internal = ArtNodeInternal {
                    header: InternalNodeHeader {
                        partial_len: longest_prefix,
                        num_children: 0,
                        partial,
                    },
                    inner: ArtNodeInternalInner::Node4 { keys: [0u8;4], children: [INIT;4] },
                };

                let old_leaf = Node::Leaf {
                    leaf: ArtNodeLeaf::new(&*leaf.key, leaf.key_len, leaf.value)
                };


                internal.add_child(leaf.key[(depth+longest_prefix)], Box::new(old_leaf));
                internal.add_child(new_leaf.key[(depth+longest_prefix)], Box::new(Node::Leaf {leaf: new_leaf }));

                *self = Node::Internal {
                    internal
                };

                return None;
            }
            Node::Internal { internal } => {
                let mut n = internal.header;

                // Check if given node has a prefix
                if n.partial_len != 0{
                    // Determine if the prefixes differ, since we need to split
                    let prefix_diff = internal.prefix_mismatch(key, key_len, depth);
                    if prefix_diff >= n.partial_len{
                        return self.recursive_insert(key, key_len, value, depth + n.partial_len, replace);
                    }

                    // Create a new node
                    let mut partial = [0u8; MAX_PREFIX_LEN];
                    for i in 0..min(MAX_PREFIX_LEN, prefix_diff){
                        partial[i] = n.partial[i];
                    }

                    const INIT: MyNode = None;
                    let mut new_node = ArtNodeInternal {
                        header: InternalNodeHeader {
                            partial_len: prefix_diff,
                            num_children: 0,
                            partial,
                        },
                        inner: ArtNodeInternalInner::Node4 { keys: [0u8;4], children: [INIT;4] },
                    };

                    // Adjust the prefix of the old node
                    if n.partial_len <= MAX_PREFIX_LEN {
                        new_node.add_child(n.partial[prefix_diff], Box::new(Node::Internal {internal: ArtNodeInternal{ header: n, inner: internal.inner.clone() }}));
                        n.partial_len -= prefix_diff + 1;
                        for i in (0..min(MAX_PREFIX_LEN,n.partial_len)).rev(){
                            n.partial[i] = n.partial[prefix_diff+1+i];
                        }
                    } else {
                        n.partial_len -= prefix_diff + 1;
                        let l = internal.minimum().unwrap();
                        for i in 0..min(MAX_PREFIX_LEN,n.partial_len){
                            n.partial[i] = l.key[depth+prefix_diff+1+i];
                        }
                        new_node.add_child(l.key[depth+prefix_diff], Box::new(Node::Internal {internal: ArtNodeInternal{ header: n, inner: internal.inner.clone() }}));
                    }




                    let new_leaf = ArtNodeLeaf::new(key, key_len, value);
                    new_node.add_child(key[depth+prefix_diff], Box::new(Node::Leaf {leaf: new_leaf}));

                    *self = Node::Internal { internal: new_node };

                    return None;
                }

                let child = internal.find_child(key[depth]);
                if let Some(node) = child{
                    return node.recursive_insert(key, key_len, value, depth, replace);
                }

                let new_leaf = Node::Leaf {leaf: ArtNodeLeaf::new(key, key_len, value)};
                internal.add_child(key[depth], Box::new(new_leaf));

                return None;
            }
        }

        None
    }


    /// Recursively iterates over the tree
    fn recursive_iter<CB>(&mut self, callback: &mut CB) -> bool
    where
        CB: FnMut(u32) -> bool
    {
        match self{
            Node::Leaf { leaf, .. } => (callback)(leaf.value),
            Node::Internal { internal } => internal.recursive_iter(callback),
        }
    }
}

impl ArtNodeInternal {
    fn find_child(&mut self, c: u8) -> Option<&mut Box<Node>> {
        let n = self.header;
        match &mut self.inner {
            ArtNodeInternalInner::Node4 { keys, children, .. } => {
                for i in 0..4 {
                    if keys[i] == c {
                        return children[i].as_mut();
                    }
                }
            }
            ArtNodeInternalInner::Node16 { keys, children } => {
                for i in 0..min(16, n.num_children as usize) {
                    if keys[i] == c {
                        return children[i].as_mut();
                    }
                }
            }
            ArtNodeInternalInner::Node48 { keys, children } => {
                let idx = (keys[c as usize] - 1)  as usize;
                if idx != 0 {
                    return children[idx].as_mut();
                }
            }
            ArtNodeInternalInner::Node256 { children } => {
                return children[c as usize].as_mut();
            }
        }
        return None;
    }


    fn add_child(&mut self, c: u8, child: Box<Node>) {
        let n = &mut self.header;

        match &mut self.inner {
            ArtNodeInternalInner::Node4 { keys, children } => {
                if n.num_children < 4 {
                    let m = n.num_children;
                    let idx = keys.iter().position(|&key| c < key).unwrap_or(m as usize);
                    //keys.copy_within(idx..m, idx+1);
                    for i in (idx..m as usize).rev() {
                        keys[i+1] = keys[i];
                        children[i+1] = children[i].take();
                    }

                    keys[idx] = c;
                    children[idx] = Some(child);
                    n.num_children += 1;
                } else {
                    const INIT: MyNode = None;
                    let mut children_new: [MyNode; 16] = [INIT; 16];
                    let mut keys_new: [u8; 16] = [0; 16];
                    for i in 0..4 {
                        keys_new[i] = keys[i];
                        children_new[i] = children[i].take();
                    }

                    self.inner = ArtNodeInternalInner::Node16 {
                        keys: keys_new,
                        children: children_new,
                    };
                    self.add_child(c, child);
                }
            }
            ArtNodeInternalInner::Node16 { keys, children } => {
                if n.num_children < 16 {
                    let m = n.num_children;
                    let idx = keys.iter().position(|&key| c < key).unwrap_or(m as usize);
                    //keys.copy_within(idx..m, idx+1);
                    for i in (idx..m as usize).rev() {
                        keys[i] = keys[i - 1];
                        children[i] = children[i - 1].take();
                    }

                    keys[idx] = c;
                    children[idx] = Some(child);
                    n.num_children += 1;
                } else {
                    const INIT: MyNode = None;
                    let mut children_new: [MyNode; 48] = [INIT; 48];
                    let mut keys_new: [u8; 256] = [0; 256];

                    for i in 0..16 {
                        keys_new[keys[i] as usize] = (i + 1) as u8;
                        children_new[i] = children[i].take();
                    }

                    self.inner = ArtNodeInternalInner::Node48 {
                        keys: keys_new,
                        children: children_new,
                    };
                    self.add_child(c, child);
                }
            }
            ArtNodeInternalInner::Node48 { keys, children } => {
                if n.num_children < 48 {
                    let pos = children.iter().position(|child| child.is_none()).unwrap();
                    children[pos] = Some(child);
                    keys[c as usize] = (pos + 1) as u8; // TODO: double check this
                    n.num_children += 1;
                } else {
                    // TODO: consider optimising this
                    const INIT: MyNode = None;
                    let mut children_new: [MyNode; 256] = [INIT; 256];
                    for (i, &key) in keys.iter().enumerate() {
                        if key != 0 {
                            children_new[i] = children[(key - 1) as usize].take();
                        }
                    }

                    self.inner = ArtNodeInternalInner::Node256 {
                        children: children_new,
                    };
                    self.add_child(c, child);
                }
            }
            ArtNodeInternalInner::Node256 { children } => {
                n.num_children += 1;
                children[c as usize] = Some(child);
            }
        }
    }

    fn minimum(&mut self) -> Option<&mut ArtNodeLeaf>{
        let n = &self.header;
        match &mut self.inner{
            ArtNodeInternalInner::Node4 { children, .. } => children[0].as_mut(),
            ArtNodeInternalInner::Node16 { children,.. } => children[0].as_mut(),
            ArtNodeInternalInner::Node48 { keys, children,.. } => {
                let idx = keys.iter().position(|&key| key != 0).unwrap_or(48);
                let idx = (keys[idx] - 1) as usize;
                children[idx].as_mut()
            },
            ArtNodeInternalInner::Node256 {  children,.. } => {
                let idx = children.iter().position(|child| child.is_some());
                match idx{
                    None => None,
                    Some(i) => children[i].as_mut(),
                }
            },
        }.and_then(|next|next.minimum())
    }

    fn maximum(&mut self) -> Option<&mut ArtNodeLeaf>{
        let n = &self.header;
        match &mut self.inner{
            ArtNodeInternalInner::Node4 {  children, .. } => children[(n.num_children-1) as usize].as_mut(),
            ArtNodeInternalInner::Node16 {children,.. } => children[(n.num_children-1) as usize].as_mut(),
            ArtNodeInternalInner::Node48 { keys, children,.. } => {
                let idx = keys.iter().rev().position(|&key| key != 0).unwrap_or(0);
                let idx = (keys[idx] - 1) as usize;
                children[idx].as_mut()
            },
            ArtNodeInternalInner::Node256 {  children,.. } => {
                let idx = children.iter().rev().position(|child| child.is_some());
                match idx{
                    None => None,
                    Some(i) => children[i].as_mut(),
                }
            },
        }.and_then(|next|next.maximum())
    }


    fn recursive_iter<CB>(&mut self, callback: &mut CB) -> bool
        where
            CB: FnMut(u32) -> bool
    {
        let n = self.header;
        match &mut self.inner{
            ArtNodeInternalInner::Node4 { children, .. } => {
                for child in children.iter_mut() {
                    if child.is_some(){
                        let result = child.as_mut().unwrap().recursive_iter(callback);
                        if result {
                            return result;
                        }
                    }
                }
            }
            ArtNodeInternalInner::Node16 { children, .. } => {
                for child in children.iter_mut() {
                    if child.is_some(){
                        let result = child.as_mut().unwrap().recursive_iter(callback);
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
                        let result = children[idx-1].as_mut().unwrap().recursive_iter(callback);
                        if result{
                            return result;
                        }
                    }
                }
            }
            ArtNodeInternalInner::Node256 { children, .. } => {
                for child in children.iter_mut(){
                    if child.is_some() {
                        let result = child.as_mut().unwrap().recursive_iter(callback);
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

impl ArtNodeInternal{
    /// Calculates the index at which the prefixes mismatch
    fn prefix_mismatch(&mut self, key: &[u8], key_len: usize, depth: usize) -> usize{
        let max_cmp = min(min(MAX_PREFIX_LEN, self.header.partial_len), key_len - depth);
        let idx = (0..max_cmp).into_iter().position(|i| self.header.partial[i] != key[(depth +i)]).unwrap_or(MAX_PREFIX_LEN);

        // If the prefix is short we can avoid finding a leaf
        if self.header.partial_len > MAX_PREFIX_LEN{
            // Prefix is longer than what we've checked, find a leaf
            let l = self.minimum().unwrap(); // TODO: check
            let max_cmp = (min(l.key_len, key_len) - depth);
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

impl ArtNodeLeaf {

    fn new(key: &[u8], key_len: usize, value: u32) -> Self{
        //let mut key_clone = Vec::with_capacity(key.len());
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

    fn longest_common_prefix(&mut self, other: &mut Self, depth: usize) -> usize{
        let max_cmp = min(self.key_len, other.key_len) - depth;
        for idx in depth..max_cmp{
            if self.key[idx] != other.key[idx] {
                return idx;
            }
        }
        return max_cmp;
    }
}
