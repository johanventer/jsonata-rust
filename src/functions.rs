use json::{array, JsonValue};

use crate::evaluator::Input;

pub fn lookup(input: &JsonValue, key: &str) -> Input {
    if input.is_array() {
        let mut values: Vec<Input> = Vec::new();
        values.reserve(input.len());

        for value in input.members() {
            let mut res = lookup(value, key);
            match &mut res {
                Input::Undefined => (),
                Input::Value(ref value) => {
                    if value.is_array() {
                        value
                            .members()
                            .cloned()
                            .for_each(|v| values.push(Input::Value(v)));
                    } else {
                        values.push(res);
                    }
                }
                Input::Sequence(ref mut seq, ..) => {
                    values.append(seq);
                }
            };
        }

        Input::Sequence(values, false)
    } else if input.is_object() && input.has_key(key) {
        Input::Value(input[key].clone())
    } else {
        Input::Undefined
    }
}

pub fn boolean(arg: &Input) -> bool {
    fn cast(value: &JsonValue) -> bool {
        match value {
            JsonValue::Null => false,
            JsonValue::Short(ref value) => !value.is_empty(),
            JsonValue::String(ref value) => !value.is_empty(),
            JsonValue::Number(ref value) => !value.is_zero(),
            JsonValue::Boolean(ref value) => *value,
            JsonValue::Object(ref value) => !value.is_empty(),
            JsonValue::Array(ref value) => match value.len() {
                0 => false,
                1 => cast(&value[0]),
                _ => {
                    let trues: Vec<_> = value.iter().filter(|x| cast(&x)).collect();
                    !trues.is_empty()
                }
            },
        }
    }

    match arg {
        Input::Undefined => false,
        Input::Value(ref value) => cast(value),
        Input::Sequence(ref seq, ..) => match seq.len() {
            0 => false,
            1 => boolean(&seq[0]),
            _ => {
                let trues: Vec<_> = seq.iter().filter(|x| boolean(&x)).collect();
                !trues.is_empty()
            }
        },
    }
}

pub fn append(arg1: Input, arg2: Input) -> Input {
    if let Input::Undefined = arg1 {
        return arg2;
    }

    if let Input::Undefined = arg2 {
        return arg1;
    }

    let mut arg1 = arg1.as_value().clone();
    let arg2 = arg2.as_value().clone();

    if !arg1.is_array() {
        Input::Value(JsonValue::Array(vec![arg1, arg2]))
    } else {
        arg1.push(arg2).unwrap();
        Input::Value(arg1)
    }
}
