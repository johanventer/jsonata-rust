use std::borrow::Cow;
use std::ops::Index;

use bitflags::bitflags;
use bumpalo::boxed::Box;
use bumpalo::Bump;
use hashbrown::HashMap;

use crate::ast::{Ast, AstKind};
use crate::frame::Frame;
use crate::functions::FunctionContext;
use crate::Result;
use jsonata_errors::Error;

mod serialize;
use serialize::{DumpSerializer, PrettySerializer, Serializer};

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
    Number(f64),
    Bool(bool),
    String(String),
    Array(Box<'a, Vec<&'a Value<'a>>>, ArrayFlags),
    Object(Box<'a, HashMap<String, &'a Value<'a>>>),
    Lambda {
        ast: Ast,
        input: &'a Value<'a>,
        frame: Frame<'a>,
    },
    NativeFn {
        name: String,
        arity: usize,
        func: fn(FunctionContext<'a, '_>, &'a Value<'a>) -> Result<&'a Value<'a>>,
    },
}

#[allow(clippy::mut_from_ref)]
impl<'a> Value<'a> {
    pub fn undefined() -> &'a Value<'a> {
        // SAFETY: The UNDEFINED const is Value<'static>, it doesn't reference any other Values,
        // and there's no Drop implementation, so there shouldn't be an issue casting it to Value<'a>.
        unsafe { std::mem::transmute::<&Value<'static>, &'a Value<'a>>(&UNDEFINED) }
    }

    pub fn null(arena: &Bump) -> &mut Value {
        arena.alloc(Value::Null)
    }

    pub fn bool(arena: &Bump, value: bool) -> &mut Value {
        arena.alloc(Value::Bool(value))
    }

    pub fn number(arena: &Bump, value: impl Into<f64>) -> &mut Value {
        arena.alloc(Value::Number(value.into()))
    }

    pub fn string(arena: &Bump, value: impl Into<String>) -> &mut Value {
        arena.alloc(Value::String(value.into()))
    }

    pub fn array(arena: &Bump, flags: ArrayFlags) -> &mut Value {
        arena.alloc(Value::Array(Box::new_in(Vec::new(), arena), flags))
    }

    pub fn array_with_capacity(arena: &Bump, capacity: usize, flags: ArrayFlags) -> &mut Value {
        arena.alloc(Value::Array(
            Box::new_in(Vec::with_capacity(capacity), arena),
            flags,
        ))
    }

    pub fn object(arena: &Bump) -> &mut Value {
        arena.alloc(Value::Object(Box::new_in(HashMap::new(), arena)))
    }

    pub fn object_with_capacity(arena: &Bump, capacity: usize) -> &mut Value {
        arena.alloc(Value::Object(Box::new_in(
            HashMap::with_capacity(capacity),
            arena,
        )))
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

    pub fn nativefn(
        arena: &'a Bump,
        name: &str,
        arity: usize,
        func: fn(FunctionContext<'a, '_>, &'a Value<'a>) -> Result<&'a Value<'a>>,
    ) -> &'a mut Value<'a> {
        arena.alloc(Value::NativeFn {
            name: name.to_string(),
            arity,
            func,
        })
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
        match self {
            Value::Number(n) => match n.classify() {
                std::num::FpCategory::Nan
                | std::num::FpCategory::Infinite
                | std::num::FpCategory::Subnormal => false,
                _ => {
                    let mantissa = n.trunc();
                    n - mantissa == 0.0
                }
            },
            _ => false,
        }
    }

    pub fn is_array_of_valid_numbers(&self) -> Result<bool> {
        match self {
            Value::Array(ref a, _) => {
                for member in a.iter() {
                    if !member.is_valid_number()? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn is_valid_number(&self) -> Result<bool> {
        match self {
            Value::Number(n) => {
                if n.is_nan() {
                    Ok(false)
                } else if n.is_infinite() {
                    Err(Error::D1001NumberOfOutRange(*n))
                } else {
                    Ok(true)
                }
            }
            _ => Ok(false),
        }
    }

    pub fn is_nan(&self) -> bool {
        matches!(*self, Value::Number(n) if n.is_nan())
    }

    pub fn is_finite(&self) -> bool {
        match self {
            Value::Number(n) => n.is_finite(),
            _ => false,
        }
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
        matches!(*self, Value::Lambda { .. } | Value::NativeFn { .. })
    }

    pub fn is_truthy(&'a self) -> bool {
        match *self {
            Value::Undefined => false,
            Value::Null => false,
            Value::Number(n) => n != 0.0,
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
            Value::Lambda { .. } | Value::NativeFn { .. } => false,
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
                    panic!("Not a lambda function")
                }
            }
            Value::NativeFn { arity, .. } => arity,
            _ => panic!("Not a function"),
        }
    }

    pub fn as_bool(&self) -> bool {
        match *self {
            Value::Bool(ref b) => *b,
            _ => panic!("Not a bool"),
        }
    }

    pub fn as_f64(&self) -> f64 {
        match *self {
            Value::Number(n) => n,
            _ => panic!("Not a number"),
        }
    }

    // TODO(math): Completely unchecked, audit usage
    pub fn as_usize(&self) -> usize {
        match *self {
            Value::Number(n) => n as usize,
            _ => panic!("Not a number"),
        }
    }

    // TODO(math): Completely unchecked, audit usage
    pub fn as_isize(&self) -> isize {
        match *self {
            Value::Number(n) => n as isize,
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

    pub fn flatten(&'a self, arena: &'a Bump) -> &'a mut Value<'a> {
        let flattened = Self::array(arena, ArrayFlags::empty());
        self._flatten(flattened)
    }

    fn _flatten(&'a self, flattened: &'a mut Value<'a>) -> &'a mut Value<'a> {
        let mut flattened = flattened;

        if self.is_array() {
            for member in self.members() {
                flattened = member._flatten(flattened);
            }
        } else {
            flattened.push(self)
        }

        flattened
    }

    pub fn wrap_in_array(
        arena: &'a Bump,
        value: &'a Value<'a>,
        flags: ArrayFlags,
    ) -> &'a Value<'a> {
        arena.alloc(Value::Array(Box::new_in(vec![value], arena), flags))
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
            Value::Array(ref array, _) => arena.alloc(Value::Array(
                Box::new_in(array.as_ref().clone(), arena),
                flags,
            )),
            _ => panic!("Not an array"),
        }
    }

    // Prints out the value as JSON string.
    pub fn dump(&'a self) -> String {
        let mut gen = DumpSerializer::new();
        gen.write_json(self).expect("Can't fail");
        gen.consume()
    }

    pub fn pretty(&'a self, spaces: u16) -> String {
        let mut gen = PrettySerializer::new(spaces);
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

impl<'a> Index<usize> for Value<'a> {
    type Output = Value<'a>;

    fn index(&self, index: usize) -> &Self::Output {
        match *self {
            Value::Array(ref a, _) => match a.get(index) {
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
            Self::Number(n) => n.fmt(f),
            Self::Bool(b) => b.fmt(f),
            Self::String(s) => s.fmt(f),
            Self::Array(a, _) => a.fmt(f),
            Self::Object(o) => o.fmt(f),
            Self::Lambda { .. } => write!(f, "<lambda>"),
            Self::NativeFn { .. } => write!(f, "<nativefn>"),
        }
    }
}

impl std::string::ToString for Value<'_> {
    fn to_string(&self) -> String {
        match self {
            Self::Undefined => "undefined".to_string(),
            Self::Null => "null".to_string(),
            Self::Number(n) => n.to_string(),
            Self::Bool(b) => b.to_string(),
            Self::String(s) => s.clone(),
            Self::Array(..) => "<array>".to_string(),
            Self::Object(..) => "<object>".to_string(),
            Self::Lambda { .. } => "<lambda>".to_string(),
            Self::NativeFn { .. } => "<nativefn>".to_string(),
        }
    }
}
