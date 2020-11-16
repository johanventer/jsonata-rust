use json::JsonValue;
use std::ops::{Index, RangeBounds};
use std::slice::Iter;
use std::vec::Drain;

#[derive(Clone, Debug)]
pub enum Value {
    Undefined,
    Raw(JsonValue),
    Array {
        arr: Vec<Value>,
        is_seq: bool,
        keep_array: bool,
        keep_singleton: bool,
        cons_array: bool,
        outer_wrapper: bool,
    },
}

impl Value {
    pub fn new(raw: Option<&JsonValue>) -> Self {
        match raw {
            None => Self::Undefined,
            Some(raw) => match raw {
                JsonValue::Array(arr) => Self::Array {
                    arr: arr.iter().map(|v| Self::new(Some(v))).collect(),
                    is_seq: false,
                    keep_array: false,
                    keep_singleton: false,
                    cons_array: false,
                    outer_wrapper: false,
                },
                _ => Self::Raw(raw.clone()),
            },
        }
    }

    pub fn new_array() -> Self {
        Self::Array {
            arr: vec![],
            is_seq: false,
            keep_array: false,
            keep_singleton: false,
            cons_array: false,
            outer_wrapper: false,
        }
    }

    pub fn new_seq_with_capacity(capacity: usize) -> Self {
        let arr: Vec<Value> = Vec::with_capacity(capacity);
        Self::Array {
            arr,
            is_seq: true,
            keep_array: false,
            keep_singleton: false,
            cons_array: false,
            outer_wrapper: false,
        }
    }

    pub fn new_seq() -> Self {
        Self::Array {
            arr: vec![],
            is_seq: true,
            keep_array: false,
            keep_singleton: false,
            cons_array: false,
            outer_wrapper: false,
        }
    }

    pub fn new_seq_from(value: &Value) -> Self {
        Self::Array {
            arr: vec![value.clone()],
            is_seq: true,
            keep_array: false,
            keep_singleton: false,
            cons_array: false,
            outer_wrapper: false,
        }
    }

    pub fn wrap(value: &Value) -> Self {
        Self::Array {
            arr: vec![value.clone()],
            is_seq: true,
            keep_array: false,
            keep_singleton: false,
            cons_array: false,
            outer_wrapper: true,
        }
    }

    pub fn is_wrapped(&self) -> bool {
        match self {
            Value::Array { outer_wrapper, .. } => *outer_wrapper,
            _ => false,
        }
    }

    pub fn unwrap(self) -> Value {
        match self {
            Value::Array { mut arr, .. } => arr.swap_remove(0),
            _ => panic!("unexpected Value type"),
        }
    }

    pub fn is_undef(&self) -> bool {
        match self {
            Value::Undefined => true,
            _ => false,
        }
    }

    pub fn is_raw(&self) -> bool {
        match self {
            Value::Raw(..) => true,
            _ => false,
        }
    }

    pub fn is_array(&self) -> bool {
        match self {
            Value::Array { .. } => true,
            _ => false,
        }
    }

    pub fn is_seq(&self) -> bool {
        match self {
            Value::Array { is_seq, .. } => *is_seq,
            _ => false,
        }
    }

    pub fn is_object(&self) -> bool {
        match self {
            Value::Raw(raw) => raw.is_object(),
            _ => false,
        }
    }

    pub fn as_raw(&self) -> &JsonValue {
        match self {
            Value::Raw(value) => &value,
            _ => panic!("unexpected Value type"),
        }
    }

    pub fn into_raw(self) -> JsonValue {
        match self {
            Value::Raw(value) => value,
            Value::Array { .. } => self.to_json().unwrap(),
            _ => panic!("unexpected Value type"),
        }
    }

    pub fn as_array_mut(&mut self) -> &mut Vec<Value> {
        match self {
            Value::Array { arr, .. } => arr,
            _ => panic!("unexpected Value type"),
        }
    }

    pub fn as_array_owned(self) -> Vec<Value> {
        match self {
            Value::Array { arr, .. } => arr,
            _ => panic!("unexpected Value type"),
        }
    }

    pub fn keep_array(&self) -> bool {
        match self {
            Value::Array { keep_array, .. } => *keep_array,
            _ => false,
        }
    }

    pub fn set_keep_array(&mut self) {
        match self {
            Value::Array { keep_array, .. } => *keep_array = true,
            _ => panic!("unexpected Value type"),
        }
    }

    pub fn keep_singleton(&self) -> bool {
        match self {
            Value::Array { keep_singleton, .. } => *keep_singleton,
            _ => false,
        }
    }

    pub fn set_keep_singleton(&mut self) {
        match self {
            Value::Array { keep_singleton, .. } => *keep_singleton = true,
            _ => panic!("unexpected Value type"),
        }
    }

    pub fn cons_array(&self) -> bool {
        match self {
            Value::Array { cons_array, .. } => *cons_array,
            _ => false,
        }
    }

