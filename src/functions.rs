use super::value::{ArrayFlags, Value, ValueKind};
use super::Evaluator;

impl Evaluator {
    pub fn lookup(&self, input: Value, key: &str) -> Value {
        match *input.as_ref() {
            ValueKind::Array { .. } => {
                let result = self.array(ArrayFlags::SEQUENCE);

                for input in input.members() {
                    let res = self.lookup(input, key);
                    match *res.as_ref() {
                        ValueKind::Undefined => {}
                        ValueKind::Array { .. } => {
                            res.members().for_each(|item| result.push_index(item.index));
                        }
                        _ => result.push_index(res.index),
                    };
                }

                result
            }
            ValueKind::Object(..) => input.get_entry(key),
            _ => self.pool.undefined(),
        }
    }

    pub fn append(&self, arg1: Value, arg2: Value) -> Value {
        if arg1.is_undefined() {
            return arg2;
        }

        if arg2.is_undefined() {
            return arg1;
        }

        let arg1 = arg1.wrap_in_array_if_needed(ArrayFlags::SEQUENCE);
        let arg2 = arg2.wrap_in_array_if_needed(ArrayFlags::empty());

        arg2.members().for_each(|m| arg1.push_index(m.index));

        arg1
    }

    pub fn boolean(&self, arg: Value) -> bool {
        fn cast(value: &Value) -> bool {
            match *value.as_ref() {
                ValueKind::Undefined => false,
                ValueKind::Null => false,
                ValueKind::Bool(b) => b,
                ValueKind::Number(num) => num != 0.0,
                ValueKind::String(ref str) => !str.is_empty(),
                ValueKind::Object(ref obj) => !obj.is_empty(),
                ValueKind::Array { .. } => unreachable!(),
            }
        }

        match *arg.as_ref() {
            ValueKind::Array { .. } => match arg.len() {
                0 => false,
                1 => self.boolean(arg.get_member(0)),
                _ => arg.members().any(|v| self.boolean(v)),
            },
            _ => cast(&arg),
        }
    }
}
// use json::{stringify, JsonValue};
// use std::rc::Rc;

// use super::value::Value;

// pub(crate) fn lookup(input: Rc<Value>, key: &str) -> Rc<Value> {
//     let result = if input.is_array() {
//         let result = Rc::new(Value::new_seq());

//         for value in input.arr().iter() {
//             let res = lookup(Rc::clone(value), key);

//             if !res.is_undef() {
//                 if res.is_array() {
//                     res.arr()
//                         .iter()
//                         .for_each(|v| result.arr_mut().push(Rc::clone(v)));
//                 } else {
//                     result.arr_mut().push(res);
//                 }
//             }
//         }

//         result
//     } else if input.is_raw() && input.as_raw().has_key(key) {
//         Rc::new(Value::from_raw(Some(&input.as_raw()[key])))
//     } else {
//         Rc::new(Value::Undef)
//     };

//     result
// }

// pub(crate) fn string(arg: Rc<Value>) -> Option<String> {
//     // TODO: Prettify output (functions.js:108)

//     if arg.is_undef() {
//         None
//     } else if arg.is_raw() {
//         if arg.as_raw().is_string() {
//             Some(arg.as_raw().as_str().unwrap().to_owned())
//         } else {
//             Some(stringify(arg.as_json()))
//         }
//     } else if arg.is_array() {
//         Some(stringify(arg.as_json()))
//     } else {
//         Some("".to_owned())
//     }
// }

// pub(crate) fn boolean(arg: &Value) -> bool {
//     fn cast(value: &Value) -> bool {
//         match *value {
//             Value::Undefined => false,
//             Value::Null => false,
//             Value::Bool(b) => b,
//             Value::Number(num) => num != 0.0,
//             Value::String(ref str) => !str.is_empty(),
//             Value::Object(ref obj) => !obj.is_empty(),
//             Value::Array { .. } => panic!("unexpected Value::Array"),
//         }
//     }

//     match *arg {
//         Value::Array { ref items, .. } => match items.len() {
//             0 => false,
//             1 => boolean(&items[0]),
//             _ => items.iter().any(boolean),
//         },
//         _ => cast(arg),
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_boolean() {
//         assert!(!boolean(&Value::Undefined));
//         assert!(!boolean(&Value::Null));
//         assert!(!boolean(&Value::Bool(false)));
//         assert!(boolean(&Value::Bool(true)));
//         assert!(!boolean(&Value::Number(0.0.into())));
//         assert!(boolean(&Value::Number(1.0.into())));
//         assert!(!boolean(&Value::String("".to_owned())));
//         assert!(boolean(&Value::String("x".to_owned())));
//         assert!(!boolean(&Value::new_object()));
//         let mut obj = Value::new_object();
//         obj.insert("hello", Value::Null);
//         assert!(boolean(&obj));
//         assert!(!boolean(&Value::new_array()));
//         assert!(boolean(&Value::with_items(vec![Value::Bool(true)])));
//         assert!(!boolean(&Value::with_items(vec![Value::Bool(false)])));
//         assert!(!boolean(&Value::with_items(vec![
//             Value::Bool(false),
//             Value::Bool(false),
//             Value::Bool(false)
//         ])));
//         assert!(boolean(&Value::with_items(vec![
//             Value::Bool(false),
//             Value::Bool(true),
//             Value::Bool(false)
//         ])));
//     }
// }
