use bitflags::bitflags;
use bumpalo::{collections::string::String, collections::vec::Vec};
use hashbrown::{hash_map::DefaultHashBuilder, BumpWrapper, HashMap};

use super::{
    iterator::{EntryIterator, MemberIterator},
    range::Range,
    Value,
};
use crate::parser::ast::Ast;

bitflags! {
    pub struct ArrayFlags: u8 {
        const SEQUENCE     = 0b00000001;
        const SINGLETON    = 0b00000010;
        const CONS         = 0b00000100;
        const WRAPPED      = 0b00001000;
        const TUPLE_STREAM = 0b00010000;
    }
}

pub enum ValueKind<'arena> {
    Undefined,
    Null,
    Number(f64),
    Bool(bool),
    String(String<'arena>),
    Array(Vec<'arena, Value<'arena>>, ArrayFlags),
    Object(HashMap<&'arena str, Value<'arena>, DefaultHashBuilder, BumpWrapper<'arena>>),
    Range(Range<'arena>),
    // Lambda {
    //     ast: Box<'a, Ast>,
    //     input: &'a Value<'a>,
    //     frame: Frame<'a>,
    // },
    // NativeFn {
    //     name: String,
    //     arity: usize,
    //     func: fn(FunctionContext<'a, '_>, &'a Value<'a>) -> Result<&'a Value<'a>>,
    // },
    Transformer {
        pattern: &'arena Ast,
        update: &'arena Ast,
        delete: Option<&'arena Ast>,
    },
}

impl<'arena> ValueKind<'arena> {
    /*
        Identity functions
    */

    pub fn is_undefined(&self) -> bool {
        matches!(self, ValueKind::Undefined)
    }

    pub fn is_null(&self) -> bool {
        matches!(self, ValueKind::Null)
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, ValueKind::Bool(..))
    }

    pub fn is_number(&self) -> bool {
        matches!(self, ValueKind::Number(..))
    }

    pub fn is_integer(&self) -> bool {
        match self {
            ValueKind::Number(n) => match n.classify() {
                std::num::FpCategory::Nan
                | std::num::FpCategory::Infinite
                | std::num::FpCategory::Subnormal => false,
                _ => {
                    let mantissa = n.trunc();
                    n - mantissa == 0.0
                }
            },
            _ => false,
        }
    }

    pub fn is_nan(&self) -> bool {
        matches!(self, ValueKind::Number(n) if n.is_nan())
    }

    pub fn is_finite(&self) -> bool {
        matches!(self, ValueKind::Number(n) if n.is_finite())
    }

    pub fn is_string(&self) -> bool {
        matches!(*self, ValueKind::String(..))
    }

    pub fn is_array(&self) -> bool {
        matches!(*self, ValueKind::Array(..) | ValueKind::Range(..))
    }

    pub fn is_object(&self) -> bool {
        matches!(*self, ValueKind::Object(..))
    }

    pub fn len(&self) -> usize {
        match self {
            ValueKind::Array(ref array, _) => array.len(),
            ValueKind::Range(ref range) => range.len(),
            ValueKind::Object(ref hash) => hash.len(),
            _ => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            ValueKind::Array(ref array, _) => array.is_empty(),
            ValueKind::Range(ref range) => range.is_empty(),
            ValueKind::Object(ref hash) => hash.is_empty(),
            _ => true,
        }
    }

    /*
        Conversion functions
    */

    pub fn as_bool(&self) -> bool {
        match self {
            ValueKind::Bool(ref b) => *b,
            _ => panic!("Not a bool"),
        }
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            ValueKind::Number(n) => *n,
            _ => panic!("Not a number"),
        }
    }

    // TODO(math): Completely unchecked, audit usage
    pub fn as_usize(&self) -> usize {
        match self {
            ValueKind::Number(n) => *n as usize,
            _ => panic!("Not a number"),
        }
    }

    // TODO(math): Completely unchecked, audit usage
    pub fn as_isize(&self) -> isize {
        match self {
            ValueKind::Number(n) => *n as isize,
            _ => panic!("Not a number"),
        }
    }

    pub fn as_str(&'arena self) -> &'arena str {
        match self {
            ValueKind::String(ref s) => s.as_str(),
            _ => panic!("Not a string"),
        }
    }

    /*
        Array functions
    */

    pub fn get_member(&self, index: usize) -> Option<Value<'arena>> {
        match self {
            ValueKind::Array(ref array, ..) => array.get(index).copied(),
            _ => panic!("Not an array"),
        }
    }

    pub fn members<'a>(&'a self) -> MemberIterator<'a, 'arena> {
        match self {
            ValueKind::Array(..) | ValueKind::Range(..) => MemberIterator::new(self),
            _ => panic!("Not an array"),
        }
    }

    pub fn push(&mut self, value: Value<'arena>) {
        match self {
            ValueKind::Range(..) => panic!("Can't mutate a Range"),
            ValueKind::Array(ref mut array, ..) => array.push(value),
            _ => panic!("Not an array"),
        }
    }

    pub fn get_flags(&self) -> ArrayFlags {
        match self {
            ValueKind::Array(_, flags) => *flags,
            _ => panic!("Not an array"),
        }
    }

    pub fn has_flags(&self, check_flags: ArrayFlags) -> bool {
        matches!(self, ValueKind::Array(_, flags) if flags.contains(check_flags))
    }

    /*
        Object functions
    */

    pub fn get_entry(&self, key: &'arena str) -> Option<Value<'arena>> {
        match self {
            ValueKind::Object(ref hash, ..) => hash.get(&key).copied(),
            _ => panic!("Not an array"),
        }
    }

    pub fn entries<'a>(&'a self) -> EntryIterator<'a, 'arena> {
        match self {
            ValueKind::Object(..) => EntryIterator::new(self),
            _ => panic!("Not an object"),
        }
    }

    pub fn insert(&mut self, key: &'arena str, value: Value<'arena>) {
        match self {
            ValueKind::Object(ref mut hash) => hash.insert(key, value),
            _ => panic!("Not an object"),
        };
    }

    pub fn remove(&mut self, key: &'arena str) {
        match self {
            ValueKind::Object(ref mut hash) => hash.remove(&key),
            _ => panic!("Not an object"),
        };
    }
}
