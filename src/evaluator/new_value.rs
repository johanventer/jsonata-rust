mod kind;
use hashbrown::{BumpWrapper, HashMap};
pub use kind::{ArrayFlags, ValueKind};

pub mod impls;

pub mod iterator;

pub mod range;
use range::Range;

use bumpalo::{boxed::Box, collections::string::String, collections::vec::Vec, Bump};
use std::{
    cell::{Ref, RefCell, RefMut},
    marker::PhantomData,
};

pub type ValuePtr<'arena> = *const RefCell<ValueKind<'arena>>;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Value<'arena> {
    ptr: ValuePtr<'arena>,
    marker: PhantomData<RefCell<ValueKind<'arena>>>,
}

impl<'arena> Value<'arena> {
    fn new(arena: &Bump, kind: ValueKind<'arena>) -> Self {
        Self {
            ptr: Box::into_raw(Box::new_in(RefCell::new(kind), arena)) as *const _,
            marker: PhantomData,
        }
    }

    pub fn borrow(&self) -> Ref<'arena, ValueKind<'arena>> {
        unsafe { (*self.ptr).borrow() }
    }

    pub fn borrow_mut(&self) -> RefMut<'arena, ValueKind<'arena>> {
        unsafe { (*self.ptr).borrow_mut() }
    }
}

#[derive(Debug)]
pub struct ValueArena {
    arena: Bump,
}

impl ValueArena {
    pub fn new() -> Self {
        let arena = Bump::new();
        Self { arena }
    }

    pub fn undefined(&self) -> Value {
        Value::new(&self.arena, ValueKind::Undefined)
    }

    pub fn bool(&self, value: impl Into<bool>) -> Value {
        Value::new(&self.arena, ValueKind::Bool(value.into()))
    }

    pub fn number(&self, value: impl Into<f64>) -> Value {
        Value::new(&self.arena, ValueKind::Number(value.into()))
    }

    pub fn string(&self, value: &str) -> Value {
        let value = String::from_str_in(value, &self.arena);
        Value::new(&self.arena, ValueKind::String(value))
    }

    pub fn array(&self) -> Value {
        let value = Vec::new_in(&self.arena);
        Value::new(&self.arena, ValueKind::Array(value, ArrayFlags::empty()))
    }

    pub fn array_with_flags(&self, flags: ArrayFlags) -> Value {
        let value = Vec::new_in(&self.arena);
        Value::new(&self.arena, ValueKind::Array(value, flags))
    }

    pub fn array_with_capacity(&self, capacity: usize) -> Value {
        let value = Vec::with_capacity_in(capacity, &self.arena);
        Value::new(&self.arena, ValueKind::Array(value, ArrayFlags::empty()))
    }

    pub fn array_with_capacity_and_flags(&self, capacity: usize, flags: ArrayFlags) -> Value {
        let value = Vec::with_capacity_in(capacity, &self.arena);
        Value::new(&self.arena, ValueKind::Array(value, flags))
    }

    pub fn range(&self, start: isize, end: isize) -> Value {
        Value::new(&self.arena, ValueKind::Range(Range::new(self, start, end)))
    }

    pub fn object(&self) -> Value {
        let hash = HashMap::new_in(BumpWrapper(&self.arena));
        Value::new(&self.arena, ValueKind::Object(hash))
    }

    pub fn object_with_capacity(&self, capacity: usize) -> Value {
        let hash = HashMap::with_capacity_in(capacity, BumpWrapper(&self.arena));
        Value::new(&self.arena, ValueKind::Object(hash))
    }
}
