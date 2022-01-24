mod kind;

pub use kind::{ArrayFlags, Value, UNDEFINED};

use bumpalo::Bump;
use std::fmt;
use std::ops::Deref;

use crate::json::codegen::{DumpGenerator, Generator, PrettyGenerator};

// const _UNDEFINED: Value = Value::Undefined;
// pub const UNDEFINED: ValuePtr = ValuePtr(&_UNDEFINED);

thread_local! {
    static ARENA: Bump = Bump::new();
}

#[derive(Clone, Copy)]
pub struct ValuePtr(*const Value);

impl ValuePtr {
    // pub fn ptr(&self) -> *const ValueKind {
    //     self.0
    // }

    pub fn new(kind: Value) -> ValuePtr {
        ValuePtr(ARENA.with(|arena| arena.alloc(kind) as *const Value))
    }

    // pub fn null() -> ValuePtr {
    //     ValuePtr(ARENA.with(|arena| arena.alloc(Value::Null) as *const Value))
    // }

    // pub fn bool(value: bool) -> ValuePtr {
    //     ValuePtr(ARENA.with(|arena| arena.alloc(Value::Bool(value)) as *const Value))
    // }

    // pub fn number<T: Into<Number>>(value: T) -> ValuePtr {
    //     ValuePtr(ARENA.with(|arena| arena.alloc(Value::Number(value.into())) as *const Value))
    // }

    // pub fn string<T: Into<String>>(value: T) -> ValuePtr {
    //     ValuePtr(ARENA.with(|arena| arena.alloc(Value::String(value.into())) as *const Value))
    // }

    // pub fn array(flags: ArrayFlags) -> ValuePtr {
    //     ValuePtr(ARENA.with(|arena| arena.alloc(Value::Array(Vec::new(), flags)) as *const Value))
    // }

    // pub fn array_with_capacity(capacity: usize, flags: ArrayFlags) -> ValuePtr {
    //     ValuePtr(ARENA.with(|arena| {
    //         arena.alloc(Value::Array(Vec::with_capacity(capacity), flags)) as *const Value
    //     }))
    // }

    // pub fn object() -> ValuePtr {
    //     ValuePtr(ARENA.with(|arena| arena.alloc(Value::Object(HashMap::new())) as *const Value))
    // }

    // pub fn object_with_capacity(capacity: usize) -> ValuePtr {
    //     ValuePtr(ARENA.with(|arena| {
    //         arena.alloc(Value::Object(HashMap::with_capacity(capacity))) as *const Value
    //     }))
    // }

    // pub fn lambda(node: &Ast, input: ValuePtr, frame: Frame) -> ValuePtr {
    //     ValuePtr(ARENA.with(|arena| {
    //         arena.alloc(Value::Lambda {
    //             ast: node,
    //             input,
    //             frame,
    //         }) as *const Value
    //     }))
    // }

    // pub fn nativefn0(name: &str, func: fn(&FunctionContext) -> Result<ValuePtr>) -> ValuePtr {
    //     ValuePtr(
    //         ARENA.with(|arena| {
    //             arena.alloc(Value::NativeFn0(name.to_string(), func)) as *const Value
    //         }),
    //     )
    // }

    // pub fn nativefn1(
    //     name: &str,
    //     func: fn(&FunctionContext, ValuePtr) -> Result<ValuePtr>,
    // ) -> ValuePtr {
    //     ValuePtr(
    //         ARENA.with(|arena| {
    //             arena.alloc(Value::NativeFn1(name.to_string(), func)) as *const Value
    //         }),
    //     )
    // }

    // pub fn nativefn2(
    //     name: &str,
    //     func: fn(&FunctionContext, ValuePtr, ValuePtr) -> Result<ValuePtr>,
    // ) -> ValuePtr {
    //     ValuePtr(
    //         ARENA.with(|arena| {
    //             arena.alloc(Value::NativeFn2(name.to_string(), func)) as *const Value
    //         }),
    //     )
    // }

    // pub fn nativefn3(
    //     name: &str,
    //     func: fn(&FunctionContext, ValuePtr, ValuePtr, ValuePtr) -> Result<ValuePtr>,
    // ) -> ValuePtr {
    //     ValuePtr(
    //         ARENA.with(|arena| {
    //             arena.alloc(Value::NativeFn3(name.to_string(), func)) as *const Value
    //         }),
    //     )
    // }

