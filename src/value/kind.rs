use bitflags::bitflags;
use std::collections::HashMap;

use super::Value;
use crate::ast::Ast;
use crate::functions::FunctionContext;
use crate::json::Number;
use crate::Result;

bitflags! {
    pub struct ArrayFlags: u32 {
        const SEQUENCE  = 0b00000001;
        const SINGLETON = 0b00000010;
        const CONS      = 0b00000100;
        const WRAPPED   = 0b00001000;
    }
}

#[derive(Clone)]
pub enum ValueKind {
    Undefined,
    Null,
    Number(Number),
    Bool(bool),
    String(String),
    Array(Vec<usize>, ArrayFlags),
    Object(HashMap<String, usize>),
    Lambda(String, Ast),
    NativeFn0(String, fn(&FunctionContext) -> Result<Value>),
    NativeFn1(String, fn(&FunctionContext, &Value) -> Result<Value>),
    NativeFn2(
        String,
        fn(&FunctionContext, &Value, &Value) -> Result<Value>,
    ),
    NativeFn3(
        String,
        fn(&FunctionContext, &Value, &Value, &Value) -> Result<Value>,
    ),
}

impl PartialEq<ValueKind> for ValueKind {
    fn eq(&self, other: &ValueKind) -> bool {
        match (self, other) {
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Array(l0, ..), Self::Array(r0, ..)) => {
                println!("l0: {:#?}", l0);
                println!("r0: {:#?}", r0);
                l0 == r0
            }
            (Self::Object(l0), Self::Object(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl PartialEq<i32> for ValueKind {
    fn eq(&self, other: &i32) -> bool {
        match *self {
            ValueKind::Number(ref n) => *n == *other,
            _ => false,
        }
    }
}

impl PartialEq<i64> for ValueKind {
    fn eq(&self, other: &i64) -> bool {
        match *self {
            ValueKind::Number(ref n) => *n == *other,
            _ => false,
        }
    }
}

impl PartialEq<f32> for ValueKind {
    fn eq(&self, other: &f32) -> bool {
        match *self {
            ValueKind::Number(ref n) => *n == *other,
            _ => false,
        }
    }
}

impl PartialEq<f64> for ValueKind {
    fn eq(&self, other: &f64) -> bool {
        match *self {
            ValueKind::Number(ref n) => *n == *other,
            _ => false,
        }
    }
}

impl PartialEq<bool> for ValueKind {
    fn eq(&self, other: &bool) -> bool {
        match *self {
            ValueKind::Bool(ref b) => *b == *other,
            _ => false,
        }
    }
}

impl PartialEq<&str> for ValueKind {
    fn eq(&self, other: &&str) -> bool {
        match *self {
            ValueKind::String(ref s) => s == *other,
            _ => false,
        }
    }
}

impl PartialEq<String> for ValueKind {
    fn eq(&self, other: &String) -> bool {
        match *self {
            ValueKind::String(ref s) => *s == *other,
            _ => false,
        }
    }
}

impl From<i32> for ValueKind {
    fn from(v: i32) -> Self {
        ValueKind::Number(v.into())
    }
}

impl From<i64> for ValueKind {
    fn from(v: i64) -> Self {
        ValueKind::Number(v.into())
    }
}

impl From<f32> for ValueKind {
    fn from(v: f32) -> Self {
        ValueKind::Number(v.into())
    }
}

impl From<f64> for ValueKind {
    fn from(v: f64) -> Self {
        ValueKind::Number(v.into())
    }
}

impl From<bool> for ValueKind {
    fn from(v: bool) -> Self {
        ValueKind::Bool(v)
    }
}

impl From<&str> for ValueKind {
    fn from(v: &str) -> Self {
        ValueKind::String(v.into())
    }
}

impl std::fmt::Debug for ValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Undefined => write!(f, "undefined"),
            Self::Null => write!(f, "null"),
            Self::Number(n) => write!(f, "{}", n.to_string()),
            Self::Bool(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            Self::String(s) => write!(f, "\"{}\"", s),
            Self::Array(a, _) => write!(f, "<array({})>", a.len()),
            Self::Object(o) => write!(
                f,
                "<object{{{}}}>",
                o.keys().cloned().collect::<Vec<String>>().join(", ")
            ),
            Self::Lambda(..) => write!(f, "<lambda>"),
            Self::NativeFn0(..)
            | Self::NativeFn1(..)
            | Self::NativeFn2(..)
            | Self::NativeFn3(..) => {
                write!(f, "<nativefn>")
            }
        }
    }
}
