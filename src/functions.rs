use json::{stringify, JsonValue};

use crate::evaluator::Value;

pub fn lookup(input: &Value, key: &str) -> Value {
    let result = if input.is_array() {
        let mut result = Value::new_seq();

        for value in input.iter() {
            let mut res = lookup(value, key);

            if !res.is_undef() {
                if res.is_array() {
                    res.drain(..).for_each(|v| result.push(v));
                } else {
                    result.push(res);
                }
            }
        }

        result
    } else if input.is_raw() && input.as_raw().has_key(key) {
        Value::new(Some(&input.as_raw()[key]))
    } else {
        Value::Undefined
    };

    result
}

pub fn boolean(arg: &Value) -> bool {
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

    match arg {
        Value::Undefined => false,
        Value::Raw(value) => cast(value),
        Value::Array { arr, .. } => match arr.len() {
            0 => false,
            1 => boolean(&arr[0]),
            _ => {
                let trues: Vec<_> = arr.iter().filter(|x| boolean(&x)).collect();
                !trues.is_empty()
            }
        },
        Value::Closure { .. } => false,
    }
}

pub fn append(mut arg1: Value, mut arg2: Value) -> Value {
    if arg1.is_undef() {
        return arg2;
    }

    if arg2.is_undef() {
        return arg1;
    }

    if !arg1.is_array() {
        arg1 = Value::new_seq_from(&arg1);
    }

    if !arg2.is_array() {
        arg1.push(arg2);
    } else {
        arg2.drain(..).for_each(|v| arg1.push(v));
    }

    arg1
}

pub fn string(mut arg: Value) -> Option<String> {
    // TODO: Prettify output (functions.js:108)

    if arg.is_undef() {
        None
    } else if arg.is_raw() {
        if arg.as_raw().is_string() {
            Some(arg.as_raw().as_str().unwrap().to_owned())
        } else {
            Some(stringify(arg.take()))
        }
    } else if arg.is_array() {
        Some(stringify(arg.to_json()))
    } else {
        Some("".to_owned())
    }
}
