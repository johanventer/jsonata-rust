use std::convert::TryFrom;
use std::marker::PhantomData;
use std::ops::Deref;
use std::{collections::HashMap, fmt};

mod kind;
mod pool;

pub use kind::{ArrayFlags, ValueKind};
pub use pool::ValuePool;

use crate::ast::{Node, NodeKind};
use crate::json::codegen::{DumpGenerator, Generator, PrettyGenerator};
use crate::node_pool::{NodePool, NodeRef};

/// A thin wrapper around the index to a `ValueKind` within a `ValuePool`.
///
/// `Value`s are intended to be created and dropped as needed without having any
/// effect on the underlying data stored in the `ValuePool`. They are essentially
/// pointers to nodes in the pool, that contain a reference to the pool and the
/// index of a node in the pool.
///
/// As a `Value` is just an `Rc` and a `usize`, it has a Clone implementation which
/// makes it very easy to pass around.
#[derive(Clone, Debug)]
pub struct Value {
    pub pool: ValuePool,
    pub index: usize,
}

impl Value {
    #[inline]
    pub fn is_undefined(&self) -> bool {
        matches!(self.pool.borrow().get(self.index), ValueKind::Undefined)
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self.pool.borrow().get(self.index), ValueKind::Null)
    }

    #[inline]
    pub fn is_bool(&self) -> bool {
        matches!(self.pool.borrow().get(self.index), ValueKind::Bool(..))
    }

    #[inline]
    pub fn is_number(&self) -> bool {
        matches!(self.pool.borrow().get(self.index), ValueKind::Number(..))
    }

    #[inline]
    pub fn is_usize(&self) -> bool {
        matches!(self.pool.borrow().get(self.index), ValueKind::Number(n) if usize::try_from(*n).is_ok())
    }

    #[inline]
    pub fn is_nan(&self) -> bool {
        matches!(self.pool.borrow().get(self.index), ValueKind::Number(n) if n.is_nan())
    }

    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self.pool.borrow().get(self.index), ValueKind::String(..))
    }

    #[inline]
    pub fn is_array(&self) -> bool {
        matches!(self.pool.borrow().get(self.index), ValueKind::Array(..))
    }

    #[inline]
    pub fn is_object(&self) -> bool {
        matches!(self.pool.borrow().get(self.index), ValueKind::Object(..))
    }

    #[inline]
    pub fn is_function(&self) -> bool {
        matches!(
            self.pool.borrow().get(self.index),
            ValueKind::Lambda { .. }
                | ValueKind::NativeFn0(..)
                | ValueKind::NativeFn1(..)
                | ValueKind::NativeFn2(..)
                | ValueKind::NativeFn3(..)
        )
    }

    pub fn is_truthy(&self) -> bool {
        match self.pool.borrow().get(self.index) {
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
            ValueKind::Lambda(_, _)
            | ValueKind::NativeFn0(_, _)
            | ValueKind::NativeFn1(_, _)
            | ValueKind::NativeFn2(_, _)
            | ValueKind::NativeFn3(_, _) => false,
        }
    }

    pub fn arity(&self) -> usize {
        match self.pool.borrow().get(self.index) {
            ValueKind::Lambda(
                _,
                Node {
                    kind: NodeKind::Lambda { ref args, .. },
                    ..
                },
            ) => args.len(),
            ValueKind::NativeFn0(..) => 0,
            ValueKind::NativeFn1(..) => 1,
            ValueKind::NativeFn2(..) => 2,
            ValueKind::NativeFn3(..) => 3,
            _ => panic!("Not a function"),
        }
    }

    pub fn as_ref(&self) -> NodeRef<ValueKind> {
        // This looks weird, but I need a way around the borrow checker (both compile-time and the
        // runtime borrow checking on the pool) to get a NodeRef that doesn't borrow the pool for the
        // entirety of its lifetime.

        let pool_ref = self.pool.borrow();
        let pool_ptr = &*pool_ref as *const NodePool<ValueKind>;

        // Safety: The pointer was just created, and will still be valid as long as the pool lives.
        let node_ref = unsafe { pool_ptr.as_ref().unwrap().get_ref(self.index) };

        node_ref
    }

    pub fn as_bool(&self) -> bool {
        match *self.pool.borrow().get(self.index) {
            ValueKind::Bool(b) => b,
            _ => panic!("Not a bool"),
        }
    }

    pub fn as_f32(&self) -> f32 {
        match *self.pool.borrow().get(self.index) {
            ValueKind::Number(n) => n.into(),
            _ => panic!("Not a number"),
        }
    }

    pub fn as_f64(&self) -> f64 {
        match *self.pool.borrow().get(self.index) {
            ValueKind::Number(n) => n.into(),
            _ => panic!("Not a number"),
        }
    }

    pub fn as_usize(&self) -> usize {
        match *self.pool.borrow().get(self.index) {
            ValueKind::Number(n) => {
                usize::try_from(n).unwrap_or_else(|_| panic!("Number is not a valid usize"))
            }
            _ => panic!("Not a number"),
        }
    }

    pub fn as_string(&self) -> String {
        match self.pool.borrow().get(self.index) {
            ValueKind::String(s) => s.clone(),
            _ => panic!("Not a string"),
        }
    }

    pub fn len(&self) -> usize {
        match self.pool.borrow().get(self.index) {
            ValueKind::Array(array, _) => array.len(),
            _ => panic!("Not an array"),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self.pool.borrow().get(self.index) {
            ValueKind::Array(array, _) => array.is_empty(),
            _ => panic!("Not an array"),
        }
    }

    pub fn get_member(&self, index: usize) -> Value {
        match self.pool.borrow().get(self.index) {
            ValueKind::Array(ref array, _) => match array.get(index) {
                Some(index) => Value {
                    pool: self.pool.clone(),
                    index: *index,
                },
                None => self.pool.undefined(),
            },
            _ => panic!("Not an array"),
        }
    }

    pub fn get_entry(&self, key: &str) -> Value {
        match self.pool.borrow().get(self.index) {
            ValueKind::Object(ref map) => match map.get(key) {
                Some(index) => Value {
                    pool: self.pool.clone(),
                    index: *index,
                },
                None => self.pool.undefined(),
            },
            _ => panic!("Not an object"),
        }
    }

    pub fn insert(&self, key: &str, kind: ValueKind) {
        let mut pool = self.pool.borrow_mut();
        let index = pool.insert(kind);
        match pool.get_mut(self.index) {
            ValueKind::Object(ref mut map) => {
                map.insert(key.to_owned(), index);
            }
            _ => panic!("Not an object"),
        }
    }

    pub fn insert_index(&self, key: &str, index: usize) {
        match self.pool.borrow_mut().get_mut(self.index) {
            ValueKind::Object(ref mut map) => {
                map.insert(key.to_owned(), index);
            }
            _ => panic!("Not an object"),
        }
    }

    /// Pushes a new `ValueKind` into the `ValueKind::Array` wrapped by this `Value`.
    ///
    /// # Panics
    ///
    /// If the `ValueKind` wrapped by this `Value` is anot a `ValueKind::Array`.
    pub fn push(&self, kind: ValueKind) {
        let mut pool = self.pool.borrow_mut();
        let index = pool.insert(kind);
        match pool.get_mut(self.index) {
            ValueKind::Array(ref mut array, _) => {
                array.push(index);
            }
            _ => panic!("Not an array"),
        }
    }

    /// Pushes a pool index into the `ValueKind::Array` wrapped by this `Value`.
    ///
    /// Use this if you have constructed a Value separately and now want it to be an item
    /// in an existing `ValueKind::Array`.
    ///
    /// Note: This makes absolutely no attempt to a) verify the the index is valid, b)
    /// that it even came from the same `ValuePool`, or c) that this does not create a circular
    /// reference.
    ///
    /// # Panics
    ///
    /// If the `ValueKind` wrapped by this `Value` is not a `ValueKind::Array`.
    pub fn push_index(&self, index: usize) {
        match self.pool.borrow_mut().get_mut(self.index) {
            ValueKind::Array(ref mut array, _) => {
                array.push(index);
            }
            _ => panic!("Not an array"),
        }
    }

    /// Wraps an existing value in an array.
    pub fn wrap_in_array(&self, flags: ArrayFlags) -> Value {
        let array = self.pool.array_with_capacity(1, flags);
        array.push_index(self.index);
        array
    }

    /// Wraps an existing value in an array if it's not already an array.
    pub fn wrap_in_array_if_needed(&self, flags: ArrayFlags) -> Value {
        if self.is_array() {
            self.clone()
        } else {
            self.wrap_in_array(flags)
        }
    }

    /// Create an iterator over the members of an array.
    ///
    /// # Panics
    ///
    /// If the `ValueKind` wrapped by this `Value` is not a `ValueKind::Array`.
    pub fn members(&self) -> Members {
        match self.pool.borrow().get(self.index) {
            ValueKind::Array(ref array, _) => unsafe { Members::new(&self.pool, array) },
            _ => panic!("Not an array"),
        }
    }

    /// Create an iterator over the entries of an object.
    ///
    /// # Panics
    ///
    /// If the `ValueKind` wrapped by this `Value` is not a `ValueKind::Object`.
    pub fn entries(&self) -> Entries {
        match self.pool.borrow().get(self.index) {
            ValueKind::Object(ref map) => unsafe { Entries::new(&self.pool, map) },
            _ => panic!("Not an object"),
        }
    }

    pub fn get_flags(&self) -> ArrayFlags {
        match self.pool.borrow().get(self.index) {
            ValueKind::Array(_, flags) => *flags,
            _ => panic!("Not an array"),
        }
    }

    pub fn set_flags(&self, new_flags: ArrayFlags) {
        match self.pool.borrow_mut().get_mut(self.index) {
            ValueKind::Array(_, ref mut flags) => *flags = new_flags,
            _ => panic!("Not an array"),
        }
    }

    pub fn add_flags(&self, flags_to_add: ArrayFlags) {
        match self.pool.borrow_mut().get_mut(self.index) {
            ValueKind::Array(_, ref mut flags) => flags.insert(flags_to_add),
            _ => panic!("Not an array"),
        }
    }

    pub fn has_flags(&self, check_flags: ArrayFlags) -> bool {
        match self.pool.borrow().get(self.index) {
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

/// Compares two `Value`s for equality by comparing their underlying `ValueKind`s.
///
/// Delegates comparison to the ValueKind instance in the pool, so you can
/// directly compare `Value`s to determine if their underlying `ValueKind`s are equal.
impl PartialEq<Value> for Value {
    fn eq(&self, other: &Value) -> bool {
        match self.pool.borrow().get(self.index) {
            ValueKind::Array(..) => {
                if other.is_array() && other.len() == self.len() {
                    self.members().zip(other.members()).all(|(l, r)| l == r)
                } else {
                    false
                }
            }
            ValueKind::Object(..) => {
                if other.is_object() {
                    self.entries().all(|(k, v)| v == other.get_entry(k))
                } else {
                    false
                }
            }
            _ => self.pool.borrow().get(self.index) == self.pool.borrow().get(other.index),
        }
    }
}

impl PartialEq<ValueKind> for Value {
    fn eq(&self, other: &ValueKind) -> bool {
        self.pool.borrow().get(self.index) == other
    }
}

impl PartialEq<bool> for Value {
    fn eq(&self, other: &bool) -> bool {
        match self.pool.borrow().get(self.index) {
            ValueKind::Bool(ref b) => *b == *other,
            _ => false,
        }
    }
}

impl PartialEq<i32> for Value {
    fn eq(&self, other: &i32) -> bool {
        match self.pool.borrow().get(self.index) {
            ValueKind::Number(ref n) => *n == *other,
            _ => false,
        }
    }
}

impl PartialEq<i64> for Value {
    fn eq(&self, other: &i64) -> bool {
        match self.pool.borrow().get(self.index) {
            ValueKind::Number(ref n) => *n == *other,
            _ => false,
        }
    }
}

impl PartialEq<f32> for Value {
    fn eq(&self, other: &f32) -> bool {
        match self.pool.borrow().get(self.index) {
            ValueKind::Number(ref n) => *n == *other,
            _ => false,
        }
    }
}

impl PartialEq<f64> for Value {
    fn eq(&self, other: &f64) -> bool {
        match self.pool.borrow().get(self.index) {
            ValueKind::Number(ref n) => *n == *other,
            _ => false,
        }
    }
}

impl PartialEq<&str> for Value {
    fn eq(&self, other: &&str) -> bool {
        match self.pool.borrow().get(self.index) {
            ValueKind::String(s) => *s == **other,
            _ => false,
        }
    }
}

impl PartialEq<String> for Value {
    fn eq(&self, other: &String) -> bool {
        match self.pool.borrow().get(self.index) {
            ValueKind::String(s) => *s == *other,
            _ => false,
        }
    }
}

pub struct Members<'a> {
    pool: &'a ValuePool,
    inner: std::slice::Iter<'a, usize>,
}

impl<'a> Members<'a> {
    /// # Safety
    /// The iterator's lifetime is tied to the pool, and as long as the array is not
    /// removed from the pool during the lifetime of this iterator then this is safe.
    pub unsafe fn new(pool: &'a ValuePool, array: *const Vec<usize>) -> Self {
        Self {
            pool,
            inner: (*array).iter(),
        }
    }
}

impl<'a> Iterator for Members<'a> {
    type Item = Value;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|index| Value {
            pool: self.pool.clone(),
            index: *index,
        })
    }
}

