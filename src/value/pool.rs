use std::{
    cell::{Ref, RefCell, RefMut},
    fmt::Debug,
    rc::Rc,
};

use super::{Value, ValueKind};
use crate::node_pool::NodePool;

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
            };
            writeln!(f)?;
        }
        Ok(())
    }
}
