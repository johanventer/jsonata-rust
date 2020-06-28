#![allow(incomplete_features)] // For const_generics
#![feature(or_patterns)]
#![feature(const_generics)]
// TODO: Disable these
#![allow(unused_variables)]
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use json::JsonValue;
use std::collections::HashMap;

#[macro_use]
mod error;
mod ast;
mod evaluator;
mod parser;
mod symbol;
mod tokenizer;

use evaluator::evaluate;

// use ast::NodeMethods;

/// A binding in a stack frame
pub enum Binding<'a> {
    Variable(JsonValue),
    Function(&'a dyn Fn(Vec<&JsonValue>) -> JsonValue, &'a str),
}

impl Binding<'_> {
    pub fn as_var(&self) -> &JsonValue {
        match self {
            Binding::Variable(value) => &value,
            _ => panic!("Binding is not a variable"),
        }
    }

    pub fn as_func(&self) -> &dyn Fn(Vec<&JsonValue>) -> JsonValue {
        match self {
            Binding::Function(func, _) => func,
            _ => panic!("Binding is not a function"),
        }
    }
}

fn sum(args: Vec<&JsonValue>) -> JsonValue {
    json::from("todo")
}

/// A stack frame of the expression evaluation
struct Frame<'a> {
    /// Stores the bindings for the frame
    bindings: HashMap<String, Binding<'a>>,

    /// The parent frame of this frame
    parent_frame: Option<&'a Frame<'a>>,

    /// The local timestamp in this frame
    timestamp: DateTime<Utc>,
    // TODO: async, global
}

impl<'a> Frame<'a> {
    /// Creates a new empty frame
    fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent_frame: None,
            timestamp: Utc::now(),
        }
    }

    /// Creates a new empty frame, with a parent frame for lookups
    fn new_from(parent_frame: &'a Frame<'a>) -> Self {
        Self {
            bindings: HashMap::new(),
            parent_frame: Some(parent_frame),
            timestamp: parent_frame.timestamp.clone(),
        }
    }

    /// Bind a value to a name in a frame
    fn bind(&mut self, name: &str, value: Binding<'a>) {
        &self.bindings.insert(name.to_string(), value);
    }

    /// Lookup a value by name in a frame
    fn lookup(&self, name: &str) -> Option<&Binding> {
        match &self.bindings.get(name) {
            Some(value) => Some(value),
            None => match &self.parent_frame {
                Some(parent) => parent.lookup(name),
                None => None,
            },
        }
    }
}

pub struct JsonAta<'a> {
    expr: String,
    environment: Frame<'a>,
    ast: ast::Node,
}

impl<'a> JsonAta<'a> {
    pub fn new(expr: &'a str) -> Self {
        let mut environment = Frame::new();

        // TODO: Apply statics to the environment
        environment.bind("sum", Binding::Function(&sum, "<a<n>:n>"));

        // TODO: Probably could just do this once somewhere to avoid doing it every time

        Self {
            expr: expr.to_string(),
            environment,
            ast: parser::parse(expr),
        }
    }

    pub fn evaluate(&self, input: String, bindings: Vec<Binding>) -> JsonValue {
        evaluate(&self.ast).unwrap().unwrap()
    }

    pub fn assign(&mut self, name: &str, value: Binding<'a>) {
        self.environment.bind(name, value);
    }

    // pub fn ast(&self) -> JsonValue {
    //     self.ast.to_json()
    // }
}

#[cfg(test)]
mod evaluator_tests {
    use super::*;

    // #[test]
    // fn bind_and_lookup() {
    //     let mut frame = Frame::new();
    //     frame.bind("bool", Binding::Variable(json::from(true)));
    //     frame.bind("number", Binding::Variable(json::from(42)));
    //     frame.bind("string", Binding::Variable(json::from("hello")));
    //     frame.bind("array", Binding::Variable(json::from(vec![1, 2, 3])));
    //     frame.bind("none", Binding::Variable(json::Null));

    //     assert!(frame.lookup("not_there").is_none());

    //     assert!(frame.lookup("bool").unwrap().as_var().is_boolean());
    //     assert!(frame.lookup("number").unwrap().as_var().is_number());
    //     assert!(frame.lookup("string").unwrap().as_var().is_string());
    //     assert!(frame.lookup("array").unwrap().as_var().is_array());
    //     assert!(frame.lookup("none").unwrap().as_var().is_empty());