    // pub fn is_undefined(&self) -> bool {
    //     matches!(unsafe { &*self.0 }, Value::Undefined)
    // }

    // pub fn is_null(&self) -> bool {
    //     matches!(unsafe { &*self.0 }, Value::Null)
    // }

    // pub fn is_bool(&self) -> bool {
    //     matches!(unsafe { &*self.0 }, Value::Bool(..))
    // }

    // pub fn is_number(&self) -> bool {
    //     matches!(unsafe { &*self.0 }, Value::Number(..))
    // }

    // pub fn is_integer(&self) -> bool {
    //     if let Value::Number(ref n) = unsafe { &*self.0 } {
    //         let n = f64::from(*n);
    //         match n.classify() {
    //             std::num::FpCategory::Nan
    //             | std::num::FpCategory::Infinite
    //             | std::num::FpCategory::Subnormal => false,
    //             _ => {
    //                 let mantissa = n.trunc();
    //                 n - mantissa == 0.0
    //             }
    //         }
    //     } else {
    //         false
    //     }
    // }

    // pub fn is_nan(&self) -> bool {
    //     matches!(unsafe { &*self.0 }, Value::Number(n) if n.is_nan())
    // }

    // pub fn is_string(&self) -> bool {
    //     matches!(unsafe { &*self.0 }, Value::String(..))
    // }

    // pub fn is_array(&self) -> bool {
    //     matches!(unsafe { &*self.0 }, Value::Array(..))
    // }

    // pub fn is_object(&self) -> bool {
    //     matches!(unsafe { &*self.0 }, Value::Object(..))
    // }

    // pub fn is_function(&self) -> bool {
    //     matches!(
    //         unsafe { &*self.0 },
    //         Value::Lambda { .. }
    //             | Value::NativeFn0(..)
    //             | Value::NativeFn1(..)
    //             | Value::NativeFn2(..)
    //             | Value::NativeFn3(..)
    //     )
    // }

    // pub fn is_truthy(&self) -> bool {
    //     match unsafe { &*self.0 } {
    //         Value::Undefined => false,
    //         Value::Null => false,
    //         Value::Number(n) => *n != 0.0,
    //         Value::Bool(b) => *b,
    //         Value::String(s) => !s.is_empty(),
    //         Value::Array(a, _) => match a.len() {
    //             0 => false,
    //             1 => self.get_member(0).is_truthy(),
    //             _ => {
    //                 for item in self.members() {
    //                     if item.is_truthy() {
    //                         return true;
    //                     }
    //                 }
    //                 false
    //             }
    //         },
    //         Value::Object(o) => !o.is_empty(),
    //         Value::Lambda { .. }
    //         | Value::NativeFn0(..)
    //         | Value::NativeFn1(..)
    //         | Value::NativeFn2(..)
    //         | Value::NativeFn3(..) => false,
    //     }
    // }

    // pub fn arity(&self) -> usize {
    //     match unsafe { &*self.0 } {
    //         Value::Lambda { ast, .. } => {
    //             if let AstKind::Lambda { args, .. } = unsafe { &(**ast).kind } {
    //                 args.len()
    //             } else {
    //                 0
    //             }
    //         }
    //         Value::NativeFn0(..) => 0,
    //         Value::NativeFn1(..) => 1,
    //         Value::NativeFn2(..) => 2,
    //         Value::NativeFn3(..) => 3,
    //         _ => panic!("Not a function"),
    //     }
    // }

    // pub fn as_bool(&self) -> bool {
    //     match unsafe { &*self.0 } {
    //         Value::Bool(b) => *b,
    //         _ => panic!("Not a bool"),
    //     }
    // }

    // pub fn as_f32(&self) -> f32 {
    //     match unsafe { &*self.0 } {
    //         Value::Number(n) => f32::from(*n),
    //         _ => panic!("Not a number"),
    //     }
    // }

    // pub fn as_f64(&self) -> f64 {
    //     match unsafe { &*self.0 } {
    //         Value::Number(n) => f64::from(*n),
    //         _ => panic!("Not a number"),
    //     }
    // }