    pub fn set_cons_array(&mut self) {
        match self {
            Value::Array { cons_array, .. } => *cons_array = true,
            _ => panic!("unexpected Value type"),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Value::Array { arr, .. } => arr.is_empty(),
            _ => panic!("unexpected Value type"),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Value::Array { arr, .. } => arr.len(),
            _ => panic!("unexpected Value type"),
        }
    }

    pub fn iter(&self) -> Iter<'_, Value> {
        match self {
            Value::Array { arr, .. } => arr.iter(),
            _ => panic!("unexpected Value type"),
        }
    }

    pub fn push(&mut self, value: Value) {
        match self {
            Value::Array { arr, .. } => arr.push(value),
            _ => panic!("unexpected Value type"),
        }
    }

    pub fn append(&mut self, value: Value) {
        match self {
            Value::Array { arr, .. } => arr.append(&mut value.as_array_owned()),
            _ => panic!("unexpected Value type"),
        }
    }

    pub fn drain<R>(&mut self, range: R) -> Drain<'_, Value>
    where
        R: RangeBounds<usize>,
    {
        match self {
            Value::Array { arr, .. } => arr.drain(range),
            _ => panic!("unexpected Value type"),
        }
    }

    pub fn take(&mut self) -> JsonValue {
        match self {
            Value::Raw(raw) => raw.take(),
            _ => panic!("unexpected Value type"),
        }
    }

    pub fn to_json(mut self) -> Option<JsonValue> {
        match self {
            Value::Undefined => None,
            Value::Raw(raw) => Some(raw),
            Value::Array { .. } => Some(JsonValue::Array(
                self.drain(..)
                    .filter(|v| !v.is_undef())
                    .map(|v| v.to_json().unwrap())
                    .collect(),
            )),
        }
    }

    /// Returns the raw JSON value as a usize if it can be converted, and checks to ensure that it
    /// is an integer (i.e. it returns None if there is any fractional part).
    pub fn as_usize(&self) -> Option<usize> {
        match self {
            Value::Raw(raw) => match raw.as_f64() {
                Some(num) => {
                    if num.trunc() == num {
                        Some(num as usize)
                    } else {
                        None
                    }
                }
                None => None,
            },
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Raw(raw) => match raw.as_f64() {
                Some(num) => {
                    if num.is_finite() {
                        Some(num)
                    } else {
                        None
                    }
                }
                None => None,
            },
            _ => None,
        }
    }

    pub fn as_f64_vec(&self) -> Option<Vec<f64>> {
        match self {
            Value::Array { arr, .. } => {
                let mut nums = vec![];
                for value in arr {
                    if let Some(num) = value.as_f64() {
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

    pub fn as_string(&self) -> Option<String> {
        match self {
            Value::Raw(raw) => {
                if raw.is_string() {
                    Some(raw.as_str().unwrap().to_owned())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Index<usize> for Value {
    type Output = Value;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Value::Array { arr, .. } => &arr[index],
            _ => panic!("unexpected Value type"),
        }
    }
}

impl From<Value> for Option<JsonValue> {
    fn from(value: Value) -> Self {
        value.to_json()
    }
}

// TODO: Cleanup new and those From impls

impl From<JsonValue> for Value {
    fn from(raw: JsonValue) -> Self {
        match raw {
            JsonValue::Array(arr) => Self::Array {
                arr: arr.iter().map(|v| Self::new(Some(v))).collect(),
                is_seq: false,
                keep_array: false,
                keep_singleton: false,
                cons_array: false,
                outer_wrapper: false,
            },
            _ => Self::Raw(raw.clone()),
        }
    }
}

impl From<&JsonValue> for Value {
    fn from(raw: &JsonValue) -> Self {
        match raw {
            JsonValue::Array(arr) => Self::Array {
                arr: arr.iter().map(|v| Self::new(Some(v))).collect(),
                is_seq: false,
                keep_array: false,
                keep_singleton: false,
                cons_array: false,
                outer_wrapper: false,
            },
            _ => Self::Raw(raw.clone()),
        }
    }
}

impl From<Option<JsonValue>> for Value {
    fn from(raw: Option<JsonValue>) -> Self {
        match raw {
            None => Value::Undefined,
            Some(raw) => match raw {
                JsonValue::Array(arr) => Self::Array {
                    arr: arr.iter().map(|v| Self::new(Some(v))).collect(),
                    is_seq: false,
                    keep_array: false,
                    keep_singleton: false,
                    cons_array: false,
                    outer_wrapper: false,
                },
                _ => Self::Raw(raw.clone()),
            },
        }
    }
}

impl From<Option<&JsonValue>> for Value {
    fn from(raw: Option<&JsonValue>) -> Self {
        match raw {
            None => Value::Undefined,
            Some(raw) => match raw {
                JsonValue::Array(arr) => Self::Array {
                    arr: arr.iter().map(|v| Self::new(Some(v))).collect(),
                    is_seq: false,
                    keep_array: false,
                    keep_singleton: false,
                    cons_array: false,
                    outer_wrapper: false,
                },
                _ => Self::Raw(raw.clone()),
            },
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
            if self.len() != other.len() {
                false
            } else {
                for i in 0..self.len() - 1 {
                    if self[i] != other[i] {
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
