use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

use super::{ArrayFlags, Value, ValueKind};
use crate::ast::Node;
use crate::functions::FunctionContext;
use crate::json::Number;
use crate::Result;

/// A reference counted array of `ValueKind` which models the tree structure of data
/// by referencing children by index.
///
/// During evaluation, `ValueKind`s are created regularly, and referenced by the `Value`
/// type which stores the index and a reference to the pool.
///
/// The tree structure of both JSON input and evaluation results is represented
/// in the pool as a flat list of `ValueKind` where children are referenced by index.
///
/// # Safety
///
/// Items in the pool can never be removed, so deferencing pointers to them is always safe.
pub struct ValuePool(Rc<RefCell<Vec<ValueKind>>>);

impl ValuePool {
    pub fn new() -> ValuePool {
        let pool = ValuePool(Rc::new(RefCell::new(Vec::with_capacity(16))));

        // The first index in any ValuePool is undefined, it's very commonly used
        pool.insert(ValueKind::Undefined);

        pool
    }

    /// Insert a new ValueKind into the pool, and return the index of the inserted value.
    pub fn insert(&self, kind: ValueKind) -> usize {
        let mut pool = self.0.borrow_mut();
        let index = pool.len();
        pool.push(kind);
        index
    }

    #[inline]
    pub fn get(&self, index: usize) -> &ValueKind {
        let pool = self.0.borrow();

        debug_assert!(index < pool.len());

        let item_ptr = &pool[index] as *const ValueKind;

        // SAFETY: Items in the pool are never removed, so pointers to them will always be valid.
        unsafe { &*item_ptr }
    }

    pub fn object_insert(&mut self, index: usize, key: &str, kind: ValueKind) {
        let index_to_insert = self.insert(kind);
        match (self.0.borrow_mut())[index] {
            ValueKind::Object(ref mut object) => {
                object.insert(key.to_owned(), index_to_insert);
            }
            _ => panic!("Not an object"),
        }
    }

    pub fn object_insert_index(&mut self, index: usize, key: &str, index_to_insert: usize) {
        match (self.0.borrow_mut())[index] {
            ValueKind::Object(ref mut object) => {
                object.insert(key.to_owned(), index_to_insert);
            }
            _ => panic!("Not an object"),
        }
    }

    pub fn array_push(&mut self, index: usize, kind: ValueKind) {
        let index_to_push = self.insert(kind);
        match (self.0.borrow_mut())[index] {
            ValueKind::Array(ref mut array, _) => array.push(index_to_push),
            _ => panic!("Not an array"),
        }
    }

    pub fn array_push_index(&mut self, index: usize, index_to_push: usize) {
        match (self.0.borrow_mut())[index] {
            ValueKind::Array(ref mut array, _) => array.push(index_to_push),
            _ => panic!("Not an array"),
        }
    }

    pub fn array_set_flags(&mut self, index: usize, new_flags: ArrayFlags) {
        match (self.0.borrow_mut())[index] {
            ValueKind::Array(_, ref mut flags) => *flags = new_flags,
            _ => panic!("Not an array"),
        }
    }

    pub fn array_add_flags(&mut self, index: usize, flags_to_add: ArrayFlags) {
        match (self.0.borrow_mut())[index] {
            ValueKind::Array(_, ref mut flags) => flags.insert(flags_to_add),
            _ => panic!("Not an array"),
        }
    }

    pub fn undefined(&self) -> Value {
        Value {
            pool: self.clone(),
            index: 0,
        }
    }

    pub fn value(&self, kind: ValueKind) -> Value {
        Value {
            pool: self.clone(),
            index: self.insert(kind),
        }
    }

    pub fn null(&self) -> Value {
        Value {
            pool: self.clone(),
            index: self.insert(ValueKind::Null),
        }
    }

    pub fn bool(&self, value: bool) -> Value {
        Value {
            pool: self.clone(),
            index: self.insert(ValueKind::Bool(value)),
        }
    }

    pub fn number<T: Into<Number>>(&self, value: T) -> Value {
        Value {
            pool: self.clone(),
            index: self.insert(ValueKind::Number(value.into())),
        }
    }

    pub fn string(&self, value: &str) -> Value {
        Value {
            pool: self.clone(),
            index: self.insert(ValueKind::String(value.to_owned())),
        }
    }

    pub fn array(&self, flags: ArrayFlags) -> Value {
        Value {
            pool: self.clone(),
            index: self.insert(ValueKind::Array(Vec::new(), flags)),
        }
    }

    pub fn array_with_capacity(&self, capacity: usize, flags: ArrayFlags) -> Value {
        Value {
            pool: self.clone(),
            index: self.insert(ValueKind::Array(Vec::with_capacity(capacity), flags)),
        }
    }

    pub fn object(&self) -> Value {
        Value {
            pool: self.clone(),
            index: self.insert(ValueKind::Object(HashMap::new())),
        }
    }

    pub fn object_with_capacity(&self, capacity: usize) -> Value {
        Value {
            pool: self.clone(),
            index: self.insert(ValueKind::Object(HashMap::with_capacity(capacity))),
        }
    }

    pub fn lambda(&self, name: &str, node: Node) -> Value {
        Value {
            pool: self.clone(),
            index: self.insert(ValueKind::Lambda(name.to_string(), node)),
        }
    }

    pub fn nativefn0(&self, name: &str, func: fn(&FunctionContext) -> Result<Value>) -> Value {
        Value {
            pool: self.clone(),
            index: self.insert(ValueKind::NativeFn0(name.to_string(), func)),
        }
    }

    pub fn nativefn1(
        &self,
        name: &str,
        func: fn(&FunctionContext, &Value) -> Result<Value>,
    ) -> Value {
        Value {
            pool: self.clone(),
            index: self.insert(ValueKind::NativeFn1(name.to_string(), func)),
        }
    }

    pub fn nativefn2(
        &self,
        name: &str,
        func: fn(&FunctionContext, &Value, &Value) -> Result<Value>,
    ) -> Value {
        Value {
            pool: self.clone(),
            index: self.insert(ValueKind::NativeFn2(name.to_string(), func)),
        }
    }
}

impl Default for ValuePool {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns a new `ValuPool` with the reference count of the contained Rc bumped.
impl Clone for ValuePool {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Debug for ValuePool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, _) in self.0.borrow().iter().enumerate() {
            write!(f, "[{}] ", i)?;
            match self.get(i) {
                ValueKind::Undefined => write!(f, "undefined")?,
                ValueKind::Null => write!(f, "null")?,
                ValueKind::Number(value) => write!(f, "{}", value)?,
                ValueKind::Bool(value) => write!(f, "{}", value)?,
                ValueKind::String(value) => write!(f, "{}", value)?,
                ValueKind::Array(array, _) => f.debug_list().entries(array.iter()).finish()?,
                ValueKind::Object(object) => f.debug_map().entries(object.iter()).finish()?,
                ValueKind::Lambda(..) => write!(f, "<lambda>")?,
                ValueKind::NativeFn0(..)
                | ValueKind::NativeFn1(..)
                | ValueKind::NativeFn2(..)
                | ValueKind::NativeFn3(..) => write!(f, "<nativefn>")?,
            };
            writeln!(f)?;
        }
        Ok(())
    }
}
