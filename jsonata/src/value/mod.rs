mod kind;

pub use kind::{ArrayFlags, ValueKind};

use bumpalo::Bump;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;

use crate::ast::{Ast, AstKind};
use crate::frame::Frame;
use crate::functions::FunctionContext;
use crate::json::codegen::{DumpGenerator, Generator, PrettyGenerator};
use crate::json::Number;
use crate::Result;

const _UNDEFINED: ValueKind = ValueKind::Undefined;
pub const UNDEFINED: Value = Value(&_UNDEFINED);

thread_local! {
    static ARENA: Bump = Bump::new();
}

#[derive(Clone, Copy)]
pub struct Value(*const ValueKind);

impl Value {
    // pub fn ptr(&self) -> *const ValueKind {
    //     self.0
    // }

    pub fn new(kind: ValueKind) -> Value {
        Value(ARENA.with(|arena| arena.alloc(kind) as *const ValueKind))
    }

    pub fn null() -> Value {
        Value(ARENA.with(|arena| arena.alloc(ValueKind::Null) as *const ValueKind))
    }

    pub fn bool(value: bool) -> Value {
        Value(ARENA.with(|arena| arena.alloc(ValueKind::Bool(value)) as *const ValueKind))
    }

    pub fn number<T: Into<Number>>(value: T) -> Value {
        Value(ARENA.with(|arena| arena.alloc(ValueKind::Number(value.into())) as *const ValueKind))
    }

    pub fn string<T: Into<String>>(value: T) -> Value {
        Value(ARENA.with(|arena| arena.alloc(ValueKind::String(value.into())) as *const ValueKind))
    }

    pub fn array(flags: ArrayFlags) -> Value {
        Value(
            ARENA
                .with(|arena| arena.alloc(ValueKind::Array(Vec::new(), flags)) as *const ValueKind),
        )
    }

    pub fn array_with_capacity(capacity: usize, flags: ArrayFlags) -> Value {
        Value(ARENA.with(|arena| {
            arena.alloc(ValueKind::Array(Vec::with_capacity(capacity), flags)) as *const ValueKind
        }))
    }

    pub fn object() -> Value {
        Value(
            ARENA.with(|arena| arena.alloc(ValueKind::Object(HashMap::new())) as *const ValueKind),
        )
    }

    pub fn object_with_capacity(capacity: usize) -> Value {
        Value(ARENA.with(|arena| {
            arena.alloc(ValueKind::Object(HashMap::with_capacity(capacity))) as *const ValueKind
        }))
    }

    pub fn lambda(name: &str, node: Ast, input: Value, frame: Frame) -> Value {
        Value(ARENA.with(|arena| {
            arena.alloc(ValueKind::Lambda {
                name: name.to_string(),
                ast: node,
                input,
                frame,
            }) as *const ValueKind
        }))
    }

    pub fn nativefn0(name: &str, func: fn(&FunctionContext) -> Result<Value>) -> Value {
        Value(ARENA.with(|arena| {
            arena.alloc(ValueKind::NativeFn0(name.to_string(), func)) as *const ValueKind
        }))
    }

    pub fn nativefn1(name: &str, func: fn(&FunctionContext, Value) -> Result<Value>) -> Value {
        Value(ARENA.with(|arena| {
            arena.alloc(ValueKind::NativeFn1(name.to_string(), func)) as *const ValueKind
        }))
    }

    pub fn nativefn2(
        name: &str,
        func: fn(&FunctionContext, Value, Value) -> Result<Value>,
    ) -> Value {
        Value(ARENA.with(|arena| {
            arena.alloc(ValueKind::NativeFn2(name.to_string(), func)) as *const ValueKind
        }))
    }

    pub fn nativefn3(
        name: &str,
        func: fn(&FunctionContext, Value, Value, Value) -> Result<Value>,
    ) -> Value {
        Value(ARENA.with(|arena| {
            arena.alloc(ValueKind::NativeFn3(name.to_string(), func)) as *const ValueKind
        }))
    }

    pub fn is_undefined(&self) -> bool {
        matches!(unsafe { &*self.0 }, ValueKind::Undefined)
    }

    pub fn is_null(&self) -> bool {
        matches!(unsafe { &*self.0 }, ValueKind::Null)
    }

