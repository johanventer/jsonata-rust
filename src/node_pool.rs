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
            nodes: Vec::new(),
            free_list: Vec::new(),
        }
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

    pub fn get_mut(&mut self, index: usize) -> &mut Node {
        debug_assert!(index < self.nodes.len());
        &mut self.nodes[index]
    }
}

impl<Node: Sized + Default + PartialEq> Default for NodePool<Node> {
    fn default() -> Self {
        Self::new()
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
