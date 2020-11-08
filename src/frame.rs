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
