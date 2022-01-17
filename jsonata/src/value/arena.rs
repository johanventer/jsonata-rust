use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

use super::{ArrayFlags, Value, ValueKind};
use crate::ast::Ast;
use crate::frame::Frame;
use crate::functions::FunctionContext;
use crate::json::Number;
use crate::Result;

/// A reference counted array of `ValueKind` which models the tree structure of data
/// by referencing children by index.
///
/// During evaluation, `ValueKind`s are created regularly, and referenced by the `Value`
/// type which wraps the index and a reference to the arena.
///
/// The tree structure of both JSON input and evaluation results is represented
/// in the arena as a flat list of `ValueKind` where children are referenced by index.
///
/// # Safety
///
/// Items in the arena can never be removed, so deferencing pointers to them is always safe.
pub struct ValueArena(Rc<RefCell<Vec<ValueKind>>>);

impl ValueArena {
    // Construct a new ValueArena.
    pub fn new() -> ValueArena {
        let arena = ValueArena(Rc::new(RefCell::new(Vec::with_capacity(16))));

        // The first index in any ValueArena is undefined, it's very commonly used
        arena.insert(ValueKind::Undefined);

        arena
    }

    /// Insert a new ValueKind into the arena, and return the index of the inserted value.
    pub fn insert(&self, kind: ValueKind) -> usize {
        let mut arena = self.0.borrow_mut();
        let index = arena.len();
        arena.push(kind);
        index
    }

    /// Gets a reference to a ValueKind by index.
    ///
    /// This really only exists for the implementation of Deref on Value to make working with
    /// them convenient. The other uses in Value could be rewritten not to use this, but I don't
    /// see a way to make Deref work without it, which is concerning because it uses a raw pointer
    /// to get the reference, even though the safety argument seems valid.
    #[inline]
    pub fn get(&self, index: usize) -> &ValueKind {
        let arena = self.0.borrow();

        debug_assert!(index < arena.len());

        let item_ptr = &arena[index] as *const ValueKind;

        // SAFETY: Items in the arena are never removed, so pointers to them will always be valid.
        //         The only kinds that can be mutated in any way are arrays and objects, and even
        //         then only to push new values or insert new keys, and the pointer here is to the
        //         ValueKind (which contains a Vec or HashMap) and not to the data contained within,
        //         so the pointer will remain stable and valid for the lifetime of the reference even
        //         if the underlying Vec or HashMap reallocates.
        unsafe { &*item_ptr }
    }

    /// Inserts a new `ValueKind` into the arena, and sets its index to the key in the object.
    //
    // # Panics
    //
    // If the `ValueKind` referenced by `index` is not a `ValueKind::Object`.
    pub fn object_insert(&mut self, index: usize, key: &str, kind: ValueKind) {
        let index_to_insert = self.insert(kind);
        match (self.0.borrow_mut())[index] {
            ValueKind::Object(ref mut object) => {
                object.insert(key.to_owned(), index_to_insert);
            }
            _ => panic!("Not an object"),
        }
    }

    // Inserts a new key into a `ValueKind::Object` directly by index to the value.
    //
    // # Panics
    //
    // If the `ValueKind` referenced by `index` is not a `ValueKind::Object`.
    pub fn object_insert_index(&mut self, index: usize, key: &str, index_to_insert: usize) {
        match (self.0.borrow_mut())[index] {
            ValueKind::Object(ref mut object) => {
                object.insert(key.to_owned(), index_to_insert);
            }
            _ => panic!("Not an object"),
        }
    }

    /// Inserts a new `ValueKind` into the arena, and pushes its index to the key in the object.
    //
    // # Panics
    //
    // If the `ValueKind` referenced by `index` is not a `ValueKind::Array`.
    pub fn array_push(&mut self, index: usize, kind: ValueKind) {
        let index_to_push = self.insert(kind);
        match (self.0.borrow_mut())[index] {
            ValueKind::Array(ref mut array, _) => array.push(index_to_push),
            _ => panic!("Not an array"),
        }
    }

    // Pushes an index into a `ValueKind::Array`.
    //
    // # Panics
    //
    // If the `ValueKind` referenced by `index` is not a `ValueKind::Array`.
    pub fn array_push_index(&mut self, index: usize, index_to_push: usize) {
        match (self.0.borrow_mut())[index] {
            ValueKind::Array(ref mut array, _) => array.push(index_to_push),
            _ => panic!("Not an array"),
        }
    }

    // Set the flags on a `ValueKind::Array`.
    //
    // # Panics
    //
    // If the `ValueKind` referenced by `index` is not a `ValueKind::Array`.
    pub fn array_set_flags(&mut self, index: usize, new_flags: ArrayFlags) {
        match (self.0.borrow_mut())[index] {
            ValueKind::Array(_, ref mut flags) => *flags = new_flags,
            _ => panic!("Not an array"),
        }
    }

    // Adds flags to a `ValueKind::Array`.
    //
    // # Panics
    //
    // If the `ValueKind` referenced by `index` is not a `ValueKind::Array`.
    pub fn array_add_flags(&mut self, index: usize, flags_to_add: ArrayFlags) {
        match (self.0.borrow_mut())[index] {
            ValueKind::Array(_, ref mut flags) => flags.insert(flags_to_add),
            _ => panic!("Not an array"),
        }
    }

