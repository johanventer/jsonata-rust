use bumpalo::Bump;
use lazy_static;

use jsonata_errors::{Error, Result};
use jsonata_signature_macro::signature;

use super::evaluator::Evaluator;
use super::frame::Frame;
use super::value::{ArrayFlags, Value, ValuePtr};
use crate::value;

#[derive(Clone)]
pub struct FunctionContext<'a> {
    pub name: &'a str,
    pub char_index: usize,
    pub input: ValuePtr,
    pub frame: Frame,
    pub evaluator: &'a Evaluator<'a>,
    pub arena: &'a Bump,
}

impl<'a> FunctionContext<'a> {
    pub fn evaluate_function(&self, proc: ValuePtr, args: ValuePtr) -> Result<ValuePtr> {
        self.evaluator
            .apply_function(self.char_index, self.input, proc, args, &self.frame)
    }
}

pub fn fn_lookup_internal(context: &FunctionContext, input: ValuePtr, key: &str) -> ValuePtr {
    match *input {
        Value::Array { .. } => {
            let result = Value::array(context.arena, ArrayFlags::SEQUENCE);

            for input in input.members() {
                let res = fn_lookup_internal(context, *input, key);
                match *res {
                    Value::Undefined => {}
                    Value::Array { .. } => {
                        res.members().for_each(|item| result.push(item));
                    }
                    _ => result.push(&*res),
                };
            }

            result.as_ptr()
        }
        Value::Object(..) => input.get_entry(key).as_ptr(),
        _ => value::UNDEFINED.as_ptr(),
    }
}

#[signature("<x-s:x>")]
pub fn fn_lookup(context: &FunctionContext, input: ValuePtr, key: ValuePtr) -> Result<ValuePtr> {
    if !key.is_string() {
        Err(Error::T0410ArgumentNotValid(
            context.char_index,
            1,
            context.name.to_string(),
        ))
    } else {
        Ok(fn_lookup_internal(context, input, &key.as_str()))
    }
}

// TODO: Added this to make `evaluate_unary_op` compile, probably can be factored out
pub fn fn_append_internal<'a>(
    context: &FunctionContext<'a>,
    arg1: &'a mut Value,
    arg2: ValuePtr,
) -> &'a mut Value {
    if arg2.is_undefined() {
        return arg1;
    }

    let arg1_len = if arg1.is_array() { arg1.len() } else { 1 };
    let arg2_len = if arg2.is_array() { arg2.len() } else { 1 };

    let result = Value::array_with_capacity(
        context.arena,
        arg1_len + arg2_len,
        if arg1.is_array() {
            arg1.as_ptr().get_flags()
        } else {
            ArrayFlags::SEQUENCE
        },
    );

    if arg1.is_array() {
        arg1.members().for_each(|m| result.push(&*m));
    } else {
        result.push(&*arg1);
    }

    if arg2.is_array() {
        arg2.members().for_each(|m| result.push(&*m));
    } else {
        result.push(&*arg2);
    }

    result
}

#[signature("<xx:a>")]
pub fn fn_append(context: &FunctionContext, arg1: ValuePtr, arg2: ValuePtr) -> Result<ValuePtr> {
    if arg1.is_undefined() {
        return Ok(arg2);
    }

    if arg2.is_undefined() {
        return Ok(arg1);
    }

    let arg1_len = if arg1.is_array() { arg1.len() } else { 1 };
    let arg2_len = if arg2.is_array() { arg2.len() } else { 1 };

    let result = Value::array_with_capacity(
        context.arena,
        arg1_len + arg2_len,
        if arg1.is_array() {
            arg1.get_flags()
        } else {
            ArrayFlags::SEQUENCE
        },
    );

    if arg1.is_array() {
        arg1.members().for_each(|m| result.push(&*m));
    } else {
        result.push(&*arg1);
    }

    if arg2.is_array() {
        arg2.members().for_each(|m| result.push(&*m));
    } else {
        result.push(&*arg2);
    }

    Ok(result.as_ptr())
}

#[signature("<x-:b>")]
pub fn fn_boolean(context: &FunctionContext, arg: ValuePtr) -> Result<ValuePtr> {
    Ok(match *arg {
        Value::Undefined => value::UNDEFINED.as_ptr(),
        Value::Null => Value::bool(context.arena, false).as_ptr(),
        Value::Bool(b) => Value::bool(context.arena, b).as_ptr(),
        Value::Number(num) => Value::bool(context.arena, num != 0.0).as_ptr(),
        Value::String(ref str) => Value::bool(context.arena, !str.is_empty()).as_ptr(),
        Value::Object(ref obj) => Value::bool(context.arena, !obj.is_empty()).as_ptr(),
        Value::Array { .. } => match arg.len() {
            0 => Value::bool(context.arena, false).as_ptr(),
            1 => fn_boolean(context, arg.get_member(0).as_ptr())?,
            _ => {
                for item in arg.members() {
                    if fn_boolean(context, *item)?.as_bool() {
                        return Ok(Value::bool(context.arena, true).as_ptr());
                    }
                }
                Value::bool(context.arena, false).as_ptr()
            }
        },
        Value::Lambda { .. }
        | Value::NativeFn0 { .. }
        | Value::NativeFn1 { .. }
        | Value::NativeFn2 { .. }
        | Value::NativeFn3 { .. } => Value::bool(context.arena, false).as_ptr(),
    })
}

