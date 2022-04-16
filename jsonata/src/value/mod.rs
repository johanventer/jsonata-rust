use std::borrow::Cow;
use std::ops::Index;

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

/// The core value type for input, output and evaluation. There's a lot of lifetimes here to avoid
/// cloning any part of the input that should be kept in the output, avoiding heap allocations for
/// every Value, and allowing structural sharing.
///
/// Values are all allocated in a Bump arena, making them contiguous in memory and further avoiding
/// heap allocations for every one.
pub enum Value<'a> {
    Undefined,
    Null,
    Number(Number),
    Bool(bool),
    String(String),
    Array(Vec<&'a Value<'a>>, ArrayFlags),
    Object(HashMap<String, &'a Value<'a>>),
    Lambda {
        ast: Ast,
        input: &'a Value<'a>,
        frame: Frame<'a>,
    },
    NativeFn0(String, fn(FunctionContext<'a, '_>) -> Result<&'a Value<'a>>),
    NativeFn1(
        String,
        fn(FunctionContext<'a, '_>, &'a Value<'a>) -> Result<&'a Value<'a>>,
    ),
    NativeFn2(
        String,
        fn(FunctionContext<'a, '_>, &'a Value<'a>, &'a Value<'a>) -> Result<&'a Value<'a>>,
    ),
    NativeFn3(
        String,
        fn(
            FunctionContext<'a, '_>,
            &'a Value<'a>,
            &'a Value<'a>,
            &'a Value<'a>,
        ) -> Result<&'a Value<'a>>,
    ),
}

#[allow(clippy::mut_from_ref)]
impl<'a> Value<'a> {
    pub fn undefined() -> &'a Value<'a> {
        // TODO: SAFETY: The UNDEFINED const is Value<'static>, and doesn't reference any other Values,
        // so there shouldn't be an issue casting it Value<'a>, right?
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
        frame: Frame<'a>,
    ) -> &'a mut Value<'a> {
        arena.alloc(Value::Lambda {
            ast: node.clone(),
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
        func: fn(FunctionContext<'a, '_>, &'a Value<'a>) -> Result<&'a Value<'a>>,
    ) -> &'a mut Value<'a> {
        arena.alloc(Value::NativeFn1(name.to_string(), func))
    }

    pub fn nativefn2(
        arena: &'a Bump,
        name: &str,
        func: fn(FunctionContext<'a, '_>, &'a Value<'a>, &'a Value<'a>) -> Result<&'a Value<'a>>,
    ) -> &'a mut Value<'a> {
        arena.alloc(Value::NativeFn2(name.to_string(), func))
    }

    pub fn nativefn3(
        arena: &'a Bump,
        name: &str,
        func: fn(
            FunctionContext<'a, '_>,
            &'a Value<'a>,
            &'a Value<'a>,
            &'a Value<'a>,
        ) -> Result<&'a Value<'a>>,
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
                if let AstKind::Lambda { ref args, .. } = ast.kind {
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
            (Value::Undefined, Value::Undefined) => true,
            (Value::Null, Value::Null) => true,
            (Value::Number(l), Value::Number(r)) => *l == *r,
            (Value::Bool(l), Value::Bool(r)) => *l == *r,
            (Value::String(l), Value::String(r)) => *l == *r,
            (Value::Array(l, ..), Value::Array(r, ..)) => *l == *r,
            (Value::Object(l), Value::Object(r)) => *l == *r,
            _ => false,
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

impl<'a> Index<&str> for Value<'a> {
    type Output = Value<'a>;

    fn index(&self, index: &str) -> &Self::Output {
        match *self {
            Value::Object(ref o) => match o.get(index) {
                Some(value) => value,
                None => Value::undefined(),
            },
            _ => Value::undefined(),
        }
    }
}

impl std::fmt::Debug for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Undefined => write!(f, "undefined"),
            Self::Null => write!(f, "null"),
            Self::Number(n) => write!(f, "{}", n),
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
