// use chrono::{DateTime, Utc};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::Value;

/// A binding in a stack frame
#[derive(Debug, Clone)]
pub enum Binding {
    Var(Rc<Value>),
    // Function(&'a dyn Fn(Vec<&JsonValue>) -> JsonValue, &'a str),
}

impl Binding {
    pub fn as_var(&self) -> &Value {
        match self {
            Binding::Var(value) => &value,
            // _ => panic!("Binding is not a variable"),
        }
    }

    // pub fn as_func(&self) -> &dyn Fn(Vec<&JsonValue>) -> JsonValue {
    //     match self {
    //         Binding::Function(func, _) => func,
    //         _ => panic!("Binding is not a function"),
    //     }
    // }
}

/// A stack frame of the expression evaluation
#[derive(Debug, Clone)]
pub struct Frame {
    /// Stores the bindings for the frame
    bindings: HashMap<String, Binding>,

    /// The parent frame of this frame
    parent_frame: Option<Rc<RefCell<Frame>>>,
    ///// The local timestamp in this frame
    //timestamp: DateTime<Utc>,
    // TODO: async, global
}

impl Frame {
    /// Creates a new empty frame
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent_frame: None,
            // timestamp: Utc::now(),
        }
    }

    /// Creates a new empty frame, with a parent frame for lookups
    pub fn new_with_parent(parent_frame: Rc<RefCell<Frame>>) -> Self {
        Self {
            bindings: HashMap::new(),
            parent_frame: Some(parent_frame), //timestamp: parent_frame.timestamp.clone(),
        }
    }

    /// Bind a value to a name in a frame
    pub fn bind(&mut self, name: &str, value: Binding) {
        &self.bindings.insert(name.to_string(), value);
    }

    /// Lookup a value by name in a frame
    pub fn lookup(&self, name: &str) -> Option<Binding> {
        match self.bindings.get(name) {
            Some(binding) => Some(binding.clone()),
            None => match &self.parent_frame {
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
    fn bind_and_lookup() {
        let mut frame = Frame::new();
        frame.bind("bool", Binding::Var(Rc::new(json::from(true).into())));
        frame.bind("number", Binding::Var(Rc::new(json::from(42).into())));
        frame.bind("string", Binding::Var(Rc::new(json::from("hello").into())));
        frame.bind(
            "array",
            Binding::Var(Rc::new(json::from(vec![1, 2, 3]).into())),
        );
        frame.bind("none", Binding::Var(Rc::new(json::Null.into())));

        assert!(frame.lookup("not_there").is_none());

        assert!(frame.lookup("bool").unwrap().as_var().as_raw().is_boolean());
        assert!(frame
            .lookup("number")
            .unwrap()
            .as_var()
            .as_raw()
            .is_number());
        assert!(frame
            .lookup("string")
            .unwrap()
            .as_var()
            .as_raw()
            .is_string());
        assert!(frame.lookup("array").unwrap().as_var().is_array());
        assert!(frame.lookup("none").unwrap().as_var().as_raw().is_empty());

        assert_eq!(
            frame
                .lookup("bool")
                .unwrap()
                .as_var()
                .as_raw()
                .as_bool()
                .unwrap(),
            true
        );
        assert_eq!(
            frame
                .lookup("number")
                .unwrap()
                .as_var()
                .as_raw()
                .as_number()
                .unwrap(),
            42
        );
        assert_eq!(
            frame
                .lookup("string")
                .unwrap()
                .as_var()
                .as_raw()
                .as_str()
                .unwrap(),
            "hello"
        );

        let array = frame.lookup("array");
        assert_eq!(array.unwrap().as_var().len(), 3);
    }

    #[test]
    fn lookup_through_parent() {
        let parent = Rc::new(RefCell::new(Frame::new()));
        parent
            .borrow_mut()
            .bind("value", Binding::Var(Rc::new(json::from(42).into())));
        let child = Frame::new_with_parent(Rc::clone(&parent));
        assert_eq!(
            child
                .lookup("value")
                .unwrap()
                .as_var()
                .as_raw()
                .as_number()
                .unwrap(),
            42
        );
    }

    // #[test]
    // fn fn_binding() {
    //     let mut frame = Frame::new();
    //     frame.bind("sum", Binding::Function(&sum, ""));
    //     let sum = frame.lookup("sum").unwrap().as_func();
    //     assert_eq!(sum(vec![]).as_str().unwrap(), "todo");
    // }
}