    pub fn is_bool(&self) -> bool {
        matches!(unsafe { &*self.0 }, ValueKind::Bool(..))
    }

    pub fn is_number(&self) -> bool {
        matches!(unsafe { &*self.0 }, ValueKind::Number(..))
    }

    pub fn is_integer(&self) -> bool {
        if let ValueKind::Number(ref n) = unsafe { &*self.0 } {
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
        matches!(unsafe { &*self.0 }, ValueKind::Number(n) if n.is_nan())
    }

    pub fn is_string(&self) -> bool {
        matches!(unsafe { &*self.0 }, ValueKind::String(..))
    }

    pub fn is_array(&self) -> bool {
        matches!(unsafe { &*self.0 }, ValueKind::Array(..))
    }

    pub fn is_object(&self) -> bool {
        matches!(unsafe { &*self.0 }, ValueKind::Object(..))
    }

    pub fn is_function(&self) -> bool {
        matches!(
            unsafe { &*self.0 },
            ValueKind::Lambda { .. }
                | ValueKind::NativeFn0(..)
                | ValueKind::NativeFn1(..)
                | ValueKind::NativeFn2(..)
                | ValueKind::NativeFn3(..)
        )
    }

    pub fn is_truthy(&self) -> bool {
        match unsafe { &*self.0 } {
            ValueKind::Undefined => false,
            ValueKind::Null => false,
            ValueKind::Number(n) => *n != 0.0,
            ValueKind::Bool(b) => *b,
            ValueKind::String(s) => !s.is_empty(),
            ValueKind::Array(a, _) => match a.len() {
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
            ValueKind::Object(o) => !o.is_empty(),
            ValueKind::Lambda { .. }
            | ValueKind::NativeFn0(..)
            | ValueKind::NativeFn1(..)
            | ValueKind::NativeFn2(..)
            | ValueKind::NativeFn3(..) => false,
        }
    }

    pub fn arity(&self) -> usize {
        match unsafe { &*self.0 } {
            ValueKind::Lambda {
                ast:
                    Ast {
                        kind: AstKind::Lambda { ref args, .. },
                        ..
                    },
                ..
            } => args.len(),
            ValueKind::NativeFn0(..) => 0,
            ValueKind::NativeFn1(..) => 1,
            ValueKind::NativeFn2(..) => 2,
            ValueKind::NativeFn3(..) => 3,
            _ => panic!("Not a function"),
        }
    }

    pub fn as_bool(&self) -> bool {
        match unsafe { &*self.0 } {
            ValueKind::Bool(b) => *b,
            _ => panic!("Not a bool"),
        }
    }

    pub fn as_f32(&self) -> f32 {
        match unsafe { &*self.0 } {
            ValueKind::Number(n) => f32::from(*n),
            _ => panic!("Not a number"),
        }
    }

    pub fn as_f64(&self) -> f64 {
        match unsafe { &*self.0 } {
            ValueKind::Number(n) => f64::from(*n),
            _ => panic!("Not a number"),
        }
    }

    pub fn as_usize(&self) -> usize {
        match unsafe { &*self.0 } {
            ValueKind::Number(ref n) => f64::from(*n) as usize,
            _ => panic!("Not a number"),
        }
    }

    pub fn as_isize(&self) -> isize {
        match unsafe { &*self.0 } {
            ValueKind::Number(ref n) => f64::from(*n) as isize,
            _ => panic!("Not a number"),
        }
    }

    pub fn as_str(&self) -> Cow<'_, str> {
        match unsafe { &*self.0 } {
            ValueKind::String(ref s) => Cow::from(s),
            _ => panic!("Not a string"),
        }
    }

    pub fn len(&self) -> usize {
        match unsafe { &*self.0 } {
            ValueKind::Array(array, _) => array.len(),
            _ => panic!("Not an array"),
        }
    }

    pub fn is_empty(&self) -> bool {
        match unsafe { &*self.0 } {
            ValueKind::Array(array, _) => array.is_empty(),
            _ => panic!("Not an array"),
        }
    }

    pub fn get_member(&self, index: usize) -> Value {
        match unsafe { &*self.0 } {
            ValueKind::Array(ref array, _) => match array.get(index) {
                Some(value) => *value,
                None => UNDEFINED,
            },
            _ => panic!("Not an array"),
        }
    }

    pub fn get_entry(&self, key: &str) -> Value {
        match unsafe { &*self.0 } {
            ValueKind::Object(ref map) => match map.get(key) {
                Some(value) => *value,
                None => UNDEFINED,
            },
            _ => panic!("Not an object"),
        }
    }

    pub fn insert_new(&self, key: &str, kind: ValueKind) {
        match unsafe { &mut *(self.0 as *mut ValueKind) } {
            ValueKind::Object(ref mut map) => {
                let kind = ARENA.with(|arena| arena.alloc(kind) as *const ValueKind);
                map.insert(key.to_owned(), Value(kind));
            }
            _ => panic!("Not an object"),
        }
    }

    pub fn insert(&self, key: &str, value: Value) {
        match unsafe { &mut *(self.0 as *mut ValueKind) } {
            ValueKind::Object(ref mut map) => {
                map.insert(key.to_owned(), value);
            }
            _ => panic!("Not an object"),
        }
    }

    pub fn push_new(&self, kind: ValueKind) {
        match unsafe { &mut *(self.0 as *mut ValueKind) } {
            ValueKind::Array(ref mut array, _) => {
                let kind = ARENA.with(|arena| arena.alloc(kind) as *const ValueKind);
                array.push(Value(kind));
            }
            _ => panic!("Not an array"),
        }
    }

    pub fn push(&self, value: Value) {
        match unsafe { &mut *(self.0 as *mut ValueKind) } {
            ValueKind::Array(ref mut array, _) => array.push(value),
            _ => panic!("Not an array"),
        }
    }

    pub fn wrap_in_array(&self, flags: ArrayFlags) -> Value {
        let array = vec![*self];
        let result =
            ARENA.with(|arena| arena.alloc(ValueKind::Array(array, flags)) as *const ValueKind);
        Value(result)
    }

    pub fn wrap_in_array_if_needed(&self, flags: ArrayFlags) -> Value {
        if self.is_array() {
            *self
        } else {
            self.wrap_in_array(flags)
        }
    }

    pub fn members(&self) -> std::slice::Iter<'_, Value> {
        match unsafe { &*self.0 } {
            ValueKind::Array(ref array, _) => array.iter(),
            _ => panic!("Not an array"),
        }
    }

