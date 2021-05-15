use json::JsonValue;
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::rc::Rc;

// use super::FramePtr;
// use crate::parser::ast::Node;

#[derive(Debug)]
pub enum Value {
    Undef,
    Raw(JsonValue),
    Array {
        arr: RefCell<Vec<Rc<Value>>>,
        is_seq: Cell<bool>,
        keep_array: Cell<bool>,
        keep_singleton: Cell<bool>,
        cons_array: Cell<bool>,
        outer_wrapper: Cell<bool>,
    },
    // Closure {
    //     input: Rc<Value>,
    //     frame: FramePtr,
    //     args: Rc<Vec<Box<Node>>>,
    //     body: Rc<Box<Node>>,
    // }
}

macro_rules! array_flag {
    ($i:ident, $g:ident) => {
        #[inline]
        pub fn $g(&self) -> bool {
            match self {
                Value::Array { $i, .. } => $i.get(),
                _ => false,
            }
        }
    };

    ($i:ident, $g:ident, $s:ident) => {
        array_flag!($i, $g);

        #[inline]
        pub fn $s(&self) {
            match self {
                Value::Array { $i, .. } => $i.set(true),
                _ => panic!("unexpected Value type"),
            }
        }
    };
}

impl Value {
    pub fn from_raw(raw: Option<&JsonValue>) -> Self {
        match raw {
            None => Self::Undef,
            Some(raw) => match raw {
                JsonValue::Array(arr) => Self::Array {
                    arr: RefCell::new(
                        arr.iter()
                            .map(|v| Rc::new(Self::from_raw(Some(v))))
                            .collect(),
                    ),
                    is_seq: Cell::new(false),
                    keep_array: Cell::new(false),
                    keep_singleton: Cell::new(false),
                    cons_array: Cell::new(false),
                    outer_wrapper: Cell::new(false),
                },
                _ => Self::Raw(raw.clone()),
            },
        }
    }

    pub fn wrap(value: Rc<Value>) -> Self {
        Self::Array {
            arr: RefCell::new(vec![value]),
            is_seq: Cell::new(true),
            keep_array: Cell::new(false),
            keep_singleton: Cell::new(false),
            cons_array: Cell::new(false),
            outer_wrapper: Cell::new(true),
        }
    }

    pub fn new_arr() -> Self {
        Self::Array {
            arr: RefCell::new(Vec::new()),
            is_seq: Cell::new(false),
            keep_array: Cell::new(false),
            keep_singleton: Cell::new(false),
            cons_array: Cell::new(false),
            outer_wrapper: Cell::new(false),
        }
    }

    pub fn new_seq() -> Self {
        Self::Array {
            arr: RefCell::new(Vec::new()),
            is_seq: Cell::new(true),
            keep_array: Cell::new(false),
            keep_singleton: Cell::new(false),
            cons_array: Cell::new(false),
            outer_wrapper: Cell::new(false),
        }
    }

    pub fn seq_from(item: Rc<Value>) -> Self {
        Self::Array {
            arr: RefCell::new(vec![item]),
            is_seq: Cell::new(true),
            keep_array: Cell::new(false),
            keep_singleton: Cell::new(false),
            cons_array: Cell::new(false),
            outer_wrapper: Cell::new(false),
        }
    }

    pub fn seq_with_capacity(size: usize) -> Self {
        Self::Array {
            arr: RefCell::new(Vec::with_capacity(size)),
            is_seq: Cell::new(true),
            keep_array: Cell::new(false),
            keep_singleton: Cell::new(false),
            cons_array: Cell::new(false),
            outer_wrapper: Cell::new(false),
        }
    }

    #[inline]
    pub fn is_undef(&self) -> bool {
        matches!(self, Value::Undef)
    }

    #[inline]
    pub fn is_raw(&self) -> bool {
        matches!(self, Value::Raw(..))
    }

    #[inline]
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array {.. })
    }

    #[inline]
    pub fn arr(&self) -> Ref<'_, Vec<Rc<Value>>> {
        match self {
            Value::Array { arr, .. } => arr.borrow(),
            _ => panic!("unexpected Value type"),
        }
    }

    #[inline]
    pub fn arr_mut(&self) -> RefMut<'_, Vec<Rc<Value>>> {
        match self {
            Value::Array { arr, .. } => arr.borrow_mut(),
            _ => panic!("unexpected Value type"),
        }
    }

    #[inline]
    pub fn as_raw(&self) -> &JsonValue {
        match self {
            Value::Raw(ref raw) => raw,
            _ => panic!("unexpected Value type"),
        }
    }

    array_flag!(outer_wrapper, is_wrapped);
    array_flag!(is_seq, is_seq);
    array_flag!(keep_array, keep_array, set_keep_array);
    array_flag!(keep_singleton, keep_singleton, set_keep_singleton);
    array_flag!(cons_array, cons_array, set_cons_array);

    pub fn as_json(&self) -> Option<JsonValue> {
        match self {
            Value::Undef => None,
            Value::Raw(raw) => Some(raw.clone()),
            Value::Array { .. } => Some(JsonValue::Array(
                self.arr()
                    .iter()
                    .filter(|v| !v.is_undef())
                    .map(|v| v.as_json().unwrap())
                    .collect(),
            )),
        }
    }

    /// Returns the raw JSON value as a isize if it can be converted, and checks to ensure that it
    /// is an integer (i.e. it returns None if there is any fractional part).
    pub fn as_isize(&self) -> Option<isize> {
        match self {
            Value::Raw(raw) => match raw.as_f64() {
                Some(num) => {
                    if num.trunc() == num {
                        Some(num as isize)
                    } else {
                        None
                    }
                }
                None => None,
            },
            _ => None,
        }
    }

    /// Convenience method for determining if a value is an array of numbers
    pub fn as_f64_vec(&self) -> Option<Vec<f64>> {
        match self {
            Value::Array { arr, .. } => {
                let mut nums = vec![];
                for value in arr.borrow().iter() {
                    if let Some(num) = value.as_raw().as_f64() {
                        nums.push(num);
                    } else {
                        return None;
                    }
                }
                return Some(nums);
            }
            _ => None,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        if self.is_undef() && other.is_undef() {
            true
        } else if self.is_raw() && other.is_raw() {
            self.as_raw() == other.as_raw()
        } else if self.is_array() && other.is_array() {
            if self.arr().len() != other.arr().len() {
                false
            } else {
                for i in 0..self.arr().len() - 1 {
                    if self.arr()[i] != other.arr()[i] {
                        return false;
                    }
                }
                true
            }
        } else {
            false
        }
    }
}
