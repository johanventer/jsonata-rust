use crate::json::object::Iter;
use crate::json::{Number, Object};
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub enum Value {
    Undefined,
    Null,
    Number(Number),
    Bool(bool),
    String(String),
    Array {
        items: Vec<Value>,
        is_sequence: bool,
        cons: bool,
        keep_singleton: bool,
    },
    Object(Object),
}

pub const UNDEFINED: Value = Value::Undefined;

impl Value {
    pub fn new_object() -> Value {
        Value::Object(Object::new())
    }

    pub fn insert(&mut self, key: &str, value: Value) {
        match *self {
            Value::Object(ref mut map) => {
                map.insert(key, value);
            }
            _ => panic!("Tried to insert into a Value that wasn't an Object"),
        }
    }

    pub fn contains(&self, key: &str) -> bool {
        match *self {
            Value::Object(ref map) => map.get(key).is_some(),
            _ => panic!("Tried to call contains on a Value that wasn't an Object"),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        match *self {
            Value::Object(ref map) => map.get(key),
            _ => panic!("Tried to call get on a Value that wasn't an Object"),
        }
    }

    pub fn new_array() -> Value {
        Value::Array {
            items: Vec::new(),
            is_sequence: false,
            cons: false,
            keep_singleton: false,
        }
    }

    pub fn new_array_with_capacity(capacity: usize) -> Value {
        Value::Array {
            items: Vec::with_capacity(capacity),
            is_sequence: false,
            cons: false,
            keep_singleton: false,
        }
    }

    pub fn with_items(items: Vec<Value>) -> Value {
        Value::Array {
            items,
            is_sequence: false,
            cons: false,
            keep_singleton: false,
        }
    }

    pub fn push(&mut self, item: Value) {
        match *self {
            Value::Array { ref mut items, .. } => {
                items.push(item);
            }
            _ => panic!("Tried to push into a Value that wasn't an Array"),
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Value> {
        match *self {
            Value::Array { ref items, .. } => items.iter(),
            _ => panic!("Tried to call iter on a Value that wasn't an Array"),
        }
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Value> {
        match *self {
            Value::Array { ref mut items, .. } => items.iter_mut(),
            _ => panic!("Tried to call iter_mut on a Value that wasn't an Array"),
        }
    }

    pub fn len(&self) -> usize {
        match *self {
            Value::Array { ref items, .. } => items.len(),
            _ => panic!("Tried to call len on a Value that wasn't an Array"),
        }
    }

    pub fn is_empty(&self) -> bool {
        match *self {
            Value::Array { ref items, .. } => items.is_empty(),
            Value::Object(ref map) => map.is_empty(),
            _ => panic!("Tried to call is_empty on a Value that wasn't an Array or an Object"),
        }
    }

    pub fn entries(&self) -> Iter {
        match *self {
            Value::Object(ref object) => object.iter(),
            _ => panic!("Tried to call entries on a Value that wasn't an Object"),
        }
    }

    pub fn is_undefined(&self) -> bool {
        matches!(*self, Value::Undefined)
    }

    pub fn is_null(&self) -> bool {
        matches!(*self, Value::Null)
    }

    pub fn is_number(&self) -> bool {
        matches!(*self, Value::Number(..))
    }

    pub fn is_bool(&self) -> bool {
        matches!(*self, Value::Bool(..))
    }

    pub fn is_string(&self) -> bool {
        matches!(*self, Value::String(..))
    }

    pub fn is_array(&self) -> bool {
        matches!(*self, Value::Array { .. })
    }

    pub fn is_sequence(&self) -> bool {
        match *self {
            Value::Array { is_sequence, .. } => is_sequence,
            _ => false,
        }
    }

    pub fn cons(&self) -> bool {
        match *self {
            Value::Array { cons, .. } => cons,
            _ => false,
        }
    }

    pub fn is_object(&self) -> bool {
        matches!(*self, Value::Object(_))
    }

    pub fn as_f64(&self) -> f64 {
        match *self {
            Value::Number(n) => n.into(),
            _ => panic!("Tried to call as_f64 on a Value that wasn't a Number"),
        }
    }

    pub fn as_str(&self) -> &str {
        match *self {
            Value::String(ref s) => s,
            _ => panic!("Tried to call as_string on a Value that wasn't a String"),
        }
    }
}

impl Index<usize> for Value {
    type Output = Value;

    fn index(&self, index: usize) -> &Self::Output {
        match *self {
            Value::Array { ref items, .. } => &items[index],
            _ => panic!("Tried to index a a Value that wasn't an Array"),
        }
    }
}

impl IndexMut<usize> for Value {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match *self {
            Value::Array { ref mut items, .. } => &mut items[index],
            _ => panic!("Tried to index a a Value that wasn't an Array"),
        }
    }
}

impl Index<&str> for Value {
    type Output = Value;

    fn index(&self, index: &str) -> &Self::Output {
        match *self {
            Value::Object(ref obj) => obj.get(index).unwrap_or(&UNDEFINED),
            _ => panic!("Tried to index a Value that wasn't an Object"),
        }
    }
}

impl PartialEq<Value> for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Array { items: l_items, .. }, Self::Array { items: r_items, .. }) => {
                l_items == r_items
            }
            (Self::Object(l0), Self::Object(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl PartialEq<bool> for Value {
    fn eq(&self, other: &bool) -> bool {
        match *self {
            Value::Bool(ref b) => b == other,
            _ => panic!("Tried to compare a non-Bool to bool"),
        }
    }
}

impl PartialEq<&str> for Value {
    fn eq(&self, other: &&str) -> bool {
        match *self {
            Value::String(ref s) => s == other,
            _ => panic!("Tried to compare a non-String to &str"),
        }
    }
}

impl<'a> From<&'a str> for Value {
    fn from(val: &'a str) -> Value {
        Value::String(val.into())
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Value::Undefined => f.write_str(""),
            Value::Null => f.write_str("null"),
            Value::Number(n) => f.write_str(&n.to_string()),
            Value::Bool(b) => f.write_str(&b.to_string()),
            Value::String(ref s) => f.write_str(s),
            Value::Array { ref items, .. } => {
                f.write_str("[")?;
                for item in items.iter() {
                    f.write_str(&item.to_string())?;
                    f.write_str(",")?;
                }
                f.write_str("]")
            }
            Value::Object(ref obj) => {
                f.write_str("{")?;
                for (key, value) in obj.iter() {
                    f.write_str("\"")?;
                    f.write_str(key)?;
                    f.write_str("\":")?;
                    f.write_str(&value.to_string())?;
                    f.write_str(",")?;
                }
                f.write_str("{")
            }
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Undefined
    }
}

// use json::JsonValue;
// use std::cell::{Cell, Ref, RefCell, RefMut};
// use std::rc::Rc;

// // use super::FramePtr;
// // use crate::parser::ast::Node;

// #[derive(Debug)]
// pub enum Value {
//     Undef,
//     Raw(JsonValue),
//     Array {
//         arr: RefCell<Vec<Rc<Value>>>,
//         is_seq: Cell<bool>,
//         keep_array: Cell<bool>,
//         keep_singleton: Cell<bool>,
//         cons_array: Cell<bool>,
//         outer_wrapper: Cell<bool>,
//     },
//     // Closure {
//     //     input: Rc<Value>,
//     //     frame: FramePtr,
//     //     args: Rc<Vec<Box<Node>>>,
//     //     body: Rc<Box<Node>>,
//     // }
// }

// macro_rules! array_flag {
//     ($i:ident, $g:ident) => {
//         #[inline]
//         pub fn $g(&self) -> bool {
//             match self {
//                 Value::Array { $i, .. } => $i.get(),
//                 _ => false,
//             }
//         }
//     };

//     ($i:ident, $g:ident, $s:ident) => {
//         array_flag!($i, $g);

//         #[inline]
//         pub fn $s(&self) {
//             match self {
//                 Value::Array { $i, .. } => $i.set(true),
//                 _ => panic!("unexpected Value type"),
//             }
//         }
//     };
// }

// impl Value {
//     pub fn from_raw(raw: Option<&JsonValue>) -> Self {
//         match raw {
//             None => Self::Undef,
//             Some(raw) => match raw {
//                 JsonValue::Array(arr) => Self::Array {
//                     arr: RefCell::new(
//                         arr.iter()
//                             .map(|v| Rc::new(Self::from_raw(Some(v))))
//                             .collect(),
//                     ),
//                     is_seq: Cell::new(false),
//                     keep_array: Cell::new(false),
//                     keep_singleton: Cell::new(false),
//                     cons_array: Cell::new(false),
//                     outer_wrapper: Cell::new(false),
//                 },
//                 _ => Self::Raw(raw.clone()),
//             },
//         }
//     }

//     pub fn wrap(value: Rc<Value>) -> Self {
//         Self::Array {
//             arr: RefCell::new(vec![value]),
//             is_seq: Cell::new(true),
//             keep_array: Cell::new(false),
//             keep_singleton: Cell::new(false),
//             cons_array: Cell::new(false),
//             outer_wrapper: Cell::new(true),
//         }
//     }

//     pub fn new_arr() -> Self {
//         Self::Array {
//             arr: RefCell::new(Vec::new()),
//             is_seq: Cell::new(false),
//             keep_array: Cell::new(false),
//             keep_singleton: Cell::new(false),
//             cons_array: Cell::new(false),
//             outer_wrapper: Cell::new(false),
//         }
//     }

//     pub fn new_seq() -> Self {
//         Self::Array {
//             arr: RefCell::new(Vec::new()),
//             is_seq: Cell::new(true),
//             keep_array: Cell::new(false),
//             keep_singleton: Cell::new(false),
//             cons_array: Cell::new(false),
//             outer_wrapper: Cell::new(false),
//         }
//     }

//     pub fn seq_from(item: Rc<Value>) -> Self {
//         Self::Array {
//             arr: RefCell::new(vec![item]),
//             is_seq: Cell::new(true),
//             keep_array: Cell::new(false),
//             keep_singleton: Cell::new(false),
//             cons_array: Cell::new(false),
//             outer_wrapper: Cell::new(false),
//         }
//     }

//     pub fn seq_with_capacity(size: usize) -> Self {
//         Self::Array {
//             arr: RefCell::new(Vec::with_capacity(size)),
//             is_seq: Cell::new(true),
//             keep_array: Cell::new(false),
//             keep_singleton: Cell::new(false),
//             cons_array: Cell::new(false),
//             outer_wrapper: Cell::new(false),
//         }
//     }

//     #[inline]
//     pub fn is_undef(&self) -> bool {
//         matches!(self, Value::Undef)
//     }

//     #[inline]
//     pub fn is_raw(&self) -> bool {
//         matches!(self, Value::Raw(..))
//     }

//     #[inline]
//     pub fn is_array(&self) -> bool {
//         matches!(self, Value::Array {.. })
//     }

//     #[inline]
//     pub fn arr(&self) -> Ref<'_, Vec<Rc<Value>>> {
//         match self {
//             Value::Array { arr, .. } => arr.borrow(),
//             _ => panic!("unexpected Value type"),
//         }
//     }

//     #[inline]
//     pub fn arr_mut(&self) -> RefMut<'_, Vec<Rc<Value>>> {
//         match self {
//             Value::Array { arr, .. } => arr.borrow_mut(),
//             _ => panic!("unexpected Value type"),
//         }
//     }

//     #[inline]
//     pub fn as_raw(&self) -> &JsonValue {
//         match self {
//             Value::Raw(ref raw) => raw,
//             _ => panic!("unexpected Value type"),
//         }
//     }

//     array_flag!(outer_wrapper, is_wrapped);
//     array_flag!(is_seq, is_seq);
//     array_flag!(keep_array, keep_array, set_keep_array);
//     array_flag!(keep_singleton, keep_singleton, set_keep_singleton);
//     array_flag!(cons_array, cons_array, set_cons_array);

//     pub fn as_json(&self) -> Option<JsonValue> {
//         match self {
//             Value::Undef => None,
//             Value::Raw(raw) => Some(raw.clone()),
//             Value::Array { .. } => Some(JsonValue::Array(
//                 self.arr()
//                     .iter()
//                     .filter(|v| !v.is_undef())
//                     .map(|v| v.as_json().unwrap())
//                     .collect(),
//             )),
//         }
//     }

//     /// Returns the raw JSON value as a isize if it can be converted, and checks to ensure that it
//     /// is an integer (i.e. it returns None if there is any fractional part).
//     pub fn as_isize(&self) -> Option<isize> {
//         match self {
//             Value::Raw(raw) => match raw.as_f64() {
//                 Some(num) => {
//                     if num.trunc() == num {
//                         Some(num as isize)
//                     } else {
//                         None
//                     }
//                 }
//                 None => None,
//             },
//             _ => None,
//         }
//     }

//     /// Convenience method for determining if a value is an array of numbers
//     pub fn as_f64_vec(&self) -> Option<Vec<f64>> {
//         match self {
//             Value::Array { arr, .. } => {
//                 let mut nums = vec![];
//                 for value in arr.borrow().iter() {
//                     if let Some(num) = value.as_raw().as_f64() {
//                         nums.push(num);
//                     } else {
//                         return None;
//                     }
//                 }
//                 return Some(nums);
//             }
//             _ => None,
//         }
//     }
// }

// impl PartialEq for Value {
//     fn eq(&self, other: &Self) -> bool {
//         if self.is_undef() && other.is_undef() {
//             true
//         } else if self.is_raw() && other.is_raw() {
//             self.as_raw() == other.as_raw()
//         } else if self.is_array() && other.is_array() {
//             if self.arr().len() != other.arr().len() {
//                 false
//             } else {
//                 for i in 0..self.arr().len() - 1 {
//                     if self.arr()[i] != other.arr()[i] {
//                         return false;
//                     }
//                 }
//                 true
//             }
//         } else {
//             false
//         }
//     }
// }
