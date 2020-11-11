// use chrono::{DateTime, Utc};
use json::JsonValue;
use std::collections::HashMap;

/// A binding in a stack frame
pub enum Binding {
    Var(JsonValue),
    // Function(&'a dyn Fn(Vec<&JsonValue>) -> JsonValue, &'a str),
}

impl Binding {
    pub fn as_var(&self) -> &JsonValue {
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
pub struct Frame<'a> {
    /// Stores the bindings for the frame
    bindings: HashMap<String, Binding>,

    /// The parent frame of this frame
    parent_frame: Option<&'a Frame<'a>>,
    ///// The local timestamp in this frame
    //timestamp: DateTime<Utc>,
    // TODO: async, global
}

impl<'a> Frame<'a> {
    /// Creates a new empty frame
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent_frame: None,
            // timestamp: Utc::now(),
        }
    }

    /// Creates a new empty frame, with a parent frame for lookups
    pub fn new_with_parent(parent_frame: &'a Frame) -> Self {
        Self {
            bindings: HashMap::new(),
            parent_frame: Some(parent_frame),
            //timestamp: parent_frame.timestamp.clone(),
        }
    }

    /// Bind a value to a name in a frame
    pub fn bind(&mut self, name: &str, value: Binding) {
        &self.bindings.insert(name.to_string(), value);
    }

    /// Lookup a value by name in a frame
    pub fn lookup(&self, name: &str) -> Option<&Binding> {
        match &self.bindings.get(name) {
            Some(value) => Some(value),
            None => match &self.parent_frame {
                Some(parent) => parent.lookup(name),
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
        frame.bind("bool", Binding::Var(json::from(true)));
        frame.bind("number", Binding::Var(json::from(42)));
        frame.bind("string", Binding::Var(json::from("hello")));
        frame.bind("array", Binding::Var(json::from(vec![1, 2, 3])));
        frame.bind("none", Binding::Var(json::Null));

        assert!(frame.lookup("not_there").is_none());

        assert!(frame.lookup("bool").unwrap().as_var().is_boolean());
        assert!(frame.lookup("number").unwrap().as_var().is_number());
        assert!(frame.lookup("string").unwrap().as_var().is_string());
        assert!(frame.lookup("array").unwrap().as_var().is_array());
        assert!(frame.lookup("none").unwrap().as_var().is_empty());

        assert_eq!(
            frame.lookup("bool").unwrap().as_var().as_bool().unwrap(),
            true
        );
        assert_eq!(
            frame
                .lookup("number")
                .unwrap()
                .as_var()
                .as_number()
                .unwrap(),
            42
        );
        assert_eq!(
            frame.lookup("string").unwrap().as_var().as_str().unwrap(),
            "hello"
        );

        let array = frame.lookup("array");
        assert_eq!(array.unwrap().as_var().len(), 3);
    }

    #[test]
    fn lookup_through_parent() {
        let mut parent = Frame::new();
        parent.bind("value", Binding::Var(json::from(42)));
        let child = Frame::new_with_parent(&parent);
        assert_eq!(
            child.lookup("value").unwrap().as_var().as_number().unwrap(),
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
