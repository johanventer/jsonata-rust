use bumpalo::Bump;
use lazy_static;

use jsonata_errors::{Error, Result};
use jsonata_signature_macro::signature;

use super::evaluator::Evaluator;
use super::frame::Frame;
use super::value::{ArrayFlags, Value};

#[derive(Clone)]
pub struct FunctionContext<'a, 'e> {
    pub name: &'a str,
    pub char_index: usize,
    pub input: &'a Value<'a>,
    pub frame: Frame<'a>,
    pub evaluator: &'e Evaluator<'a>,
    pub arena: &'a Bump,
}

impl<'a, 'e> FunctionContext<'a, 'e> {
    pub fn evaluate_function(
        &self,
        proc: &'a Value<'a>,
        args: &'a Value<'a>,
    ) -> Result<&'a Value<'a>> {
        self.evaluator
            .apply_function(self.char_index, self.input, proc, args, &self.frame)
    }
}

pub fn fn_lookup_internal<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    input: &'a Value<'a>,
    key: &str,
) -> &'a Value<'a> {
    match input {
        Value::Array { .. } => {
            let result = Value::array(context.arena, ArrayFlags::SEQUENCE);

            for input in input.members() {
                let res = fn_lookup_internal(context.clone(), input, key);
                match res {
                    Value::Undefined => {}
                    Value::Array { .. } => {
                        res.members().for_each(|item| result.push(item));
                    }
                    _ => result.push(res),
                };
            }

            result
        }
        Value::Object(..) => input.get_entry(key),
        _ => Value::undefined(),
    }
}

#[signature("<x-s:x>")]
pub fn fn_lookup<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    input: &'a Value<'a>,
    key: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    if !key.is_string() {
        Err(Error::T0410ArgumentNotValid(
            context.char_index,
            1,
            context.name.to_string(),
        ))
    } else {
        Ok(fn_lookup_internal(context.clone(), input, &key.as_str()))
    }
}

// TODO: Added this to make `evaluate_unary_op` compile, probably can be factored out
pub fn fn_append_internal<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    arg1: &'a mut Value<'a>,
    arg2: &'a Value<'a>,
) -> &'a mut Value<'a> {
    if arg2.is_undefined() {
        return arg1;
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
        arg1.members().for_each(|m| result.push(m));
    } else {
        result.push(&*arg1);
    }

    if arg2.is_array() {
        arg2.members().for_each(|m| result.push(m));
    } else {
        result.push(arg2);
    }

    result
}

#[signature("<xx:a>")]
pub fn fn_append<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    arg1: &'a Value<'a>,
    arg2: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
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
        arg1.members().for_each(|m| result.push(m));
    } else {
        result.push(arg1);
    }

    if arg2.is_array() {
        arg2.members().for_each(|m| result.push(m));
    } else {
        result.push(arg2)
    }

    Ok(result)
}

#[signature("<x-:b>")]
pub fn fn_boolean<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    arg: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    Ok(match arg {
        Value::Undefined => Value::undefined(),
        Value::Null => Value::bool(context.arena, false),
        Value::Bool(b) => Value::bool(context.arena, *b),
        Value::Unsigned(n) => Value::bool(context.arena, *n != 0),
        Value::Signed(n) => Value::bool(context.arena, *n != 0),
        Value::Float(n) => Value::bool(context.arena, *n != 0.0),
        Value::String(ref str) => Value::bool(context.arena, !str.is_empty()),
        Value::Object(ref obj) => Value::bool(context.arena, !obj.is_empty()),
        Value::Array { .. } => match arg.len() {
            0 => Value::bool(context.arena, false),
            1 => fn_boolean(context.clone(), arg.get_member(0))?,
            _ => {
                for item in arg.members() {
                    if fn_boolean(context.clone(), item)?.as_bool() {
                        return Ok(Value::bool(context.arena, true));
                    }
                }
                Value::bool(context.arena, false)
            }
        },
        Value::Lambda { .. }
        | Value::NativeFn0 { .. }
        | Value::NativeFn1 { .. }
        | Value::NativeFn2 { .. }
        | Value::NativeFn3 { .. } => Value::bool(context.arena, false),
    })
}

#[signature("<af>")]
pub fn fn_filter<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    arr: &'a Value<'a>,
    func: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    if arr.is_undefined() {
        return Ok(Value::undefined());
    }

    let arr = Value::wrap_in_array_if_needed(context.arena, arr, ArrayFlags::empty());

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
            args.push(Value::unsigned(context.arena, index as u64));
        }
        if arity >= 3 {
            args.push(&*arr);
        }

        let include = context.evaluate_function(func, args)?;

        if include.is_truthy() {
            result.push(item);
        }
    }

    Ok(result)
}

