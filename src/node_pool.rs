use std::{marker::PhantomData, ops::Deref};

/// A flat array of nodes with a free list.
///
/// A `NodePool` is a flat array of generic `Node` instances, which is used to model
/// tree structures in both AST generation and evaluation.
///
/// When an index is removed from the pool, its index is add to a free list.
/// Subsequent inserts into the pool will use free indices until there are no more
/// available before expanding the underlying `Vec` being used for storage.
#[derive(Debug)]
pub struct NodePool<Node: Sized + PartialEq> {
    nodes: Vec<Node>,
    free_list: Vec<usize>,
}

impl<Node: Sized + PartialEq> NodePool<Node> {
    pub fn new() -> Self {
        NodePool {
            nodes: Vec::with_capacity(10),
            free_list: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Node> {
        self.nodes.iter()
    }

    pub fn insert(&mut self, node: Node) -> usize {
        let index = self.free_list.pop().unwrap_or(self.nodes.len());
        if index == self.nodes.len() {
            self.nodes.push(node);
        } else {
            self.nodes[index] = node;
        }
        index
    }

    pub fn remove(&mut self, index: usize) {
        self.free_list.push(index);
    }

    pub fn get(&self, index: usize) -> &Node {
        debug_assert!(index < self.nodes.len());
        &self.nodes[index]
    }

    pub fn get_ref(&self, index: usize) -> NodeRef<Node> {
        debug_assert!(index < self.nodes.len());
        NodeRef {
            pool: self,
            index,
            _marker: PhantomData,
        }
    }

    pub fn get_mut(&mut self, index: usize) -> &mut Node {
        debug_assert!(index < self.nodes.len());
        &mut self.nodes[index]
    }
}

impl<Node: Sized + PartialEq> Default for NodePool<Node> {
    fn default() -> Self {
        Self::new()
    }
}

/// A wrapper for a reference to a node in the pool.
///
/// As the pool grows the array of nodes will reallocate, so we can't keep a pointer
/// around to a particular node as it will become invalid.
///
/// The pool is wrapped in Rc<RefCell> in the usage code, which makes it difficult
/// to directly hand out a reference to a node, as it requires the pool to be borrowed.
///
/// To get around the borrow checker, the NodeRef stores a pointer to the pool and
/// the index of the node, and when dereferenced it returns a reference to the actual
/// node in the nodes array.
///
/// The only potential issue here is if you happen to remove the the indexed node
/// from the pool while holding a NodeRef you will still get a valid node, it just
/// won't be the one that you expect.
pub struct NodeRef<'pool, Node: 'pool + Sized + PartialEq> {
    pool: *const NodePool<Node>,
    index: usize,
    _marker: PhantomData<&'pool NodePool<Node>>,
}

impl<'pool, Node: 'pool + Sized + PartialEq> Deref for NodeRef<'pool, Node> {
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        // Safety: The NodeRef is tied to the lifetime of the pool.
        let pool = unsafe { &*self.pool };
        pool.get(self.index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert() {
        let mut tree: NodePool<i32> = NodePool::new();
        let one = tree.insert(1);
        let two = tree.insert(2);
        let three = tree.insert(3);
        assert_eq!(one, 0);
        assert_eq!(two, 1);
        assert_eq!(three, 2);
        assert_eq!(tree.get(one), &1);
        assert_eq!(tree.get(two), &2);
        assert_eq!(tree.get(three), &3);
    }

    #[test]
    fn remove() {
        let mut tree: NodePool<i32> = NodePool::new();
        tree.insert(1);
        let two = tree.insert(2);
        tree.insert(3);
        tree.remove(two);
        let four = tree.insert(4);
        assert_eq!(four, two);
        assert_eq!(tree.get(four), &4);
        let five = tree.insert(5);
        assert_eq!(five, 3);
        assert_eq!(tree.get(five), &5);
    }
}
