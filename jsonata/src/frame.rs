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

    pub fn new_with_parent(parent: &Frame) -> Frame {
        Frame(Rc::new(RefCell::new(FrameData {
            bindings: HashMap::new(),
            parent: Some(parent.clone()),
        })))
    }

    pub fn bind(&self, name: &str, value: Value) {
        // Values in the frame need to be complete clones, otherwise modifying them would change their value.
        // Arrays and object will still point at the same members, and this replicates the reference semantics
        // in Javascript.
        let v = Value::new((*value).clone());
        self.0.borrow_mut().bindings.insert(name.to_string(), v);
    }

    pub fn lookup(&self, name: &str) -> Option<Value> {
        match self.0.borrow().bindings.get(name) {
            Some(value) => Some(*value),
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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn bind() {
//         let frame = Frame::new();
//         frame.bind("a", Value::number(1));
//         let a = frame.lookup("a");
//         assert!(a.is_some());
//         assert_eq!(a.unwrap(), 1);
//     }

//     #[test]
//     fn lookup_through_parent() {
//         let parent = Frame::new();
//         parent.bind("a", &arena.number(1));
//         let frame = Frame::new_with_parent(&parent);
//         let a = frame.lookup("a");
//         assert!(a.is_some());
//         assert_eq!(a.unwrap(), 1);
//     }

//     #[test]
//     fn lookup_overriding_parent() {
//         let parent = Frame::new();
//         let arena = ValueArena::new();
//         parent.bind("a", arena.clone(), &arena.number(1));
//         let frame = Frame::new_with_parent(&parent);
//         frame.bind("a", arena.clone(), &arena.number(2));
//         let a = frame.lookup("a");
//         assert!(a.is_some());
//         assert_eq!(a.unwrap(), 2);
//     }
// }
