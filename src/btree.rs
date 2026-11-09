//! A mediocre B-Tree implentation since we can't use the standard library's.

use std::fmt::Debug;

/// [`BTree`] t-value.
///
/// We choose 6 to match the [standard library](https://doc.rust-lang.org/src/alloc/collections/btree/node.rs.html#43).
/// Their choice of `B` is informed by benchmarks.
///
/// Although we'd love to have calculated `B` based on page size, the calculation is trickier since
/// our data is generic and we don't exactly have child pointers.
const B: usize = 6;

#[derive(Debug)]
struct Node<T: Ord> {
    keys: Vec<T>,
    children: Vec<Node<T>>,
    is_leaf: bool,
}

impl<T: Ord> Node<T> {
    fn new(is_leaf: bool) -> Self {
        Self {
            keys: Vec::new(),
            children: Vec::new(),
            is_leaf,
        }
    }
}

#[derive(Debug)]
pub struct BTree<T: Ord> {
    root: Option<Node<T>>,
    size: usize,
}

impl<T: Ord> BTree<T> {
    #[inline]
    #[must_use]
    pub fn new() -> Self { Self::default() }

    /// Determine whether this [`BTree`] contains `key`.
    pub fn contains(&self, key: &T) -> bool {
        match &self.root {
            Some(root) => self.search(root, key),
            None => false,
        }
    }

    /// Search for a key in this [`BTree`].
    fn search(&self, node: &Node<T>, key: &T) -> bool {
        let i = node.keys.partition_point(|k| k < key);

        if i < node.keys.len() && &node.keys[i] == key {
            return true
        }

        if node.is_leaf {
            return false
        }

        self.search(&node.children[i], key)
    }

    /// Insert `key` into a [`Node`] in this [`BTree`].
    pub fn insert(&mut self, key: T) {
        // Ignore duplicates.
        if self.contains(&key) { return }

        self.size += 1;
        let max_keys = 2 * B - 1;

        if self.root.is_none() {
            let mut root = Node::new(true);
            root.keys.push(key);
            self.root = Some(root);
            return
        }

        // Split root if full.
        if self.root.as_ref().unwrap().keys.len() == max_keys {
            let old_root = self.root.take().unwrap();

            let mut new_root = Node::new(false);
            new_root.children.push(old_root);

            Self::split_child(&mut new_root, 0);

            self.root = Some(new_root);
        }

        let root = self.root.as_mut().unwrap();
        Self::insert_non_full(root, key);
    }

    /// Insert into a node that is not full.
    fn insert_non_full(node: &mut Node<T>, key: T) {
        let mut i = node.keys.partition_point(|k| k < &key);

        if node.is_leaf {
            node.keys.insert(i, key);
            return
        }

        // Split full child before descending.
        if node.children[i].keys.len() == 2 * B - 1 {
            Self::split_child(node, i);

            if key > node.keys[i] { i += 1 }
        }

        Self::insert_non_full(&mut node.children[i], key);
    }

    /// Split a full child into two nodes.
    fn split_child(parent: &mut Node<T>, index: usize) {
        let child = &mut parent.children[index];

        let mut right = Node::new(child.is_leaf);

        // Median key moves into parent.
        let median = child.keys.remove(B - 1);

        // Move keys greater than median into right node.
        right.keys = child.keys.split_off(B - 1);

        // Move children if this is an internal node.
        if !child.is_leaf {
            right.children = child.children.split_off(B);
        }

        parent.keys.insert(index, median);
        parent.children.insert(index + 1, right);
    }

    /// Inorder traversal of the [`BTree`].
    pub fn inorder(&self) -> Vec<&T> {
        let mut result = Vec::with_capacity(self.size);

        if let Some(root) = &self.root {
            Self::inorder_inner(root, &mut result);
        }

        result
    }

    /// Private helper for [`Self::inorder()`].
    fn inorder_inner<'a>(node: &'a Node<T>, result: &mut Vec<&'a T>) {
        for i in 0..node.keys.len() {
            if !node.is_leaf {
                Self::inorder_inner(&node.children[i], result);
            }

            result.push(&node.keys[i]);
        }

        if !node.is_leaf {
            Self::inorder_inner(
                &node.children[node.keys.len()],
                result,
            );
        }
    }
}

impl<T: Ord> Default for BTree<T> {
    fn default() -> Self {  Self { root: None, size: 0 } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_functionality() {
        let mut tree = BTree::new();

        for value in (0x00..=0xFF).rev() {
            tree.insert(value);
        }

        assert! { tree.contains(&0x42) }
        assert! { tree.contains(&0xFF) }

        assert! { tree.inorder().is_sorted() }

        assert_eq! { tree.size, 256 }
    }
}