    pub fn entries(&self) -> std::collections::hash_map::Iter<'_, String, Value> {
        match unsafe { &*self.0 } {
            ValueKind::Object(ref map) => map.iter(),
            _ => panic!("Not an object"),
        }
    }

    pub fn get_flags(&self) -> ArrayFlags {
        match unsafe { &*self.0 } {
            ValueKind::Array(_, flags) => *flags,
            _ => panic!("Not an array"),
        }
    }

    pub fn set_flags(&mut self, new_flags: ArrayFlags) {
        match unsafe { &mut *(self.0 as *mut ValueKind) } {
            ValueKind::Array(_, flags) => *flags = new_flags,
            _ => panic!("Not an array"),
        }
    }

    pub fn add_flags(&mut self, flags_to_add: ArrayFlags) {
        match unsafe { &mut *(self.0 as *mut ValueKind) } {
            ValueKind::Array(_, flags) => flags.insert(flags_to_add),
            _ => panic!("Not an array"),
        }
    }

    pub fn has_flags(&self, check_flags: ArrayFlags) -> bool {
        match unsafe { &*self.0 } {
            ValueKind::Array(_, flags) => flags.contains(check_flags),
            _ => false,
        }
    }

    // Prints out the value as JSON string.
    pub fn dump(&self) -> String {
        let mut gen = DumpGenerator::new();
        gen.write_json(self).expect("Can't fail");
        gen.consume()
    }

    /// Pretty prints out the value as JSON string. Takes an argument that's
    /// number of spaces to indent new blocks with.
    pub fn pretty(&self, spaces: u16) -> String {
        let mut gen = PrettyGenerator::new(spaces);
        gen.write_json(self).expect("Can't fail");
        gen.consume()
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.dump())
    }
}