#[signature("<af>")]
pub fn fn_filter(context: &FunctionContext, arr: ValuePtr, func: ValuePtr) -> Result<ValuePtr> {
    if arr.is_undefined() {
        return Ok(value::UNDEFINED.as_ptr());
    }

    let arr = Value::wrap_in_array_if_needed(context.arena, &*arr, ArrayFlags::empty());

    if !func.is_function() {
        return Err(Error::T0410ArgumentNotValid(
            context.char_index,
            2,
            context.name.to_string(),
        ));
    }

    let result = Value::array(context.arena, ArrayFlags::SEQUENCE);

    for (index, item) in arr.members().enumerate() {
        let args = Value::array(context.arena, ArrayFlags::empty());
        let arity = func.arity();

        args.push(item);
        if arity >= 2 {
            args.push(Value::number(context.arena, index));
        }
        if arity >= 3 {
            args.push(&*arr);
        }

        let include = context.evaluate_function(func, args.as_ptr())?;

        if include.is_truthy() {
            result.push(item);
        }
    }

    Ok(result.as_ptr())
}

#[signature("<x-b?:s>")]
pub fn fn_string(context: &FunctionContext, arg: ValuePtr) -> Result<ValuePtr> {
    if arg.is_undefined() {
        return Ok(value::UNDEFINED.as_ptr());
    }

    if arg.is_string() {
        Ok(arg)
    } else if arg.is_function() {
        Ok(Value::string(context.arena, String::from("")).as_ptr())

    // TODO: Check for infinite numbers
    // } else if arg.is_number() && arg.is_infinite() {
    //     // TODO: D3001
    //     unreachable!()

    // TODO: pretty printing
    } else {
        Ok(Value::string(context.arena, arg.dump()).as_ptr())
    }
}

#[signature("<a:n>")]
pub fn fn_count(context: &FunctionContext, arg: ValuePtr) -> Result<ValuePtr> {
    Ok(Value::number(
        context.arena,
        if arg.is_undefined() {
            0
        } else if arg.is_array() {
            arg.len()
        } else {
            1
        },
    )
    .as_ptr())
}

#[signature("<x-:b>")]
pub fn fn_not(context: &FunctionContext, arg: ValuePtr) -> Result<ValuePtr> {
    Ok(if arg.is_undefined() {
        value::UNDEFINED.as_ptr()
    } else {
        Value::bool(context.arena, !arg.is_truthy()).as_ptr()
    })
}

#[signature("<s-:s>")]
pub fn fn_lowercase(context: &FunctionContext, arg: ValuePtr) -> Result<ValuePtr> {
    Ok(if !arg.is_string() {
        value::UNDEFINED.as_ptr()
    } else {
        Value::string(context.arena, arg.as_str().to_lowercase()).as_ptr()
    })
}

#[signature("<s-:s>")]
pub fn fn_uppercase(context: &FunctionContext, arg: ValuePtr) -> Result<ValuePtr> {
    if !arg.is_string() {
        Ok(value::UNDEFINED.as_ptr())
    } else {
        Ok(Value::string(context.arena, arg.as_str().to_uppercase()).as_ptr())
    }
}

#[signature("<s-nn?:s>")]
pub fn fn_substring(
    context: &FunctionContext,
    string: ValuePtr,
    start: ValuePtr,
    length: ValuePtr,
) -> Result<ValuePtr> {
    if string.is_undefined() {
        return Ok(value::UNDEFINED.as_ptr());
    }

    if !string.is_string() {
        return Err(Error::T0410ArgumentNotValid(
            context.char_index,
            1,
            context.name.to_string(),
        ));
    }

    if !start.is_number() {
        return Err(Error::T0410ArgumentNotValid(
            context.char_index,
            2,
            context.name.to_string(),
        ));
    }

    let string = string.as_str();

    // Scan the string chars for the actual number of characters.
    // NOTE: Chars are not grapheme clusters, so for some inputs like "नमस्ते" we will get 6
    //       as it will include the diacritics.
    //       See: https://doc.rust-lang.org/nightly/book/ch08-02-strings.html
    let len = string.chars().count() as isize;
    let mut start = start.as_isize();

    // If start is negative and runs off the front of the string
    if len + start < 0 {
        start = 0;
    }

    // If start is negative, count from the end of the string
    let start = if start < 0 { len + start } else { start };

    if length.is_undefined() {
        Ok(Value::string(context.arena, string[start as usize..].to_string()).as_ptr())
    } else {
        if !length.is_number() {
            return Err(Error::T0410ArgumentNotValid(
                context.char_index,
                3,
                context.name.to_string(),
            ));
        }

        let length = length.as_isize();
        if length < 0 {
            Ok(Value::string(context.arena, String::from("")).as_ptr())
        } else {
            let end = if start >= 0 {
                (start + length) as usize
            } else {
                (len + start + length) as usize
            };

            let substring = string
                .chars()
                .skip(start as usize)
                .take(end - start as usize)
                .collect::<String>();

            Ok(Value::string(context.arena, substring).as_ptr())
        }
    }
}

