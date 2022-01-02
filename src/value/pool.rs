use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    fmt::Debug,
    rc::Rc,
};

use super::{ArrayFlags, Value, ValueKind};
use crate::ast::Node;
use crate::functions::FunctionContext;
use crate::json::Number;
use crate::node_pool::NodePool;
use crate::Result;

/// A reference counted `NodePool` of `ValueKind`.
///
/// The tree structure of both JSON input and evaluation results is represented
/// in the pool as a flat list of `ValueKind` where children are referenced by index.
pub struct ValuePool(Rc<RefCell<NodePool<ValueKind>>>);

impl ValuePool {
    pub fn new() -> ValuePool {
        let pool = ValuePool(Rc::new(RefCell::new(NodePool::new())));

        // The first index in any ValuePool is undefined
        pool.borrow_mut().insert(ValueKind::Undefined);

        pool
    }

    #[inline]
    pub fn undefined(&self) -> Value {
        Value {
            pool: self.clone(),
            index: 0,
        }
    }

    #[inline]
    pub fn borrow(&self) -> Ref<'_, NodePool<ValueKind>> {
        (*self.0).borrow()
    }

    #[inline]
    pub fn borrow_mut(&self) -> RefMut<'_, NodePool<ValueKind>> {
        (*self.0).borrow_mut()
    }

    #[inline]
    pub fn value(&self, kind: ValueKind) -> Value {
        Value {
            pool: self.clone(),
            index: self.borrow_mut().insert(kind),
        }
    }

    #[inline]
    pub fn null(&self) -> Value {
        Value {
            pool: self.clone(),
            index: self.borrow_mut().insert(ValueKind::Null),
        }
    }

    #[inline]
    pub fn bool(&self, value: bool) -> Value {
        Value {
            pool: self.clone(),
            index: self.borrow_mut().insert(ValueKind::Bool(value)),
        }
    }

    #[inline]
    pub fn number<T: Into<Number>>(&self, value: T) -> Value {
        Value {
            pool: self.clone(),
            index: self.borrow_mut().insert(ValueKind::Number(value.into())),
        }
    }

    #[inline]
    pub fn string(&self, value: &str) -> Value {
        Value {
            pool: self.clone(),
            index: self
                .borrow_mut()
                .insert(ValueKind::String(value.to_owned())),
        }
    }

    #[inline]
    pub fn array(&self, flags: ArrayFlags) -> Value {
        Value {
            pool: self.clone(),
            index: self
                .borrow_mut()
                .insert(ValueKind::Array(Vec::new(), flags)),
        }
    }

    #[inline]
    pub fn array_with_flags(&self, flags: ArrayFlags) -> Value {
        Value {
            pool: self.clone(),
            index: self
                .borrow_mut()
                .insert(ValueKind::Array(Vec::new(), flags)),
        }
    }

    #[inline]
    pub fn array_with_capacity(&self, capacity: usize, flags: ArrayFlags) -> Value {
        Value {
            pool: self.clone(),
            index: self
                .borrow_mut()
                .insert(ValueKind::Array(Vec::with_capacity(capacity), flags)),
        }
    }

    #[inline]
    pub fn object(&self) -> Value {
        Value {
            pool: self.clone(),
            index: self.borrow_mut().insert(ValueKind::Object(HashMap::new())),
        }
    }

    #[inline]
    pub fn object_with_capacity(&self, capacity: usize) -> Value {
        Value {
            pool: self.clone(),
            index: self
                .borrow_mut()
                .insert(ValueKind::Object(HashMap::with_capacity(capacity))),
        }
    }

    #[inline]
    pub fn lambda(&self, name: &str, node: Node) -> Value {
        Value {
            pool: self.clone(),
            index: self
                .borrow_mut()
                .insert(ValueKind::Lambda(name.to_string(), node)),
        }
    }

    #[inline]
    pub fn nativefn0(&self, name: &str, func: fn(FunctionContext) -> Result<Value>) -> Value {
        Value {
            pool: self.clone(),
            index: self
                .borrow_mut()
                .insert(ValueKind::NativeFn0(name.to_string(), func)),
        }
    }

    #[inline]
    pub fn nativefn1(
        &self,
        name: &str,
        func: fn(FunctionContext, Value) -> Result<Value>,
    ) -> Value {
        Value {
            pool: self.clone(),
            index: self
                .borrow_mut()
                .insert(ValueKind::NativeFn1(name.to_string(), func)),
        }
    }

    #[inline]
    pub fn nativefn2(
        &self,
        name: &str,
        func: fn(FunctionContext, Value, Value) -> Result<Value>,
    ) -> Value {
        Value {
            pool: self.clone(),
            index: self
                .borrow_mut()
                .insert(ValueKind::NativeFn2(name.to_string(), func)),
        }
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

impl Debug for ValuePool {
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
