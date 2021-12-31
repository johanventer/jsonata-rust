use std::{collections::HashMap, fmt};

mod kind;
mod pool;

pub use kind::ValueKind;
pub use pool::ValuePool;

use crate::json::Number;
use kind::ArrayProps;

/// A thin wrapper around the index to a `ValueKind` within a `ValuePool`.
///
/// `Value`s are intended to be created and dropped as needed without having any
/// effect on the underlying data stored in the `ValuePool`. They are essentially
/// pointers to nodes in the pool, that contain a reference to the pool and the
/// index of a node in the pool.
///
/// As a `Value` is just an `Rc` and a `usize`, it has a Clone implementation which
/// makes it very easy to pass around.
#[derive(Clone)]
pub struct Value {
    pub pool: ValuePool,
    pub index: usize,
}

impl Value {
    pub fn new(pool: ValuePool, kind: ValueKind) -> Value {
        let index = pool.borrow_mut().insert(kind);
        Value { pool, index }
    }

    pub fn new_undefined(pool: ValuePool) -> Value {
        let index = pool.borrow_mut().insert(ValueKind::Undefined);
        Value { pool, index }
    }

    pub fn new_null(pool: ValuePool) -> Value {
        let index = pool.borrow_mut().insert(ValueKind::Null);
        Value { pool, index }
    }

    pub fn new_bool(pool: ValuePool, value: bool) -> Value {
        let index = pool.borrow_mut().insert(ValueKind::Bool(value));
        Value { pool, index }
    }

    pub fn new_number(pool: ValuePool, value: Number) -> Value {
        let index = pool.borrow_mut().insert(ValueKind::Number(value));
        Value { pool, index }
    }

    pub fn new_string(pool: ValuePool, value: &str) -> Value {
        let index = pool
            .borrow_mut()
            .insert(ValueKind::String(value.to_owned()));
        Value { pool, index }
    }

    pub fn new_array(pool: ValuePool) -> Value {
        let index = pool
            .borrow_mut()
            .insert(ValueKind::Array(Vec::new(), ArrayProps::default()));
        Value { pool, index }
    }

    pub fn new_array_with_capacity(pool: ValuePool, capacity: usize) -> Value {
        let index = pool.borrow_mut().insert(ValueKind::Array(
            Vec::with_capacity(capacity),
            ArrayProps::default(),
        ));
        Value { pool, index }
    }

    pub fn new_object(pool: ValuePool) -> Value {
        let index = pool.borrow_mut().insert(ValueKind::Object(HashMap::new()));
        Value { pool, index }
    }

    pub fn new_object_with_capacity(pool: ValuePool, capacity: usize) -> Value {
        let index = pool
            .borrow_mut()
            .insert(ValueKind::Object(HashMap::with_capacity(capacity)));
        Value { pool, index }
    }

    pub fn is_undefined(&self) -> bool {
        matches!(self.pool.borrow().get(self.index), ValueKind::Undefined)
    }

    pub fn is_null(&self) -> bool {
        matches!(self.pool.borrow().get(self.index), ValueKind::Null)
    }

    pub fn is_bool(&self) -> bool {
        matches!(self.pool.borrow().get(self.index), ValueKind::Bool(..))
    }

    pub fn is_number(&self) -> bool {
        matches!(self.pool.borrow().get(self.index), ValueKind::Number(..))
    }

    pub fn is_string(&self) -> bool {
        matches!(self.pool.borrow().get(self.index), ValueKind::String(..))
    }

    pub fn is_array(&self) -> bool {
        matches!(self.pool.borrow().get(self.index), ValueKind::Array(..))
    }