    // pub fn as_usize(&self) -> usize {
    //     match unsafe { &*self.0 } {
    //         Value::Number(ref n) => f64::from(*n) as usize,
    //         _ => panic!("Not a number"),
    //     }
    // }

    // pub fn as_isize(&self) -> isize {
    //     match unsafe { &*self.0 } {
    //         Value::Number(ref n) => f64::from(*n) as isize,
    //         _ => panic!("Not a number"),
    //     }
    // }

    // pub fn as_str(&self) -> Cow<'_, str> {
    //     match unsafe { &*self.0 } {
    //         Value::String(ref s) => Cow::from(s),
    //         _ => panic!("Not a string"),
    //     }
    // }

    // pub fn len(&self) -> usize {
    //     match unsafe { &*self.0 } {
    //         Value::Array(array, _) => array.len(),
    //         _ => panic!("Not an array"),
    //     }
    // }

    // pub fn is_empty(&self) -> bool {
    //     match unsafe { &*self.0 } {
    //         Value::Array(array, _) => array.is_empty(),
    //         _ => panic!("Not an array"),
    //     }
    // }

    // pub fn get_member(&self, index: usize) -> ValuePtr {
    //     match unsafe { &*self.0 } {
    //         Value::Array(ref array, _) => match array.get(index) {
    //             Some(value) => *value,
    //             None => UNDEFINED,
    //         },
    //         _ => panic!("Not an array"),
    //     }
    // }

    // pub fn get_entry(&self, key: &str) -> ValuePtr {
    //     match unsafe { &*self.0 } {
    //         Value::Object(ref map) => match map.get(key) {
    //             Some(value) => *value,
    //             None => UNDEFINED,
    //         },
    //         _ => panic!("Not an object"),
    //     }
    // }

    // pub fn insert_new(&self, key: &str, kind: Value) {
    //     match unsafe { &mut *(self.0 as *mut Value) } {
    //         Value::Object(ref mut map) => {
    //             let kind = ARENA.with(|arena| arena.alloc(kind) as *const Value);
    //             map.insert(key.to_owned(), ValuePtr(kind));
    //         }
    //         _ => panic!("Not an object"),
    //     }
    // }

    // pub fn insert(&self, key: &str, value: ValuePtr) {
    //     match unsafe { &mut *(self.0 as *mut Value) } {
    //         Value::Object(ref mut map) => {
    //             map.insert(key.to_owned(), value);
    //         }
    //         _ => panic!("Not an object"),
    //     }
    // }

    // pub fn push_new(&self, kind: Value) {
    //     match unsafe { &mut *(self.0 as *mut Value) } {
    //         Value::Array(ref mut array, _) => {
    //             let kind = ARENA.with(|arena| arena.alloc(kind) as *const Value);
    //             array.push(ValuePtr(kind));
    //         }
    //         _ => panic!("Not an array"),
    //     }
    // }

    // pub fn push(&self, value: ValuePtr) {
    //     match unsafe { &mut *(self.0 as *mut Value) } {
    //         Value::Array(ref mut array, _) => array.push(value),
    //         _ => panic!("Not an array"),
    //     }
    // }

    // pub fn wrap_in_array(&self, flags: ArrayFlags) -> ValuePtr {
    //     let array = vec![*self];
    //     let result = ARENA.with(|arena| arena.alloc(Value::Array(array, flags)) as *const Value);
    //     ValuePtr(result)
    // }

    // pub fn wrap_in_array_if_needed(&self, flags: ArrayFlags) -> ValuePtr {
    //     if self.is_array() {
    //         *self
    //     } else {
    //         self.wrap_in_array(flags)
    //     }
    // }

    // pub fn members(&self) -> std::slice::Iter<'_, ValuePtr> {
    //     match unsafe { &*self.0 } {
    //         Value::Array(ref array, _) => array.iter(),
    //         _ => panic!("Not an array"),
    //     }
    // }

    // pub fn entries(&self) -> hashbrown::hash_map::Iter<'_, String, ValuePtr> {
    //     match unsafe { &*self.0 } {
    //         Value::Object(ref map) => map.iter(),
    //         _ => panic!("Not an object"),
    //     }
    // }

    pub fn get_flags(&self) -> ArrayFlags {
        match unsafe { &*self.0 } {
            Value::Array(_, flags) => *flags,
            _ => panic!("Not an array"),
        }
    }

