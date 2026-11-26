//! A mediocre B-Tree implementation since we can't use the standard library's.

use std::fmt::Debug;

/// [`BTree`] t-value.
///
/// We choose 6 to match the [standard library](https://doc.rust-lang.org/src/alloc/collections/btree/node.rs.html#43).
/// Their choice of `B` is informed by benchmarks.
///
/// Although we'd love to have calculated `B` based on page size, the calculation is trickier since
/// our data is generic and we don't exactly have child pointers.
const B: usize = 6;

/// A node in this [`BTree`].
#[derive(Debug)]
struct Node<T: Ord> {
    keys: Vec<T>,
    children: Vec<Self>,
    is_leaf: bool,
}

impl<T: Ord> Node<T> {
    /// Create a new [`Node`].
    const fn new(is_leaf: bool) -> Self {
        Self {
            keys: Vec::new(),
            children: Vec::new(),
            is_leaf,
        }
    }
}

/// A [`BTree`] of degree [`B`].
#[derive(Debug)]
pub struct BTree<T: Ord> {
    root: Option<Node<T>>,
    size: usize,
}

impl<T: Ord> BTree<T> {
    /// Create a new [`BTree`].
    #[inline]
    #[must_use]
    pub fn new() -> Self { Self::default() }

    #[inline]
    pub fn len(&self) -> usize { self.size }

    #[inline]
    pub fn is_empty(&self) -> bool { self.size == 0 }

    /// Determine whether this [`BTree`] contains `key`.
    pub fn contains(&self, key: &T) -> bool {
        self.root.as_ref().is_some_and(|root| Self::search(root, key))
    }

    /// Search for a key in this [`BTree`].
    fn search(node: &Node<T>, key: &T) -> bool {
        let i = node.keys.partition_point(|k| k < key);

        if i < node.keys.len() && &node.keys[i] == key {
            return true
        }

        if node.is_leaf {
            return false
        }

        Self::search(&node.children[i], key)
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

    /// Take the maximum key from the subtree starting at `node`.
    fn take_maximum(node: &mut Node<T>) -> T {
        if node.is_leaf {
            return node.keys.pop().unwrap();
        }

        let index = node.children.len() - 1;
        Self::take_maximum(&mut node.children[index])
    }

    /// Take the minimum key from the subtree starting at `node`.
    fn take_minimum(node: &mut Node<T>) -> T {
        if node.is_leaf {
            return node.keys.remove(0);
        }

        Self::take_minimum(&mut node.children[0])
    }

    /// Borrow a key from the previous node.
    fn borrow_from_previous(node: &mut Node<T>, index: usize) {
        // Avoid multiple mutable borrows.
        let (lhs, rhs) = node.children.split_at_mut(index);
        let sibling = &mut lhs[index-1];
        let child = &mut rhs[0];

        // Move the parent's separator into the child.
        child.keys.insert(0, std::mem::replace(&mut node.keys[index-1], sibling.keys.pop().unwrap()));

        // If we're dealing with internal nodes, move the sibling's rightmost child to the beginning
        // of the borrowing child.
        if !sibling.is_leaf {
            child.children.insert(0, sibling.children.pop().unwrap());
        }
    }

    /// Borrow a key from the next node.
    fn borrow_from_next(node: &mut Node<T>, index: usize) {
        // Avoid multiple mutable borrows.
        let (lhs, rhs) = node.children.split_at_mut(index+1);
        let child = &mut lhs[index];
        let sibling = &mut rhs[0];

        // Move the parent's separator into the child.
        child.keys.push(std::mem::replace(&mut node.keys[index], sibling.keys.remove(0)));

        // If we're dealing with internal nodes, move the sibling's leftmost child to the end of the
        // borrowing child.
        if !sibling.is_leaf {
            child.children.push(sibling.children.remove(0));
        }
    }

    /// Merge two nodes.
    fn merge(node: &mut Node<T>, index: usize) {
        // Avoid multiple mutable borrows.
        let (lhs, rhs) = node.children.split_at_mut(index+1);
        let child = &mut lhs[index];
        let sibling = &mut rhs[0];

        // Move the parent's separator into the child.
        child.keys.push(node.keys.remove(index));

        // Move all keys from the sibling.
        child.keys.append(&mut sibling.keys);

        // Move all children if we're dealing with internal nodes.
        if !sibling.is_leaf {
            child.children.append(&mut sibling.children);
        }

        // Remove the sibling we just emptied.
        node.children.remove(index+1);
    }

    /// Remove a key from a leaf node.
    fn remove_from_leaf(node: &mut Node<T>, index: usize) -> T {
        node.keys.remove(index)
    }

    /// Remove a key from an internal node, handling borrowing and merging.
    fn remove_from_internal(node: &mut Node<T>, index: usize) -> T {
        if node.children[index].keys.len() >= B {
            // Replace with predecessor
            let predecessor = Self::take_maximum(&mut node.children[index]);
            std::mem::replace(&mut node.keys[index], predecessor)
        } else if node.children[index+1].keys.len() >= B {
            // Replace with successor
            let successor = Self::take_minimum(&mut node.children[index+1]);
            std::mem::replace(&mut node.keys[index], successor)
        } else {
            // Merge
            Self::merge(node, index);
            Self::remove_inner(&mut node.children[index], &node.keys[index]).unwrap()
        }
    }

    /// Remove a `key` from this [`BTree`].
    pub fn remove(&mut self, key: &T) -> Option<T> {
        let root = self.root.as_mut()?;
        let removed = Self::remove_inner(root, key);

        if removed.is_some() {
            self.size -= 1;

            // Handle the case where we've removed all of root's keys.
            if self.root.as_ref().unwrap().keys.is_empty() {
                let root = self.root.take().unwrap();

                self.root = match root.is_leaf {
                    true => None,
                    false => Some(root.children.into_iter().next().unwrap())
                }
            }
        }

        removed
    }

    fn remove_from_any(node: &mut Node<T>, index: usize) -> T {
        match node.is_leaf {
            true => Self::remove_from_leaf(node, index),
            false => Self::remove_from_internal(node, index),
        }
    }

    /// Private helper for [`Self::remove()`].
    fn remove_inner(node: &mut Node<T>, key: &T) -> Option<T> {
        let index = node.keys.partition_point(|k| k < key);

        // Base case where the key exists in this node.
        if index < node.keys.len() && &node.keys[index] == key {
            return Some(Self::remove_from_any(node, index))
        }

        // Base case where the key is not found :(
        if node.is_leaf {
            return None
        }

        // Ensure the child has sufficient keys before recursing.
        let mut child_index = index;
        if node.children[child_index].keys.len() == B - 1 {
            if child_index > 0 && node.children[child_index-1].keys.len() >= B {
                Self::borrow_from_previous(node, child_index);
            } else if child_index < node.keys.len() && node.children[child_index+1].keys.len() >= B {
                Self::borrow_from_next(node, child_index);
            } else if child_index < node.keys.len() {
                Self::merge(node, child_index);
            } else {
                child_index -= 1;
                Self::merge(node, child_index);
            }
        }

        // Recursively search the appropriate child.
        Self::remove_inner(&mut node.children[child_index], key)
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

        assert! { tree.remove(&0xAB).is_some() }
        assert! { tree.remove(&0xAB).is_none() }

        for value in 0x00..=0xFF {
            tree.remove(&value);
        }

        assert_eq! { tree.inorder().as_slice(), [&0i32; 0] }
    }
}
