// use json::{stringify, JsonValue};
// use std::rc::Rc;

use crate::evaluator::Value;

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

// pub(crate) fn append(mut arg1: Rc<Value>, arg2: Rc<Value>) -> Rc<Value> {
//     if arg1.is_undef() {
//         return arg2;
//     }

//     if arg2.is_undef() {
//         return arg1;
//     }

//     if !arg1.is_array() {
//         arg1 = Rc::new(Value::seq_from(arg1));
//     }

//     if !arg2.is_array() {
//         arg1.arr_mut().push(arg2);
//     } else {
//         arg2.arr()
//             .iter()
//             .for_each(|v| arg1.arr_mut().push(Rc::clone(v)));
//     }

//     arg1
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

pub(crate) fn boolean(arg: &Value) -> bool {
    fn cast(value: &Value) -> bool {
        match *value {
            Value::Undefined => false,
            Value::Null => false,
            Value::Bool(b) => b,
            Value::Number(num) => num != 0.0,
            Value::String(ref str) => !str.is_empty(),
            Value::Object(ref obj) => !obj.is_empty(),
            Value::Array(_) => panic!("unexpected Value::Array"),
        }
    }

    match *arg {
        Value::Array(ref arr) => match arr.len() {
            0 => false,
            1 => boolean(&arr[0]),
            _ => arr.iter().any(boolean),
        },
        _ => cast(arg),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean() {
        assert!(!boolean(&Value::Undefined));
        assert!(!boolean(&Value::Null));
        assert!(!boolean(&Value::Bool(false)));
        assert!(boolean(&Value::Bool(true)));
        assert!(!boolean(&Value::Number(0.0.into())));
        assert!(boolean(&Value::Number(1.0.into())));
        assert!(!boolean(&Value::String("".to_owned())));
        assert!(boolean(&Value::String("x".to_owned())));
        assert!(!boolean(&Value::new_object()));
        let mut obj = Value::new_object();
        obj.insert("hello", Value::Null);
        assert!(boolean(&obj));
        assert!(!boolean(&Value::Array(Vec::new())));
        assert!(boolean(&Value::Array(vec![Value::Bool(true)])));
        assert!(!boolean(&Value::Array(vec![Value::Bool(false)])));
        assert!(!boolean(&Value::Array(vec![
            Value::Bool(false),
            Value::Bool(false),
            Value::Bool(false)
        ])));
        assert!(boolean(&Value::Array(vec![
            Value::Bool(false),
            Value::Bool(true),
            Value::Bool(false)
        ])));
    }
}
