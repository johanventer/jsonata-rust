use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::new_value::{Value, ValuePtr};

#[derive(Debug)]
pub struct FrameData<'arena> {
    bindings: HashMap<&'arena str, Value<'arena>>,
    parent: Option<Frame<'arena>>,
}

#[derive(Debug)]
pub struct Frame<'arena>(Rc<RefCell<FrameData<'arena>>>);

impl<'arena> Frame<'arena> {
    pub fn new() -> Frame<'arena> {
        Frame(Rc::new(RefCell::new(FrameData {
            bindings: HashMap::new(),
            parent: None,
        })))
    }

    pub fn new_with_parent(parent: &Frame<'arena>) -> Frame<'arena> {
        Frame(Rc::new(RefCell::new(FrameData {
            bindings: HashMap::new(),
            parent: Some(parent.clone()),
        })))
    }

    pub fn from_tuple(parent: &Frame<'arena>, tuple: Value<'arena>) -> Frame<'arena> {
        let tuple = tuple.borrow();
        let mut bindings = HashMap::with_capacity(tuple.len());
        for (key, value) in tuple.entries() {
            bindings.insert(key, value);
        }

        Frame(Rc::new(RefCell::new(FrameData {
            bindings,
            parent: Some(parent.clone()),
        })))
    }

    pub fn bind(&self, name: &'arena str, value: Value<'arena>) {
        self.0.borrow_mut().bindings.insert(name, value);
    }

    pub fn lookup(&self, name: &str) -> Option<Value<'arena>> {
        match self.0.borrow().bindings.get(name) {
            Some(value) => Some(*value),
            None => match &self.0.borrow().parent {
                Some(parent) => parent.lookup(name),
                None => None,
            },
        }
    }
}

impl Default for Frame<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Frame<'_> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::super::new_value::*;
    use super::*;

    #[test]
    fn bind() {
        let arena = ValueArena::new();
        let frame = Frame::new();
        frame.bind("a", arena.number(1));
        let a = frame.lookup("a");
        assert!(a.is_some());
        assert_eq!(*a.unwrap().borrow(), 1_isize);
    }

    #[test]
    fn lookup_through_parent() {
        let arena = ValueArena::new();
        let parent = Frame::new();
        parent.bind("a", arena.number(1));
        let frame = Frame::new_with_parent(&parent);
        let a = frame.lookup("a");
        assert!(a.is_some());
        assert_eq!(*a.unwrap().borrow(), 1_isize);
    }

    #[test]
    fn lookup_overriding_parent() {
        let arena = ValueArena::new();

        let parent = Frame::new();
        parent.bind("a", arena.number(1));

        let frame = Frame::new_with_parent(&parent);
        frame.bind("a", arena.number(2));

        let a = frame.lookup("a");
        assert!(a.is_some());
        assert_eq!(*a.unwrap().borrow(), 2_isize);
    }
}
