use std::borrow::Cow;
use std::fmt;

use bitflags::bitflags;
use bumpalo::Bump;
use hashbrown::HashMap;

use crate::ast::{Ast, AstKind};
use crate::frame::Frame;
use crate::functions::FunctionContext;
use crate::json::codegen::{DumpGenerator, Generator, PrettyGenerator};
use crate::json::Number;
use crate::Result;

bitflags! {
    pub struct ArrayFlags: u8 {
        const SEQUENCE  = 0b00000001;
        const SINGLETON = 0b00000010;
        const CONS      = 0b00000100;
        const WRAPPED   = 0b00001000;
    }
}

pub const UNDEFINED: Value = Value::Undefined;

pub enum Value<'a> {
    Undefined,
    Null,
    Number(Number),
    Bool(bool),
    String(String),
    Array(Vec<&'a Value<'a>>, ArrayFlags),
    Object(HashMap<String, &'a Value<'a>>),
    Lambda {
        ast: *const Ast,
        input: &'a Value<'a>,
        frame: Frame,
    },
    NativeFn0(String, fn(FunctionContext<'a>) -> Result<&'a Value<'a>>),
    NativeFn1(
        String,
        fn(FunctionContext<'a>, ValuePtr) -> Result<&'a Value<'a>>,
    ),
    NativeFn2(
        String,
        fn(FunctionContext, ValuePtr, ValuePtr) -> Result<ValuePtr>,
    ),
    NativeFn3(
        String,
        fn(FunctionContext, ValuePtr, ValuePtr, ValuePtr) -> Result<ValuePtr>,
    ),
}

#[allow(clippy::mut_from_ref)]
impl<'a> Value<'a> {
    pub fn as_ptr(&'a self) -> ValuePtr {
        ValuePtr(unsafe { std::mem::transmute::<&'a Value<'a>, &'static Value<'static>>(self) })
    }

    pub fn undefined() -> &'a Value<'a> {
        unsafe { std::mem::transmute::<&Value<'static>, &'a Value<'a>>(&UNDEFINED) }
    }

    pub fn null(arena: &Bump) -> &mut Value {
        arena.alloc(Value::Null)
    }

    pub fn bool(arena: &Bump, value: bool) -> &mut Value {
        arena.alloc(Value::Bool(value))
    }

    pub fn number(arena: &Bump, value: impl Into<Number>) -> &mut Value {
        arena.alloc(Value::Number(value.into()))
    }

    pub fn string(arena: &Bump, value: impl Into<String>) -> &mut Value {
        arena.alloc(Value::String(value.into()))
    }

    pub fn array(arena: &Bump, flags: ArrayFlags) -> &mut Value {
        arena.alloc(Value::Array(Vec::new(), flags))
    }

    pub fn array_with_capacity(arena: &Bump, capacity: usize, flags: ArrayFlags) -> &mut Value {
        arena.alloc(Value::Array(Vec::with_capacity(capacity), flags))
    }

    pub fn object(arena: &Bump) -> &mut Value {
        arena.alloc(Value::Object(HashMap::new()))
    }

    pub fn object_with_capacity(arena: &Bump, capacity: usize) -> &mut Value {
        arena.alloc(Value::Object(HashMap::with_capacity(capacity)))
    }

    pub fn lambda(
        arena: &'a Bump,
        node: &Ast,
        input: &'a Value<'a>,
        frame: Frame,
    ) -> &'a mut Value<'a> {
        arena.alloc(Value::Lambda {
            ast: node,
            input,
            frame,
        })
    }

    pub fn nativefn0(
        arena: &'a Bump,
        name: &str,
        func: fn(FunctionContext) -> Result<&'a Value<'a>>,
    ) -> &'a mut Value<'a> {
        arena.alloc(Value::NativeFn0(name.to_string(), func))
    }

    pub fn nativefn1(
        arena: &'a Bump,
        name: &str,
        func: fn(FunctionContext<'a>, ValuePtr) -> Result<&'a Value<'a>>,
    ) -> &'a mut Value<'a> {
        arena.alloc(Value::NativeFn1(name.to_string(), func))
    }

    pub fn nativefn2(
        arena: &'a Bump,
        name: &str,
        func: fn(FunctionContext, ValuePtr, ValuePtr) -> Result<ValuePtr>,
    ) -> &'a mut Value<'a> {
        arena.alloc(Value::NativeFn2(name.to_string(), func))
    }

    pub fn nativefn3(
        arena: &'a Bump,
        name: &str,
        func: fn(FunctionContext, ValuePtr, ValuePtr, ValuePtr) -> Result<ValuePtr>,
    ) -> &'a mut Value<'a> {
        arena.alloc(Value::NativeFn3(name.to_string(), func))
    }

    pub fn is_undefined(&self) -> bool {
        matches!(*self, Value::Undefined)
    }

    pub fn is_null(&self) -> bool {
        matches!(*self, Value::Null)
    }

    pub fn is_bool(&self) -> bool {
        matches!(&self, Value::Bool(..))
    }

    pub fn is_number(&self) -> bool {
        matches!(&self, Value::Number(..))
    }

    pub fn is_integer(&self) -> bool {
        if let Value::Number(ref n) = *self {
            let n = f64::from(*n);
            match n.classify() {
                std::num::FpCategory::Nan
                | std::num::FpCategory::Infinite
                | std::num::FpCategory::Subnormal => false,
                _ => {
                    let mantissa = n.trunc();
                    n - mantissa == 0.0
                }
            }
        } else {
            false
        }
    }

    pub fn is_nan(&self) -> bool {
        matches!(*self, Value::Number(n) if n.is_nan())
    }

    pub fn is_string(&self) -> bool {
        matches!(*self, Value::String(..))
    }

    pub fn is_array(&self) -> bool {
        matches!(*self, Value::Array(..))
    }

    pub fn is_object(&self) -> bool {
        matches!(*self, Value::Object(..))
    }

    pub fn is_function(&self) -> bool {
        matches!(
            *self,
            Value::Lambda { .. }
                | Value::NativeFn0(..)
                | Value::NativeFn1(..)
                | Value::NativeFn2(..)
                | Value::NativeFn3(..)
        )
    }

    pub fn is_truthy(&'a self) -> bool {
        match *self {
            Value::Undefined => false,
            Value::Null => false,
            Value::Number(ref n) => *n != 0.0,
            Value::Bool(ref b) => *b,
            Value::String(ref s) => !s.is_empty(),
            Value::Array(ref a, _) => match a.len() {
                0 => false,
                1 => self.get_member(0).is_truthy(),
                _ => {
                    for item in self.members() {
                        if item.is_truthy() {
                            return true;
                        }
                    }
                    false
                }
            },
            Value::Object(ref o) => !o.is_empty(),
            Value::Lambda { .. }
            | Value::NativeFn0(..)
            | Value::NativeFn1(..)
            | Value::NativeFn2(..)
            | Value::NativeFn3(..) => false,
        }
    }

    pub fn get_member(&'a self, index: usize) -> &Value {
        match *self {
            Value::Array(ref array, _) => match array.get(index) {
                Some(value) => *value,
                None => Value::undefined(),
            },
            _ => panic!("Not an array"),
        }
    }

    pub fn members(&self) -> std::slice::Iter<'_, &'a Value> {
        match *self {
            Value::Array(ref array, _) => array.iter(),
            _ => panic!("Not an array"),
        }
    }

    pub fn entries(&self) -> hashbrown::hash_map::Iter<'_, String, &'a Value> {
        match *self {
            Value::Object(ref map) => map.iter(),
            _ => panic!("Not an object"),
        }
    }

    pub fn arity(&self) -> usize {
        match *self {
            Value::Lambda { ref ast, .. } => {
                if let AstKind::Lambda { args, .. } = unsafe { &(**ast).kind } {
                    args.len()
                } else {
                    0
                }
            }
            Value::NativeFn0(..) => 0,
            Value::NativeFn1(..) => 1,
            Value::NativeFn2(..) => 2,
            Value::NativeFn3(..) => 3,
            _ => panic!("Not a function"),
        }
    }

    pub fn as_bool(&self) -> bool {
        match *self {
            Value::Bool(ref b) => *b,
            _ => panic!("Not a bool"),
        }
    }

    pub fn as_f32(&self) -> f32 {
        match *self {
            Value::Number(ref n) => f32::from(*n),
            _ => panic!("Not a number"),
        }
    }

    pub fn as_f64(&self) -> f64 {
        match *self {
            Value::Number(ref n) => f64::from(*n),
            _ => panic!("Not a number"),
        }
    }

    pub fn as_usize(&self) -> usize {
        match *self {
            Value::Number(ref n) => f64::from(*n) as usize,
            _ => panic!("Not a number"),
        }
    }

    pub fn as_isize(&self) -> isize {
        match *self {
            Value::Number(ref n) => f64::from(*n) as isize,
            _ => panic!("Not a number"),
        }
    }

    pub fn as_str(&self) -> Cow<'_, str> {
        match *self {
            Value::String(ref s) => Cow::from(s),
            _ => panic!("Not a string"),
        }
    }

    pub fn len(&self) -> usize {
        match *self {
            Value::Array(ref array, _) => array.len(),
            _ => panic!("Not an array"),
        }
    }

    pub fn is_empty(&self) -> bool {
        match *self {
            Value::Array(ref array, _) => array.is_empty(),
            _ => panic!("Not an array"),
        }
    }

    pub fn get_entry(&'a self, key: &str) -> &Value {
        match *self {
            Value::Object(ref map) => match map.get(key) {
                Some(value) => value,
                None => Value::undefined(),
            },
            _ => panic!("Not an object"),
        }
    }

    pub fn push(&mut self, value: &'a Value<'a>) {
        match *self {
            Value::Array(ref mut array, _) => array.push(value),
            _ => panic!("Not an array"),
        }
    }

    pub fn insert(&mut self, key: &str, value: &'a Value<'a>) {
        match *self {
            Value::Object(ref mut map) => {
                map.insert(key.to_owned(), value);
            }
            _ => panic!("Not an object"),
        }
    }

    pub fn wrap_in_array(
        arena: &'a Bump,
        value: &'a Value<'a>,
        flags: ArrayFlags,
    ) -> &'a Value<'a> {
        arena.alloc(Value::Array(vec![value], flags))
    }

    pub fn wrap_in_array_if_needed(
        arena: &'a Bump,
        value: &'a Value<'a>,
        flags: ArrayFlags,
    ) -> &'a Value<'a> {
        if value.is_array() {
            value
        } else {
            Value::wrap_in_array(arena, value, flags)
        }
    }

    pub fn get_flags(&self) -> ArrayFlags {
        match *self {
            Value::Array(_, flags) => flags,
            _ => panic!("Not an array"),
        }
    }

    pub fn has_flags(&self, check_flags: ArrayFlags) -> bool {
        match *self {
            Value::Array(_, flags) => flags.contains(check_flags),
            _ => false,
        }
    }

    pub fn clone_array_with_flags(&self, arena: &'a Bump, flags: ArrayFlags) -> &'a Value<'a> {
        match *self {
            Value::Array(ref array, _) => arena.alloc(Value::Array(array.clone(), flags)),
            _ => panic!("Not an array"),
        }
    }

    // Prints out the value as JSON string.
    pub fn dump(&'a self) -> String {
        let mut gen = DumpGenerator::new();
        gen.write_json(self).expect("Can't fail");
        gen.consume()
    }

    /// Pretty prints out the value as JSON string. Takes an argument that's
    /// number of spaces to indent new blocks with.
    pub fn pretty(&'a self, spaces: u16) -> String {
        let mut gen = PrettyGenerator::new(spaces);
        gen.write_json(self).expect("Can't fail");
        gen.consume()
    }
}

impl<'a> PartialEq<Value<'a>> for Value<'a> {
    fn eq(&self, other: &Value<'a>) -> bool {
        match (self, other) {
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Array(l0, ..), Self::Array(r0, ..)) => l0 == r0,
            (Self::Object(l0), Self::Object(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl PartialEq<i32> for Value<'_> {
    fn eq(&self, other: &i32) -> bool {
        match *self {
            Value::Number(ref n) => *n == *other,
            _ => false,
        }
    }
}

impl PartialEq<i64> for Value<'_> {
    fn eq(&self, other: &i64) -> bool {
        match *self {
            Value::Number(ref n) => *n == *other,
            _ => false,
        }
    }
}

impl PartialEq<f32> for Value<'_> {
    fn eq(&self, other: &f32) -> bool {
        match *self {
            Value::Number(ref n) => *n == *other,
            _ => false,
        }
    }
}

impl PartialEq<f64> for Value<'_> {
    fn eq(&self, other: &f64) -> bool {
        match *self {
            Value::Number(ref n) => *n == *other,
            _ => false,
        }
    }
}

impl PartialEq<bool> for Value<'_> {
    fn eq(&self, other: &bool) -> bool {
        match *self {
            Value::Bool(ref b) => *b == *other,
            _ => false,
        }
    }
}

impl PartialEq<&str> for Value<'_> {
    fn eq(&self, other: &&str) -> bool {
        match *self {
            Value::String(ref s) => s == *other,
            _ => false,
        }
    }
}

