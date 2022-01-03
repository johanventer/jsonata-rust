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
    pub fn evaluate_function(&self, proc: &Value, args: &Value) -> Result<Value> {
        self.evaluator
            .apply_function(self.position, &self.input, proc, args, &self.frame)
    }
}

pub fn fn_lookup_internal(context: &FunctionContext, input: &Value, key: &str) -> Value {
    match **input {
        ValueKind::Array { .. } => {
            let mut result = context.pool.array(ArrayFlags::SEQUENCE);

            for input in input.members() {
                let res = fn_lookup_internal(context, &input, key);
                match *res {
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

pub fn fn_lookup(context: &FunctionContext, input: &Value, key: &Value) -> Result<Value> {
    Ok(fn_lookup_internal(context, input, &key.as_string()))
}

pub fn fn_append(context: &FunctionContext, arg1: &Value, arg2: &Value) -> Result<Value> {
    if arg1.is_undefined() {
        return Ok(arg2.clone());
    }

    if arg2.is_undefined() {
        return Ok(arg1.clone());
    }

    let result = context.pool.value((**arg1).clone());
    let mut result = result.wrap_in_array_if_needed(ArrayFlags::SEQUENCE);
    let arg2 = arg2.wrap_in_array_if_needed(ArrayFlags::empty());
    arg2.members().for_each(|m| result.push_index(m.index));

    Ok(result)
}

pub fn fn_boolean(context: &FunctionContext, arg: &Value) -> Result<Value> {
    Ok(match **arg {
        ValueKind::Undefined => context.pool.undefined(),
        ValueKind::Null => context.pool.bool(false),
        ValueKind::Bool(b) => context.pool.bool(b),
        ValueKind::Number(num) => context.pool.bool(num != 0.0),
        ValueKind::String(ref str) => context.pool.bool(!str.is_empty()),
        ValueKind::Object(ref obj) => context.pool.bool(!obj.is_empty()),
        ValueKind::Array { .. } => match arg.len() {
            0 => context.pool.bool(false),
            1 => fn_boolean(context, &arg.get_member(0))?,
            _ => {
                for item in arg.members() {
                    if fn_boolean(context, &item)?.as_bool() {
                        return Ok(context.pool.bool(true));
                    }
                }
                context.pool.bool(false)
            }
        },
        ValueKind::Lambda(..)
        | ValueKind::NativeFn0(..)
        | ValueKind::NativeFn1(..)
        | ValueKind::NativeFn2(..)
        | ValueKind::NativeFn3(..) => context.pool.bool(false),
    })
}

pub fn fn_filter(context: &FunctionContext, arr: &Value, func: &Value) -> Result<Value> {
    if arr.is_undefined() {
        return Ok(context.pool.undefined());
    }

    // TODO: These asserts are here because we don't have function signature validation
    debug_assert!(arr.is_array());
    debug_assert!(func.is_function());

    let mut result = context.pool.array(ArrayFlags::SEQUENCE);

    for (index, item) in arr.members().enumerate() {
        let mut args = context.pool.array(ArrayFlags::empty());
        let arity = func.arity();

        args.push_index(item.index);
        if arity >= 2 {
            args.push(ValueKind::Number(index.into()));
        }
        if arity >= 3 {
            args.push_index(arr.index);
        }

        let include = context.evaluate_function(func, &args)?;

        if include.is_truthy() {
            result.push_index(item.index);
        }
    }

    Ok(result)
}

pub fn fn_string(context: &FunctionContext, arg: &Value) -> Result<Value> {
    if arg.is_undefined() {
        return Ok(context.pool.undefined());
    }

    if arg.is_string() {
        Ok(arg.clone())
    } else if arg.is_function() {
        Ok(context.pool.string(""))

    // TODO: Check for infinite numbers
    // } else if arg.is_number() && arg.is_infinite() {
    //     // TODO: D3001
    //     unreachable!()

    // TODO: pretty printing
    } else {
        Ok(context.pool.string(&arg.dump()))
    }
}

pub fn fn_count(context: &FunctionContext, arg: &Value) -> Result<Value> {
    Ok(context.pool.number(if arg.is_undefined() {
        0
    } else if arg.is_array() {
        arg.len()
    } else {
        1
    }))
}