pub struct Entries<'a> {
    pool: &'a ValuePool,
    inner: std::collections::hash_map::Iter<'a, String, usize>,
}

impl<'a> Entries<'a> {
    /// # Safety
    /// The iterator's lifetime is tied to the pool, and as long as the map is not
    /// removed from the pool during the lifetime of this iterator then this is safe.
    pub unsafe fn new(pool: &'a ValuePool, map: *const HashMap<String, usize>) -> Self {
        Self {
            pool,
            inner: (*map).iter(),
        }
    }
}

impl<'a> Iterator for Entries<'a> {
    type Item = (&'a String, Value);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(key, index)| {
            (
                key,
                Value {
                    pool: self.pool.clone(),
                    index: *index,
                },
            )
        })
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            f.write_str(&self.pretty(4))
        } else {
            match self.pool.borrow().get(self.index) {
                ValueKind::String(ref value) => value.fmt(f),
                ValueKind::Number(ref value) => value.fmt(f),
                ValueKind::Bool(ref value) => value.fmt(f),
                ValueKind::Null => f.write_str("null"),
                _ => f.write_str(&self.dump()),
            }
        }
    }
}

// FIXME: This is going to break if the pool grows as it will reallocate and
// the pointer will no longer be correct
pub struct ValueRef<'a> {
    pool: PhantomData<&'a ValuePool>,
    kind: *const ValueKind,
}