#[signature("<x-b?:s>")]
pub fn fn_string<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    arg: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    if arg.is_undefined() {
        return Ok(Value::undefined());
    }

    if arg.is_string() {
        Ok(arg)
    } else if arg.is_function() {
        Ok(Value::string(context.arena, String::from("")))

    // TODO: Check for infinite numbers
    // } else if arg.is_number() && arg.is_infinite() {
    //     // TODO: D3001
    //     unreachable!()

    // TODO: pretty printing
    } else {
        Ok(Value::string(context.arena, arg.dump()))
    }
}

#[signature("<a:n>")]
pub fn fn_count<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    arg: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    Ok(Value::unsigned(
        context.arena,
        if arg.is_undefined() {
            0
        } else if arg.is_array() {
            arg.len() as u64
        } else {
            1
        },
    ))
}

#[signature("<x-:b>")]
pub fn fn_not<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    arg: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    Ok(if arg.is_undefined() {
        Value::undefined()
    } else {
        Value::bool(context.arena, !arg.is_truthy())
    })
}

#[signature("<s-:s>")]
pub fn fn_lowercase<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    arg: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    Ok(if !arg.is_string() {
        Value::undefined()
    } else {
        Value::string(context.arena, arg.as_str().to_lowercase())
    })
}

#[signature("<s-:s>")]
pub fn fn_uppercase<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    arg: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    if !arg.is_string() {
        Ok(Value::undefined())
    } else {
        Ok(Value::string(context.arena, arg.as_str().to_uppercase()))
    }
}

#[signature("<s-nn?:s>")]
pub fn fn_substring<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    string: &'a Value<'a>,
    start: &'a Value<'a>,
    length: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    if string.is_undefined() {
        return Ok(Value::undefined());
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
        Ok(Value::string(
            context.arena,
            string[start as usize..].to_string(),
        ))
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
            Ok(Value::string(context.arena, String::from("")))
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

            Ok(Value::string(context.arena, substring))
        }
    }
}

#[signature("<n-:n>")]
pub fn fn_abs<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    arg: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    if arg.is_undefined() {
        Ok(Value::undefined())
    } else if !arg.is_number() {
        Err(Error::T0410ArgumentNotValid(
            context.char_index,
            1,
            context.name.to_string(),
        ))
    } else {
        Ok(Value::float(context.arena, arg.as_f64().abs()))
    }
}

#[signature("<n-:n>")]
pub fn fn_floor<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    arg: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    if arg.is_undefined() {
        Ok(Value::undefined())
    } else if !arg.is_number() {
        Err(Error::T0410ArgumentNotValid(
            context.char_index,
            1,
            context.name.to_string(),
        ))
    } else {
        Ok(Value::float(context.arena, arg.as_f64().floor()))
    }
}

#[signature("<n-:n>")]
pub fn fn_ceil<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    arg: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    if arg.is_undefined() {
        Ok(Value::undefined())
    } else if !arg.is_number() {
        Err(Error::T0410ArgumentNotValid(
            context.char_index,
            1,
            context.name.to_string(),
        ))
    } else {
        Ok(Value::float(context.arena, arg.as_f64().ceil()))
    }
}

#[signature("<a<n>:n>")]
pub fn fn_max<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    args: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    if args.is_undefined() || (args.is_array() && args.is_empty()) {
        return Ok(Value::undefined());
    }
    let args = Value::wrap_in_array_if_needed(context.arena, args, ArrayFlags::empty());
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
    Ok(Value::float(context.arena, max))
}

#[signature("<a<n>:n>")]
pub fn fn_min<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    args: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    if args.is_undefined() || (args.is_array() && args.is_empty()) {
        return Ok(Value::undefined());
    }
    let args = Value::wrap_in_array_if_needed(context.arena, args, ArrayFlags::empty());
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
    Ok(Value::float(context.arena, min))
}

#[signature("<a<n>:n>")]
pub fn fn_sum<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    args: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    if args.is_undefined() || (args.is_array() && args.is_empty()) {
        return Ok(Value::undefined());
    }
    let args = Value::wrap_in_array_if_needed(context.arena, args, ArrayFlags::empty());
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
    Ok(Value::float(context.arena, sum))
}