    //     assert_eq!(
    //         frame.lookup("bool").unwrap().as_var().as_bool().unwrap(),
    //         true
    //     );
    //     assert_eq!(
    //         frame
    //             .lookup("number")
    //             .unwrap()
    //             .as_var()
    //             .as_number()
    //             .unwrap(),
    //         42
    //     );
    //     assert_eq!(
    //         frame.lookup("string").unwrap().as_var().as_str().unwrap(),
    //         "hello"
    //     );

    //     let array = frame.lookup("array");
    //     assert_eq!(array.unwrap().as_var().len(), 3);
    // }

    // #[test]
    // fn lookup_through_parent() {
    //     let mut parent = Frame::new();
    //     parent.bind("value", Binding::Variable(json::from(42)));
    //     let child = Frame::new_from(&parent);
    //     assert_eq!(
    //         child.lookup("value").unwrap().as_var().as_number().unwrap(),
    //         42
    //     );
    // }

    // #[test]
    // fn fn_binding() {
    //     let mut frame = Frame::new();
    //     frame.bind("sum", Binding::Function(&sum, ""));
    //     let sum = frame.lookup("sum").unwrap().as_func();
    //     assert_eq!(sum(vec![]).as_str().unwrap(), "todo");
    // }

    #[test]
    fn add() {
        let jsonata = JsonAta::new("1 + 3");
        let result = jsonata.evaluate("".to_string(), Vec::new());
        assert_eq!(result, json::from(4));
    }

    #[test]
    fn sub() {
        let jsonata = JsonAta::new("1 - 3");
        let result = jsonata.evaluate("".to_string(), Vec::new());
        assert_eq!(result, json::from(-2));
    }

    #[test]
    fn mul() {
        let jsonata = JsonAta::new("4 * 7");
        let result = jsonata.evaluate("".to_string(), Vec::new());
        assert_eq!(result, json::from(28));
    }

    #[test]
    fn div() {
        let jsonata = JsonAta::new("10 / 2");
        let result = jsonata.evaluate("".to_string(), Vec::new());
        assert_eq!(result, json::from(5));
    }

    #[test]
    fn modulo() {
        let jsonata = JsonAta::new("10 % 8");
        let result = jsonata.evaluate("".to_string(), Vec::new());
        assert_eq!(result, json::from(2));
    }

    #[test]
    fn less_than_num_true() {
        let jsonata = JsonAta::new("3 < 4");
        let result = jsonata.evaluate("".to_string(), Vec::new());
        assert_eq!(result, json::from(true));
    }

    #[test]
    fn less_than_num_false() {
        let jsonata = JsonAta::new("4 < 3");
        let result = jsonata.evaluate("".to_string(), Vec::new());
        assert_eq!(result, json::from(false));
    }

    #[test]
    fn less_than_str_true() {
        let jsonata = JsonAta::new("\"3\" < \"4\"");
        let result = jsonata.evaluate("".to_string(), Vec::new());
        assert_eq!(result, json::from(true));
    }

    #[test]
    fn less_than_str_false() {
        let jsonata = JsonAta::new("\"4\" < \"3\"");
        let result = jsonata.evaluate("".to_string(), Vec::new());
        assert_eq!(result, json::from(false));
    }

    #[test]
    fn str_concat() {
        let jsonata = JsonAta::new("\"hello\" & \" world\"");
        let result = jsonata.evaluate("".to_string(), Vec::new());
        assert_eq!(result, json::from("hello world"));
    }

    #[test]
    fn eq() {
        let jsonata = JsonAta::new("1 = 1");
        let result = jsonata.evaluate("".to_string(), Vec::new());
        assert_eq!(result, json::from(true));
    }

    #[test]
    fn neq() {
        let jsonata = JsonAta::new("1 != 2");
        let result = jsonata.evaluate("".to_string(), Vec::new());
        assert_eq!(result, json::from(true));
    }

    // #[test]
    // fn math() {
    //     let jsonata = JsonAta::new("(2 + 3) * 4 + 2");
    //     let result = jsonata.evaluate("".to_string(), Vec::new());
    //     assert_eq!(result, json::from(26));
    // }
}