impl Deref for Value {
    type Target = ValueKind;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

/// Compares two `Value`s for equality by comparing their underlying `ValueKind`s.
///
/// Delegates comparison to the ValueKind instance in the arena, so you can
/// directly compare `Value`s to determine if their underlying `ValueKind`s are equal.
impl PartialEq<Value> for Value {
    fn eq(&self, other: &Value) -> bool {
        match unsafe { &*self.0 } {
            ValueKind::Array(..) => {
                if other.is_array() && other.len() == self.len() {
                    self.members().zip(other.members()).all(|(l, r)| l == r)
                } else {
                    false
                }
            }
            ValueKind::Object(..) => {
                if other.is_object() {
                    self.entries().all(|(k, v)| *v == other.get_entry(k))
                } else {
                    false
                }
            }
            _ => unsafe { *self.0 == **other },
        }
    }
}

impl PartialEq<ValueKind> for Value {
    fn eq(&self, other: &ValueKind) -> bool {
        unsafe { *self.0 == *other }
    }
}

impl PartialEq<bool> for Value {
    fn eq(&self, other: &bool) -> bool {
        match unsafe { &*self.0 } {
            ValueKind::Bool(ref b) => *b == *other,
            _ => false,
        }
    }
}

impl PartialEq<i32> for Value {
    fn eq(&self, other: &i32) -> bool {
        match unsafe { &*self.0 } {
            ValueKind::Number(ref n) => *n == *other,
            _ => false,
        }
    }
}

impl PartialEq<i64> for Value {
    fn eq(&self, other: &i64) -> bool {
        match unsafe { &*self.0 } {
            ValueKind::Number(ref n) => *n == *other,
            _ => false,
        }
    }
}

impl PartialEq<f32> for Value {
    fn eq(&self, other: &f32) -> bool {
        match unsafe { &*self.0 } {
            ValueKind::Number(ref n) => *n == *other,
            _ => false,
        }
    }
}

impl PartialEq<f64> for Value {
    fn eq(&self, other: &f64) -> bool {
        match unsafe { &*self.0 } {
            ValueKind::Number(n) => *n == *other,
            _ => false,
        }
    }
}

impl PartialEq<&str> for Value {
    fn eq(&self, other: &&str) -> bool {
        match unsafe { &*self.0 } {
            ValueKind::String(s) => *s == **other,
            _ => false,
        }
    }
}

impl PartialEq<String> for Value {
    fn eq(&self, other: &String) -> bool {
        match unsafe { &*self.0 } {
            ValueKind::String(s) => *s == *other,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn members_iter() {
    //     let mut a = arena.array(ArrayFlags::empty());
    //     a.push_new(ValueKind::Number(5.into()));
    //     a.push_new(ValueKind::Number(4.into()));
    //     a.push_new(ValueKind::Number(3.into()));
    //     a.push_new(ValueKind::Number(2.into()));
    //     a.push_new(ValueKind::Number(1.into()));
    //     let mut iter = a.members();
    //     assert!((5.0 - iter.next().unwrap().as_f64()).abs() < f64::EPSILON);
    //     assert!((4.0 - iter.next().unwrap().as_f64()).abs() < f64::EPSILON);
    //     assert!((3.0 - iter.next().unwrap().as_f64()).abs() < f64::EPSILON);
    //     assert!((2.0 - iter.next().unwrap().as_f64()).abs() < f64::EPSILON);
    //     assert!((1.0 - iter.next().unwrap().as_f64()).abs() < f64::EPSILON);
    //     assert!(iter.next().is_none());
    // }

    // #[test]
    // fn entries_iter() {
    //     let map = HashMap::from([("a", "1"), ("b", "2"), ("c", "3"), ("d", "4"), ("e", "5")]);
    //     let arena = ValueArena::new();
    //     let mut o = arena.object();
    //     map.iter().for_each(|(k, v)| o.insert_new(*k, (*v).into()));
    //     let entries: Vec<(String, String)> = o
    //         .entries()
    //         .map(|(k, v)| (k.clone(), v.as_str().to_string()))
    //         .collect();
    //     let mut result: HashMap<&str, &str> = HashMap::new();
    //     entries.iter().for_each(|(k, v)| {
    //         result.insert(k, v);
    //     });
    //     assert_eq!(map, result);
    // }

    // #[test]
    // fn wrap_in_array() {
    //     let arena = ValueArena::new();
    //     let v = arena.string(String::from("hello world"));
    //     let v = v.wrap_in_array(ArrayFlags::empty());
    //     assert!(v.is_array());
    //     assert_eq!(v.get_member(0).as_str(), "hello world");
    // }
}
