use json::{stringify, JsonValue};
use std::rc::Rc;

use crate::evaluator::Value;

pub(crate) fn lookup(input: Rc<Value>, key: &str) -> Rc<Value> {
    let result = if input.is_array() {
        let result = Rc::new(Value::new_seq());

        for value in input.arr().iter() {
            let res = lookup(Rc::clone(value), key);

            if !res.is_undef() {
                if res.is_array() {
                    res.arr()
                        .iter()
                        .for_each(|v| result.arr_mut().push(Rc::clone(v)));
                } else {
                    result.arr_mut().push(res);
                }
            }
        }

        result
    } else if input.is_raw() && input.as_raw().has_key(key) {
        Rc::new(Value::from_raw(Some(&input.as_raw()[key])))
    } else {
        Rc::new(Value::Undef)
    };

    result
}

pub(crate) fn append(mut arg1: Rc<Value>, arg2: Rc<Value>) -> Rc<Value> {
    if arg1.is_undef() {
        return arg2;
    }

    if arg2.is_undef() {
        return arg1;
    }

    if !arg1.is_array() {
        arg1 = Rc::new(Value::seq_from(arg1));
    }

    if !arg2.is_array() {
        arg1.arr_mut().push(arg2);
    } else {
        arg2.arr()
            .iter()
            .for_each(|v| arg1.arr_mut().push(Rc::clone(v)));
    }

    arg1
}

pub(crate) fn string(arg: Rc<Value>) -> Option<String> {
    // TODO: Prettify output (functions.js:108)

    if arg.is_undef() {
        None
    } else if arg.is_raw() {
        if arg.as_raw().is_string() {
            Some(arg.as_raw().as_str().unwrap().to_owned())
        } else {
            Some(stringify(arg.as_json()))
        }
    } else if arg.is_array() {
        Some(stringify(arg.as_json()))
    } else {
        Some("".to_owned())
    }
}

pub(crate) fn boolean(arg: Rc<Value>) -> bool {
    fn cast(value: &JsonValue) -> bool {
        match value {
            JsonValue::Null => false,
            JsonValue::Short(value) => !value.is_empty(),
            JsonValue::String(value) => !value.is_empty(),
            JsonValue::Number(value) => !value.is_zero(),
            JsonValue::Boolean(value) => *value,
            JsonValue::Object(value) => !value.is_empty(),
            JsonValue::Array(..) => panic!("unexpected JsonValue::Array"),
        }
    }

    match *arg {
        Value::Undef => false,
        Value::Raw(ref value) => cast(value),
        Value::Array { ref arr, .. } => match arr.borrow().len() {
            0 => false,
            1 => boolean(Rc::clone(&arr.borrow()[0])),
            _ => {
                let arr = arr.borrow();
                let trues: Vec<_> = arr.iter().filter(|x| boolean(Rc::clone(&x))).collect();
                !trues.is_empty()
            }
        },
    }
}
