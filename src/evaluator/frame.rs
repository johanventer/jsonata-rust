use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::value::Value;

pub(crate) type Frame = Rc<RefCell<FrameData>>;

#[derive(Debug)]
pub struct FrameData {
    bindings: HashMap<String, Value>,
    parent: Option<Frame>,
}

impl FrameData {
    pub(crate) fn new() -> Frame {
        Rc::new(RefCell::new(FrameData {
            bindings: HashMap::new(),
            parent: None,
        }))
    }

    pub(crate) fn new_with_parent(parent: Frame) -> Frame {
        Rc::new(RefCell::new(FrameData {
            bindings: HashMap::new(),
            parent: Some(Rc::clone(&parent)),
        }))
    }

    pub(crate) fn bind(&mut self, name: &str, value: Value) {
        self.bindings.insert(name.to_string(), value);
    }

    pub(crate) fn lookup(&self, name: &str) -> Option<Value> {
        match self.bindings.get(name) {
            Some(value) => Some(value.clone()),
            None => match &self.parent {
                Some(parent) => parent.borrow().lookup(name),
                None => None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind() {
        let frame = FrameData::new();
        frame.borrow_mut().bind("a", Value::Number(1.0));
        let a = frame.borrow().lookup("a");
        assert!(a.is_some());
        assert_eq!(a.unwrap(), Value::Number(1.0));
    }

    #[test]
    fn lookup_through_parent() {
        let parent = FrameData::new();
        parent.borrow_mut().bind("a", Value::Number(1.0));
        let frame = FrameData::new_with_parent(parent);
        let a = frame.borrow().lookup("a");
        assert!(a.is_some());
        assert_eq!(a.unwrap(), Value::Number(1.0));
    }

    #[test]
    fn lookup_overriding_parent() {
        let parent = FrameData::new();
        parent.borrow_mut().bind("a", Value::Number(1.0));
        let frame = FrameData::new_with_parent(parent);
        frame.borrow_mut().bind("a", Value::Number(2.0));
        let a = frame.borrow().lookup("a");
        assert!(a.is_some());
        assert_eq!(a.unwrap(), Value::Number(2.0));
    }
}