    pub fn is_object(&self) -> bool {
        matches!(self.pool.borrow().get(self.index), ValueKind::Object(..))
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

    pub fn as_string(&self) -> String {
        match self.pool.borrow().get(self.index) {
            ValueKind::String(s) => s.clone(),
            _ => panic!("Not a string"),
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

    /// Removes the actual `ValueKind` wrapped by the `Value` from the pool and consumes the `Value`.
    ///
    /// As `Value`s are thin wrappers around an index in the pool, and there could be other
    /// `Value`s referencing the indexed `ValueKind`, this is not in a real `Drop` implementation.
    ///
    /// "Dropping" in this case actually means removing the ValueKind from the pool and putting
    /// its index on the free list.
    pub fn drop(self) {
        self.pool.borrow_mut().remove(self.index);
        drop(self);
    }

    /// Wraps an existing value in an array by creating a new array in the pool and adding this
    /// `Value`'s index as the only item.
    ///
    /// This is used extensively throughout evaluation to implement the rules of sequences, and
    /// one of the actions that has a major advantage using the pool implementation. If there wasn't
    /// a pool, the entire input would need to be cloned which could be very costly.
    pub fn wrap_in_array(self) -> Value {
        let array = Value::new_array_with_capacity(self.pool, 1);
        array.push_index(self.index);
        array
    }

    /// Create an iterator over the members of an array.
    ///
    /// # Panics
    ///
    /// If the `ValueKind` wrapped by this `Value` is not a `ValueKind::Array`.
    pub fn members(&self) -> Members {
        match self.pool.borrow().get(self.index) {
            ValueKind::Array(ref array, _) => Members::new(&self.pool, array),
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
}

/// Compares two `Value`s for equality by comparing their underlying `ValueKind`s.
///
/// Delegates comparison to the ValueKind instance in the pool, so you can
/// directly compare `Value`s to determine if their underlying `ValueKind`s are equal.
impl PartialEq<Value> for Value {
    fn eq(&self, other: &Value) -> bool {
        self.pool.borrow().get(self.index) == self.pool.borrow().get(other.index)
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
            ValueKind::String(s) => s == *other,
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

pub struct Members<'pool> {
    pool: &'pool ValuePool,
    array: *const Vec<usize>,
    index: usize,
}

impl<'pool> Members<'pool> {
    pub fn new(pool: &'pool ValuePool, array: *const Vec<usize>) -> Self {
        Self {
            pool,
            array,
            index: 0,
        }
    }
}

impl<'pool> Iterator for Members<'pool> {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: The iterator's lifetime is tied to the pool, and as long as the array is not
        // removed from the pool during the lifetime of the iterator then this is safe.
        let array = unsafe { &*self.array };

        if array.is_empty() || self.index > array.len() - 1 {
            None
        } else {
            let next = Some(Value {
                pool: self.pool.clone(),
                index: array[self.index],
            });

            self.index += 1;

            next
        }
    }
}

pub struct Entries<'pool> {
    pool: &'pool ValuePool,
    inner: std::collections::hash_map::Iter<'pool, String, usize>,
}

impl<'pool> Entries<'pool> {
    /// # Safety
    /// The iterator's lifetime is tied to the pool, and as long as the map is not
    /// removed from the pool during the lifetime of this iterator then this is safe.
    pub unsafe fn new(pool: &'pool ValuePool, map: *const HashMap<String, usize>) -> Self {
        Self {
            pool,
            inner: (*map).iter(),
        }
    }
}

impl<'pool> Iterator for Entries<'pool> {
    type Item = (&'pool String, Value);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.next();
        match next {
            Some((k, v)) => Some((
                k,
                Value {
                    pool: self.pool.clone(),
                    index: *v,
                },
            )),
            None => None,
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.pool.borrow().get(self.index) {
            ValueKind::Array(..) => f.debug_list().entries(self.members()).finish(),
            ValueKind::Object(..) => f.debug_map().entries(self.entries()).finish(),
            _ => ValueKind::fmt(self.pool.borrow().get(self.index), f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn members_iter() {
        let pool = ValuePool::new();
        let a = Value::new_array(pool);
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
        let o = Value::new_object(pool);
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
        let v = Value::new_string(pool, "hello world");
        let v = v.wrap_in_array();
        assert!(v.is_array());
        assert_eq!(v.get_member(0).as_string(), "hello world");
    }
}
