use super::evaluator::Evaluator;
use super::frame::Frame;
use super::position::Position;
use super::value::{ArrayFlags, Value, ValueKind, ValuePool};
use super::Result;

#[derive(Clone)]
pub struct FunctionContext<'a> {
    pub position: Position,
    pub pool: ValuePool,
    pub input: Value,
    pub frame: Frame,
    pub evaluator: &'a Evaluator,
}

impl<'a> FunctionContext<'a> {
    pub fn evaluate_function(&self, proc: Value, args: Value) -> Result<Value> {
        self.evaluator.apply_function(
            self.position,
            self.input.clone(),
            proc,
            args,
            self.frame.clone(),
        )
    }
}

pub fn fn_test(context: FunctionContext) -> Result<Value> {
    Ok(Value::new_string(context.pool, "Hello From Rust!"))
}

pub fn fn_lookup_internal(context: FunctionContext, input: Value, key: &str) -> Value {
    match *input.as_ref() {
        ValueKind::Array { .. } => {
            let result = Value::new_array_with_flags(context.pool.clone(), ArrayFlags::SEQUENCE);

            for input in input.members() {
                let res = fn_lookup_internal(context.clone(), input, key);
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
        _ => context.pool.undefined(),
    }
}

pub fn fn_lookup(context: FunctionContext, input: Value, key: Value) -> Result<Value> {
    Ok(fn_lookup_internal(context, input, &key.as_string()))
}

pub fn fn_append(_context: FunctionContext, arg1: Value, arg2: Value) -> Result<Value> {
    if arg1.is_undefined() {
        return Ok(arg2);
    }

    if arg2.is_undefined() {
        return Ok(arg1);
    }

    let arg1 = arg1.wrap_in_array_if_needed(ArrayFlags::SEQUENCE);
    let arg2 = arg2.wrap_in_array_if_needed(ArrayFlags::empty());

    arg2.members().for_each(|m| arg1.push_index(m.index));

    Ok(arg1)
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

pub fn fn_boolean(context: FunctionContext, arg: Value) -> Result<Value> {
    Ok(Value::new_bool(context.pool, fn_boolean_internal(arg)))
}

pub fn fn_filter(context: FunctionContext, arr: Value, func: Value) -> Result<Value> {
    if arr.is_undefined() {
        return Ok(context.pool.undefined());
    }

    // TODO: These asserts are here because we don't have function signature validation
    debug_assert!(arr.is_array());
    debug_assert!(func.is_function());

    let result = Value::new_array_with_flags(context.pool.clone(), ArrayFlags::SEQUENCE);

    for (index, item) in arr.members().enumerate() {
        let args = Value::new_array(context.pool.clone());
        let index_arg = Value::new_number(context.pool.clone(), index);
        let arity = func.arity();

        args.push_index(item.index);
        if arity >= 2 {
            args.push_index(index_arg.index);
        }
        if arity >= 3 {
            args.push_index(arr.index);
        }

        if fn_boolean_internal(context.evaluate_function(func.clone(), args)?) {
            result.push_index(item.index);
        }

        index_arg.drop();
    }

    Ok(result)
}

pub fn fn_string(context: FunctionContext, arg: Value) -> Result<Value> {
    if arg.is_undefined() {
        return Ok(context.pool.undefined());
    }

    return if arg.is_string() {
        Ok(arg)
    } else if arg.is_function() {
        Ok(Value::new_string(context.pool, ""))

    // TODO: Check for infinite numbers
    // } else if arg.is_number() && arg.is_infinite() {
    //     // TODO: D3001
    //     unreachable!()

    // TODO: Proper JSON stringify of Value, pretty printing
    } else {
        Ok(Value::new_string(context.pool, &format!("{:?}", arg)))
    };
}

pub fn fn_count(context: FunctionContext, arg: Value) -> Result<Value> {
    Ok(Value::new_number(
        context.pool,
        if arg.is_undefined() {
            0
        } else if arg.is_array() {
            arg.len()
        } else {
            1
        },
    ))
}
