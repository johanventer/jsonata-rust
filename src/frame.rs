use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::value::Value;

pub struct Frame(Rc<RefCell<FrameData>>);

impl Frame {
    pub fn new() -> Frame {
        Frame(Rc::new(RefCell::new(FrameData {
            bindings: HashMap::new(),
            parent: None,
        })))
    }

    pub fn new_with_parent(parent: Frame) -> Frame {
        Frame(Rc::new(RefCell::new(FrameData {
            bindings: HashMap::new(),
            parent: Some(parent),
        })))
    }

    pub fn bind(&self, name: &str, value: Value) {
        self.0.borrow_mut().bindings.insert(name.to_string(), value);
    }

    pub fn lookup(&self, name: &str) -> Option<Value> {
        match self.0.borrow().bindings.get(name) {
            Some(value) => Some(value.clone()),
            None => match &self.0.borrow().parent {
                Some(parent) => parent.lookup(name),
                None => None,
            },
        }
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Frame {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub struct FrameData {
    bindings: HashMap<String, Value>,
    parent: Option<Frame>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::ValuePool;

    #[test]
    fn bind() {
        let frame = Frame::new();
        let pool = ValuePool::new();
        frame.bind("a", Value::new_number(pool, 1));
        let a = frame.lookup("a");
        assert!(a.is_some());
        assert_eq!(a.unwrap(), 1);
    }

    #[test]
    fn lookup_through_parent() {
        let parent = Frame::new();
        let pool = ValuePool::new();
        parent.bind("a", Value::new_number(pool, 1));
        let frame = Frame::new_with_parent(parent);
        let a = frame.lookup("a");
        assert!(a.is_some());
        assert_eq!(a.unwrap(), 1);
    }

    #[test]
    fn lookup_overriding_parent() {
        let parent = Frame::new();
        let pool = ValuePool::new();
        parent.bind("a", Value::new_number(pool.clone(), 1));
        let frame = Frame::new_with_parent(parent);
        frame.bind("a", Value::new_number(pool, 2));
        let a = frame.lookup("a");
        assert!(a.is_some());
        assert_eq!(a.unwrap(), 2);
    }
}