    // Return a `Value` which points at the default `ValueKind::Undefined` in the arena.
    pub fn undefined(&self) -> Value {
        Value {
            arena: self.clone(),
            index: 0,
        }
    }

    // Inserts a new `ValueKind` into the arena, returning a `Value` which references it.
    pub fn value(&self, kind: ValueKind) -> Value {
        Value {
            arena: self.clone(),
            index: self.insert(kind),
        }
    }

    // Inserts a new `ValueKind:Null` into the arena, returning a `Value` which references it.
    pub fn null(&self) -> Value {
        Value {
            arena: self.clone(),
            index: self.insert(ValueKind::Null),
        }
    }

    // Inserts a new `ValueKind:Bool` into the arena, returning a `Value` which references it.
    pub fn bool(&self, value: bool) -> Value {
        Value {
            arena: self.clone(),
            index: self.insert(ValueKind::Bool(value)),
        }
    }

    // Inserts a new `ValueKind:Number` into the arena, returning a `Value` which references it.
    pub fn number<T: Into<Number>>(&self, value: T) -> Value {
        Value {
            arena: self.clone(),
            index: self.insert(ValueKind::Number(value.into())),
        }
    }

    // Inserts a new `ValueKind:String` into the arena, returning a `Value` which references it.
    pub fn string<T: Into<String>>(&self, value: T) -> Value {
        Value {
            arena: self.clone(),
            index: self.insert(ValueKind::String(value.into())),
        }
    }

    // Inserts a new `ValueKind:Array` into the arena, returning a `Value` which references it.
    pub fn array(&self, flags: ArrayFlags) -> Value {
        Value {
            arena: self.clone(),
            index: self.insert(ValueKind::Array(Vec::new(), flags)),
        }
    }

    // Inserts a new `ValueKind:Array` into the arena with the specified capacity,
    // returning a `Value` which references it.
    pub fn array_with_capacity(&self, capacity: usize, flags: ArrayFlags) -> Value {
        Value {
            arena: self.clone(),
            index: self.insert(ValueKind::Array(Vec::with_capacity(capacity), flags)),
        }
    }

    // Inserts a new `ValueKind:Object` into the arena, returning a `Value` which references it.
    pub fn object(&self) -> Value {
        Value {
            arena: self.clone(),
            index: self.insert(ValueKind::Object(HashMap::new())),
        }
    }

    // Inserts a new `ValueKind:Object` into the arena with the specified capacity,
    // returning a `Value` which references it.
    pub fn object_with_capacity(&self, capacity: usize) -> Value {
        Value {
            arena: self.clone(),
            index: self.insert(ValueKind::Object(HashMap::with_capacity(capacity))),
        }
    }

    // Inserts a new `ValueKind:Lambda` into the arena with the specified capacity,
    pub fn lambda(&self, name: &str, node: Ast, input: Value, frame: Frame) -> Value {
        Value {
            arena: self.clone(),
            index: self.insert(ValueKind::Lambda {
                name: name.to_string(),
                ast: node,
                input,
                frame,
            }),
        }
    }

    // Inserts a new `ValueKind:NativeFn0` into the arena with the specified capacity,
    pub fn nativefn0(&self, name: &str, func: fn(&FunctionContext) -> Result<Value>) -> Value {
        Value {
            arena: self.clone(),
            index: self.insert(ValueKind::NativeFn0(name.to_string(), func)),
        }
    }

    // Inserts a new `ValueKind:NativeFn1` into the arena with the specified capacity,
    pub fn nativefn1(
        &self,
        name: &str,
        func: fn(&FunctionContext, &Value) -> Result<Value>,
    ) -> Value {
        Value {
            arena: self.clone(),
            index: self.insert(ValueKind::NativeFn1(name.to_string(), func)),
        }
    }

    // Inserts a new `ValueKind:NativeFn2` into the arena with the specified capacity,
    pub fn nativefn2(
        &self,
        name: &str,
        func: fn(&FunctionContext, &Value, &Value) -> Result<Value>,
    ) -> Value {
        Value {
            arena: self.clone(),
            index: self.insert(ValueKind::NativeFn2(name.to_string(), func)),
        }
    }

    // Inserts a new `ValueKind:NativeFn3` into the arena with the specified capacity,
    pub fn nativefn3(
        &self,
        name: &str,
        func: fn(&FunctionContext, &Value, &Value, &Value) -> Result<Value>,
    ) -> Value {
        Value {
            arena: self.clone(),
            index: self.insert(ValueKind::NativeFn3(name.to_string(), func)),
        }
    }
}

impl Default for ValueArena {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns a new `ValuArena` with the reference count of the contained Rc bumped.
impl Clone for ValueArena {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Debug for ValueArena {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, kind) in self.0.borrow().iter().enumerate() {
            write!(f, "[{}] ", i)?;
            match kind {
                ValueKind::Undefined => write!(f, "undefined")?,
                ValueKind::Null => write!(f, "null")?,
                ValueKind::Number(value) => write!(f, "{}", value)?,
                ValueKind::Bool(value) => write!(f, "{}", value)?,
                ValueKind::String(value) => write!(f, "{}", value)?,
                ValueKind::Array(array, _) => f.debug_list().entries(array.iter()).finish()?,
                ValueKind::Object(object) => f.debug_map().entries(object.iter()).finish()?,
                ValueKind::Lambda { .. } => write!(f, "<lambda>")?,
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