impl Deref for ValueRef<'_> {
    type Target = ValueKind;

    fn deref(&self) -> &Self::Target {
        // SAFETY: The reference's lifetime is tied to the pool, as long the ValueKind is not
        // removed from the pool then this is safe.
        unsafe { &*self.kind }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn members_iter() {
        let pool = ValuePool::new();
        let a = pool.array(ArrayFlags::empty());
        a.push(ValueKind::Number(5.into()));
        a.push(ValueKind::Number(4.into()));
        a.push(ValueKind::Number(3.into()));
        a.push(ValueKind::Number(2.into()));
        a.push(ValueKind::Number(1.into()));
        let mut iter = a.members();
        assert!((5.0 - iter.next().unwrap().as_f64()).abs() < f64::EPSILON);
        assert!((4.0 - iter.next().unwrap().as_f64()).abs() < f64::EPSILON);
        assert!((3.0 - iter.next().unwrap().as_f64()).abs() < f64::EPSILON);
        assert!((2.0 - iter.next().unwrap().as_f64()).abs() < f64::EPSILON);
        assert!((1.0 - iter.next().unwrap().as_f64()).abs() < f64::EPSILON);
        assert!(iter.next().is_none());
    }

    #[test]
    fn entries_iter() {
        let map = HashMap::from([("a", "1"), ("b", "2"), ("c", "3"), ("d", "4"), ("e", "5")]);
        let pool = ValuePool::new();
        let o = pool.object();
        map.iter().for_each(|(k, v)| o.insert(*k, (*v).into()));
        let entries: Vec<(String, String)> = o
            .entries()
            .map(|(k, v)| (k.clone(), v.as_string()))
            .collect();
        let mut result: HashMap<&str, &str> = HashMap::new();
        entries.iter().for_each(|(k, v)| {
            result.insert(k, v);
        });
        assert_eq!(map, result);
    }

    #[test]
    fn wrap_in_array() {
        let pool = ValuePool::new();
        let v = pool.string("hello world");
        let v = v.wrap_in_array(ArrayFlags::empty());
        assert!(v.is_array());
        assert_eq!(v.get_member(0).as_string(), "hello world");
    }
}
