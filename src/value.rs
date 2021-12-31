use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
};

use super::json::Number;
use super::node_pool::NodePool;

#[derive(Default, Debug)]
pub struct ArrayProps {
    is_sequence: bool,
    keep_singleton: bool,
    cons: bool,
}

#[derive(Debug)]
pub enum ValueKind {
    Undefined,
    Null,
    Number(Number),
    Bool(bool),
    String(String),
    Array(Vec<usize>, ArrayProps),
    Object(HashMap<String, usize>),
}

impl PartialEq<ValueKind> for ValueKind {
    fn eq(&self, other: &ValueKind) -> bool {
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

// A reference counted `NodePool` of `ValueKind`.
//
// The tree structure of both JSON input and evaluation results is represented
// in the pool as a flat list of `ValueKind` where children are referenced by index.
#[derive(Debug)]
pub struct ValuePool(Rc<RefCell<NodePool<ValueKind>>>);

impl ValuePool {
    pub fn new() -> ValuePool {
        let pool = ValuePool(Rc::new(RefCell::new(NodePool::new())));

        // The first index in any ValuePool is undefined
        pool.borrow_mut().insert(ValueKind::Undefined);

        pool
    }

    pub fn undefined(&self) -> Value {
        Value {
            pool: self.clone(),
            index: 0,
        }
    }

    pub fn borrow(&self) -> Ref<'_, NodePool<ValueKind>> {
        (*self.0).borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, NodePool<ValueKind>> {
        (*self.0).borrow_mut()
    }
}

impl Default for ValuePool {
    fn default() -> Self {
        Self::new()
    }
}

/// Clones a `ValuePool` by cloning the `Rc` of the underlying `NodePool` (thus
/// increasing the reference count).
impl Clone for ValuePool {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

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

    pub fn as_string(&self) -> String {
        match self.pool.borrow().get(self.index) {
            ValueKind::String(s) => s.clone(),
            _ => panic!("Not a string"),
        }
    }

    pub fn as_f64(&self) -> f64 {
        match *self.pool.borrow().get(self.index) {
            ValueKind::Number(n) => n.into(),
            _ => panic!("Not a number"),
        }
    }

    pub fn get(&self, key: &str) -> Value {
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

    pub fn members(&self) -> MembersIter {
        match self.pool.borrow_mut().get_mut(self.index) {
            ValueKind::Array(ref mut array, _) => MembersIter {
                pool: self.pool.clone(),
                array_index: self.index,
                length: array.len(),
                iter_index: 0,
            },
            _ => panic!("Not an array"),
        }
    }

    // TODO: This is not good, see the collect
    pub fn entries(&self) -> EntriesIter {
        match self.pool.borrow().get(self.index) {
            ValueKind::Object(ref map) => {
                let inner: Vec<(String, usize)> =
                    map.iter().map(|(k, v)| (k.clone(), *v)).collect();
                EntriesIter {
                    pool: self.pool.clone(),
                    length: inner.len(),
                    iter_index: 0,
                    inner,
                }
            }
            _ => panic!("Not an object"),
        }
    }
}

/// Delegates comparison to the ValueKind instance in the pool, so you can
/// directly compare Value's to determine if their underlying ValueKinds are equal.
impl PartialEq<Value> for Value {
    fn eq(&self, other: &Value) -> bool {
        self.pool.borrow().get(self.index) == self.pool.borrow().get(other.index)
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

impl PartialEq<bool> for Value {
    fn eq(&self, other: &bool) -> bool {
        match self.pool.borrow().get(self.index) {
            ValueKind::Bool(b) => *b == *other,
            _ => false,
        }
    }
}

pub struct MembersIter {
    pool: ValuePool,
    array_index: usize,
    length: usize,
    iter_index: usize,
}

impl Iterator for MembersIter {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        if self.length == 0 || self.iter_index > self.length - 1 {
            None
        } else if let ValueKind::Array(array, _) = self.pool.borrow().get(self.array_index) {
            self.iter_index += 1;
            Some(Value {
                pool: self.pool.clone(),
                index: array[self.iter_index - 1],
            })
        } else {
            None
        }
    }
}

pub struct EntriesIter {
    pool: ValuePool,
    length: usize,
    iter_index: usize,
    inner: Vec<(String, usize)>,
}

// TODO: This is terrible because it clones the strings, but the whole iterator is bullshit and needs to be rewritten
impl Iterator for EntriesIter {
    type Item = (String, Value);

    fn next(&mut self) -> Option<Self::Item> {
        if self.length == 0 || self.iter_index > self.length - 1 {
            None
        } else {
            self.iter_index += 1;
            Some((
                self.inner[self.iter_index - 1].0.clone(),
                Value {
                    pool: self.pool.clone(),
                    index: self.inner[self.iter_index - 1].1,
                },
            ))
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
        a.push(ValueKind::Number(1.into()));
        a.push(ValueKind::Number(2.into()));
        a.push(ValueKind::Number(3.into()));
        a.push(ValueKind::Number(4.into()));
        a.push(ValueKind::Number(5.into()));

        for v in a.members() {
            println!("{:#?}", v);
        }
    }

    #[test]
    fn entries_iter() {}

    #[test]
    fn wrap_in_array() {
        let pool = ValuePool::new();
        let v1 = Value::new_string(pool.clone(), "hello world");
        let v1 = v1.wrap_in_array();
        println!("{:#?}", v1);
        v1.drop();
        println!("{:#?}", pool);
    }
}