    pub fn set_flags(&mut self, new_flags: ArrayFlags) {
        match unsafe { &mut *(self.0 as *mut Value) } {
            Value::Array(_, flags) => *flags = new_flags,
            _ => panic!("Not an array"),
        }
    }

    pub fn add_flags(&mut self, flags_to_add: ArrayFlags) {
        match unsafe { &mut *(self.0 as *mut Value) } {
            Value::Array(_, flags) => flags.insert(flags_to_add),
            _ => panic!("Not an array"),
        }
    }

    pub fn has_flags(&self, check_flags: ArrayFlags) -> bool {
        match unsafe { &*self.0 } {
            Value::Array(_, flags) => flags.contains(check_flags),
            _ => false,
        }
    }

    // Prints out the value as JSON string.
    pub fn dump(&self) -> String {
        let mut gen = DumpGenerator::new();
        gen.write_json(self).expect("Can't fail");
        gen.consume()
    }

    /// Pretty prints out the value as JSON string. Takes an argument that's
    /// number of spaces to indent new blocks with.
    pub fn pretty(&self, spaces: u16) -> String {
        let mut gen = PrettyGenerator::new(spaces);
        gen.write_json(self).expect("Can't fail");
        gen.consume()
    }
}

impl std::fmt::Debug for ValuePtr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match unsafe { &*self.0 } {
            Value::Lambda { ast, .. } => write!(f, "<lambda: {:#?}>", ast),
            Value::NativeFn0(..) => write!(f, "<nativefn0>"),
            Value::NativeFn1(..) => write!(f, "<nativefn1>"),
            Value::NativeFn2(..) => write!(f, "<nativefn2>"),
            Value::NativeFn3(..) => write!(f, "<nativefn3>"),
            _ => write!(f, "{}", self.dump()),
        }
    }
}

impl Deref for ValuePtr {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl PartialEq<ValuePtr> for ValuePtr {
    fn eq(&self, other: &ValuePtr) -> bool {
        match unsafe { &*self.0 } {
            Value::Array(..) => {
                if other.is_array() && other.len() == self.len() {
                    self.members().zip(other.members()).all(|(l, r)| l == r)
                } else {
                    false
                }
            }
            Value::Object(..) => {
                if other.is_object() {
                    self.entries().all(|(k, v)| &**v == other.get_entry(k))
                } else {
                    false
                }
            }
            _ => unsafe { *self.0 == **other },
        }
    }
}

impl PartialEq<Value> for ValuePtr {
    fn eq(&self, other: &Value) -> bool {
        unsafe { *self.0 == *other }
    }
}

impl PartialEq<bool> for ValuePtr {
    fn eq(&self, other: &bool) -> bool {
        match unsafe { &*self.0 } {
            Value::Bool(ref b) => *b == *other,
            _ => false,
        }
    }
}

impl PartialEq<i32> for ValuePtr {
    fn eq(&self, other: &i32) -> bool {
        match unsafe { &*self.0 } {
            Value::Number(ref n) => *n == *other,
            _ => false,
        }
    }
}

impl PartialEq<i64> for ValuePtr {
    fn eq(&self, other: &i64) -> bool {
        match unsafe { &*self.0 } {
            Value::Number(ref n) => *n == *other,
            _ => false,
        }
    }
}

impl PartialEq<f32> for ValuePtr {
    fn eq(&self, other: &f32) -> bool {
        match unsafe { &*self.0 } {
            Value::Number(ref n) => *n == *other,
            _ => false,
        }
    }
}

impl PartialEq<f64> for ValuePtr {
    fn eq(&self, other: &f64) -> bool {
        match unsafe { &*self.0 } {
            Value::Number(n) => *n == *other,
            _ => false,
        }
    }
}

impl PartialEq<&str> for ValuePtr {
    fn eq(&self, other: &&str) -> bool {
        match unsafe { &*self.0 } {
            Value::String(s) => *s == **other,
            _ => false,
        }
    }
}

impl PartialEq<String> for ValuePtr {
    fn eq(&self, other: &String) -> bool {
        match unsafe { &*self.0 } {
            Value::String(s) => *s == *other,
            _ => false,
        }
    }
}