#[signature("<n-:n>")]
pub fn fn_abs(context: &FunctionContext, arg: ValuePtr) -> Result<ValuePtr> {
    if arg.is_undefined() {
        Ok(value::UNDEFINED.as_ptr())
    } else if !arg.is_number() {
        Err(Error::T0410ArgumentNotValid(
            context.char_index,
            1,
            context.name.to_string(),
        ))
    } else {
        Ok(Value::number(context.arena, arg.as_f64().abs()).as_ptr())
    }
}

#[signature("<n-:n>")]
pub fn fn_floor(context: &FunctionContext, arg: ValuePtr) -> Result<ValuePtr> {
    if arg.is_undefined() {
        Ok(value::UNDEFINED.as_ptr())
    } else if !arg.is_number() {
        Err(Error::T0410ArgumentNotValid(
            context.char_index,
            1,
            context.name.to_string(),
        ))
    } else {
        Ok(Value::number(context.arena, arg.as_f64().floor()).as_ptr())
    }
}

#[signature("<n-:n>")]
pub fn fn_ceil(context: &FunctionContext, arg: ValuePtr) -> Result<ValuePtr> {
    if arg.is_undefined() {
        Ok(value::UNDEFINED.as_ptr())
    } else if !arg.is_number() {
        Err(Error::T0410ArgumentNotValid(
            context.char_index,
            1,
            context.name.to_string(),
        ))
    } else {
        Ok(Value::number(context.arena, arg.as_f64().ceil()).as_ptr())
    }
}

#[signature("<a<n>:n>")]
pub fn fn_max(context: &FunctionContext, args: ValuePtr) -> Result<ValuePtr> {
    if args.is_undefined() || (args.is_array() && args.is_empty()) {
        return Ok(value::UNDEFINED.as_ptr());
    }
    let args = Value::wrap_in_array_if_needed(context.arena, &*args, ArrayFlags::empty());
    let mut max = f64::MIN;
    for arg in args.members() {
        if !arg.is_number() {
            return Err(Error::T0412ArgumentMustBeArrayOfType(
                context.char_index,
                2,
                context.name.to_string(),
                "number".to_string(),
            ));
        }
        max = f64::max(max, arg.as_f64());
    }
    Ok(Value::number(context.arena, max).as_ptr())
}

#[signature("<a<n>:n>")]
pub fn fn_min(context: &FunctionContext, args: ValuePtr) -> Result<ValuePtr> {
    if args.is_undefined() || (args.is_array() && args.is_empty()) {
        return Ok(value::UNDEFINED.as_ptr());
    }
    let args = Value::wrap_in_array_if_needed(context.arena, &*args, ArrayFlags::empty());
    let mut min = f64::MAX;
    for arg in args.members() {
        if !arg.is_number() {
            return Err(Error::T0412ArgumentMustBeArrayOfType(
                context.char_index,
                2,
                context.name.to_string(),
                "number".to_string(),
            ));
        }
        min = f64::min(min, arg.as_f64());
    }
    Ok(Value::number(context.arena, min).as_ptr())
}

#[signature("<a<n>:n>")]
pub fn fn_sum(context: &FunctionContext, args: ValuePtr) -> Result<ValuePtr> {
    if args.is_undefined() || (args.is_array() && args.is_empty()) {
        return Ok(value::UNDEFINED.as_ptr());
    }
    let args = Value::wrap_in_array_if_needed(context.arena, &*args, ArrayFlags::empty());
    let mut sum = 0.0;
    for arg in args.members() {
        if !arg.is_number() {
            return Err(Error::T0412ArgumentMustBeArrayOfType(
                context.char_index,
                2,
                context.name.to_string(),
                "number".to_string(),
            ));
        }
        sum += arg.as_f64();
    }
    Ok(Value::number(context.arena, sum).as_ptr())
}