impl PartialEq<String> for Value<'_> {
    fn eq(&self, other: &String) -> bool {
        match *self {
            Value::String(ref s) => *s == *other,
            _ => false,
        }
    }
}

impl From<i32> for Value<'_> {
    fn from(v: i32) -> Self {
        Value::Number(v.into())
    }
}

impl From<i64> for Value<'_> {
    fn from(v: i64) -> Self {
        Value::Number(v.into())
    }
}

impl From<f32> for Value<'_> {
    fn from(v: f32) -> Self {
        Value::Number(v.into())
    }
}

impl From<f64> for Value<'_> {
    fn from(v: f64) -> Self {
        Value::Number(v.into())
    }
}

impl From<bool> for Value<'_> {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

impl From<&str> for Value<'_> {
    fn from(v: &str) -> Self {
        Value::String(v.into())
    }
}

impl std::fmt::Debug for Value<'_> {
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
            Self::Lambda { .. } => write!(f, "<lambda>"),
            Self::NativeFn0(..)
            | Self::NativeFn1(..)
            | Self::NativeFn2(..)
            | Self::NativeFn3(..) => {
                write!(f, "<nativefn>")
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct ValuePtr(*const Value<'static>);

impl ValuePtr {
    pub fn as_ref<'a>(&self, _arena: &'a Bump) -> &'a Value<'a> {
        unsafe { std::mem::transmute::<&'a Value<'static>, &'a Value<'a>>(&*self.0) }
    }
}

impl std::fmt::Debug for ValuePtr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe { (*self.0).fmt(f) }
    }
}

// impl PartialEq<ValuePtr> for ValuePtr {
//     fn eq(&self, other: &ValuePtr) -> bool {
//         match unsafe { &*self.0 } {
//             Value::Array(..) => {
//                 if other.as_ref().is_array() && other.as_ref().len() == self.as_ref().len() {
//                     self.as_ref()
//                         .members()
//                         .zip(other.as_ref().members())
//                         .all(|(l, r)| l == r)
//                 } else {
//                     false
//                 }
//             }
//             Value::Object(..) => {
//                 if other.as_ref().is_object() {
//                     self.as_ref()
//                         .entries()
//                         .all(|(k, v)| *v == other.as_ref().get_entry(k))
//                 } else {
//                     false
//                 }
//             }
//             _ => unsafe { self.as_ref() == other.as_ref() },
//         }
//     }
// }
