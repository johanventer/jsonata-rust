use super::value::{ArrayFlags, Value, ValueKind, ValuePool};

pub fn fn_test(pool: ValuePool) -> Value {
    Value::new_string(pool, "Hello From Rust!")
}

pub fn fn_lookup(pool: ValuePool, input: Value, key: Value) -> Value {
    match *input.as_ref() {
        ValueKind::Array { .. } => {
            let result = Value::new_array_with_flags(pool.clone(), ArrayFlags::SEQUENCE);

            for input in input.members() {
                let res = fn_lookup(pool.clone(), input, key.clone());
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
        ValueKind::Object(..) => input.get_entry(&key.as_string()),
        _ => pool.undefined(),
    }
}

pub fn fn_append(_pool: ValuePool, arg1: Value, arg2: Value) -> Value {
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

pub fn fn_boolean_internal(arg: Value) -> bool {
    fn cast(value: &Value) -> bool {
        match *value.as_ref() {
            ValueKind::Undefined => false,
            ValueKind::Null => false,
            ValueKind::Bool(b) => b,
            ValueKind::Number(num) => num != 0.0,
            ValueKind::String(ref str) => !str.is_empty(),
            ValueKind::Object(ref obj) => !obj.is_empty(),
            ValueKind::Array { .. } => unreachable!(),
            ValueKind::Lambda(..) => true,
            ValueKind::NativeFn0(..)
            | ValueKind::NativeFn1(..)
            | ValueKind::NativeFn2(..)
            | ValueKind::NativeFn3(..) => true,
        }
    }

    let result = match *arg.as_ref() {
        ValueKind::Array { .. } => match arg.len() {
            0 => false,
            1 => fn_boolean_internal(arg.get_member(0)),
            _ => arg.members().any(fn_boolean_internal),
        },
        _ => cast(&arg),
    };

    result
}

pub fn fn_boolean(pool: ValuePool, arg: Value) -> Value {
    Value::new_bool(pool, fn_boolean_internal(arg))
}
