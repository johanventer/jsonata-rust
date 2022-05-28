use bumpalo::Bump;

use jsonata_errors::{Error, Result};

use super::evaluator::Evaluator;
use super::frame::Frame;
use super::value::serialize::{DumpFormatter, PrettyFormatter, Serializer};
use super::value::{ArrayFlags, Value};

// macro_rules! min_args {
//     ($context:ident, $args:ident, $min:literal) => {
//         if $args.len() < $min {
//             return Err(Error::T0410ArgumentNotValid(
//                 $context.char_index,
//                 $min,
//                 $context.name.to_string(),
//             ));
//         }
//     };
// }

macro_rules! max_args {
    ($context:ident, $args:ident, $max:literal) => {
        if $args.len() > $max {
            return Err(Error::T0410ArgumentNotValid(
                $context.char_index,
                $max,
                $context.name.to_string(),
            ));
        }
    };
}

macro_rules! bad_arg {
    ($context:ident, $index:literal) => {
        return Err(Error::T0410ArgumentNotValid(
            $context.char_index,
            $index,
            $context.name.to_string(),
        ))
    };
}

macro_rules! assert_arg {
    ($condition: expr, $context:ident, $index:literal) => {
        if !($condition) {
            bad_arg!($context, $index);
        }
    };
}

macro_rules! assert_array_of_type {
    ($condition:expr, $context:ident, $index:literal, $t:literal) => {
        if !($condition) {
            return Err(Error::T0412ArgumentMustBeArrayOfType(
                $context.char_index,
                $index,
                $context.name.to_string(),
                $t.to_string(),
            ));
        };
    };
}

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

// Version of append that takes a mutable arg1 - this could probably be collapsed
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

pub fn fn_append<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    args: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    let arg1 = &args[0];
    let arg2 = &args[1];

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

pub fn fn_boolean<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    args: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    max_args!(context, args, 1);

    let arg = &args[0];
    Ok(match arg {
        Value::Undefined => Value::undefined(),
        Value::Null => Value::bool(context.arena, false),
        Value::Bool(b) => Value::bool(context.arena, *b),
        Value::Number(n) => {
            arg.is_valid_number()?;
            Value::bool(context.arena, *n != 0.0)
        }
        Value::String(ref str) => Value::bool(context.arena, !str.is_empty()),
        Value::Object(ref obj) => Value::bool(context.arena, !obj.is_empty()),
        Value::Array { .. } => match arg.len() {
            0 => Value::bool(context.arena, false),
            1 => fn_boolean(
                context.clone(),
                Value::wrap_in_array(context.arena, arg.get_member(0), ArrayFlags::empty()),
            )?,
            _ => {
                for item in arg.members() {
                    if fn_boolean(
                        context.clone(),
                        Value::wrap_in_array(context.arena, item, ArrayFlags::empty()),
                    )?
                    .as_bool()
                    {
                        return Ok(Value::bool(context.arena, true));
                    }
                }
                Value::bool(context.arena, false)
            }
        },
        Value::Lambda { .. } | Value::NativeFn { .. } => Value::bool(context.arena, false),
    })
}

pub fn fn_filter<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    args: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    let arr = &args[0];
    let func = &args[1];

    if arr.is_undefined() {
        return Ok(Value::undefined());
    }

    let arr = Value::wrap_in_array_if_needed(context.arena, arr, ArrayFlags::empty());

    assert_arg!(func.is_function(), context, 2);

    let result = Value::array(context.arena, ArrayFlags::SEQUENCE);

    for (index, item) in arr.members().enumerate() {
        let args = Value::array(context.arena, ArrayFlags::empty());
        let arity = func.arity();

        args.push(item);
        if arity >= 2 {
            args.push(Value::number(context.arena, index as f64));
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

pub fn fn_string<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    args: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    max_args!(context, args, 2);

    let input = if args.is_empty() {
        if context.input.is_array() && context.input.has_flags(ArrayFlags::WRAPPED) {
            &context.input[0]
        } else {
            context.input
        }
    } else {
        &args[0]
    };

    if input.is_undefined() {
        return Ok(Value::undefined());
    }

    let pretty = &args[1];
    assert_arg!(pretty.is_undefined() || pretty.is_bool(), context, 2);

    if input.is_string() {
        Ok(input)
    } else if input.is_function() {
        Ok(Value::string(context.arena, String::from("")))
    } else if input.is_number() && !input.is_finite() {
        Err(Error::D3001StringNotFinite(context.char_index))
    } else if *pretty == true {
        let serializer = Serializer::new(PrettyFormatter::default(), true);
        let output = serializer.serialize(input)?;
        Ok(Value::string(context.arena, output))
    } else {
        let serializer = Serializer::new(DumpFormatter, true);
        let output = serializer.serialize(input)?;
        Ok(Value::string(context.arena, output))
    }
}

pub fn fn_not<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    args: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    let arg = &args[0];

    Ok(if arg.is_undefined() {
        Value::undefined()
    } else {
        Value::bool(context.arena, !arg.is_truthy())
    })
}

pub fn fn_lowercase<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    args: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    let arg = &args[0];

    Ok(if !arg.is_string() {
        Value::undefined()
    } else {
        Value::string(context.arena, arg.as_str().to_lowercase())
    })
}

pub fn fn_uppercase<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    args: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    let arg = &args[0];

    if !arg.is_string() {
        Ok(Value::undefined())
    } else {
        Ok(Value::string(context.arena, arg.as_str().to_uppercase()))
    }
}

pub fn fn_substring<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    args: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    let string = &args[0];
    let start = &args[1];
    let length = &args[2];

    if string.is_undefined() {
        return Ok(Value::undefined());
    }

    assert_arg!(string.is_string(), context, 1);
    assert_arg!(start.is_number(), context, 2);

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
        assert_arg!(length.is_number(), context, 3);

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

pub fn fn_abs<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    args: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    let arg = &args[0];

    if arg.is_undefined() {
        return Ok(Value::undefined());
    }

    assert_arg!(arg.is_number(), context, 1);

    Ok(Value::number(context.arena, arg.as_f64().abs()))
}

pub fn fn_floor<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    args: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    let arg = &args[0];

    if arg.is_undefined() {
        return Ok(Value::undefined());
    }

    assert_arg!(arg.is_number(), context, 1);

    Ok(Value::number(context.arena, arg.as_f64().floor()))
}

pub fn fn_ceil<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    args: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    let arg = &args[0];

    if arg.is_undefined() {
        return Ok(Value::undefined());
    }

    assert_arg!(arg.is_number(), context, 1);

    Ok(Value::number(context.arena, arg.as_f64().ceil()))
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

pub fn fn_lookup<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    args: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    let input = &args[0];
    let key = &args[1];
    assert_arg!(key.is_string(), context, 2);
    Ok(fn_lookup_internal(context.clone(), input, &key.as_str()))
}

pub fn fn_count<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    args: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    max_args!(context, args, 1);

    let arg = &args[0];

    Ok(Value::number(
        context.arena,
        if arg.is_undefined() {
            0.0
        } else if arg.is_array() {
            arg.len() as f64
        } else {
            1.0
        },
    ))
}

pub fn fn_max<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    args: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    max_args!(context, args, 1);

    let arg = &args[0];

    // $max(undefined) and $max([]) return undefined
    if arg.is_undefined() || (arg.is_array() && arg.is_empty()) {
        return Ok(Value::undefined());
    }

    let arr = Value::wrap_in_array_if_needed(context.arena, arg, ArrayFlags::empty());

    let mut max = f64::MIN;

    for member in arr.members() {
        assert_array_of_type!(member.is_number(), context, 1, "number");
        max = f64::max(max, member.as_f64());
    }
    Ok(Value::number(context.arena, max))
}

pub fn fn_min<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    args: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    max_args!(context, args, 1);

    let arg = &args[0];

    // $min(undefined) and $min([]) return undefined
    if arg.is_undefined() || (arg.is_array() && arg.is_empty()) {
        return Ok(Value::undefined());
    }

    let arr = Value::wrap_in_array_if_needed(context.arena, arg, ArrayFlags::empty());

    let mut min = f64::MAX;

    for member in arr.members() {
        assert_array_of_type!(member.is_number(), context, 1, "number");
        min = f64::min(min, member.as_f64());
    }
    Ok(Value::number(context.arena, min))
}

pub fn fn_sum<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    args: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    max_args!(context, args, 1);

    let arg = &args[0];

    // $sum(undefined) returns undefined
    if arg.is_undefined() {
        return Ok(Value::undefined());
    }

    let arr = Value::wrap_in_array_if_needed(context.arena, arg, ArrayFlags::empty());

    let mut sum = 0.0;

    for member in arr.members() {
        assert_array_of_type!(member.is_number(), context, 1, "number");
        sum += member.as_f64();
    }
    Ok(Value::number(context.arena, sum))
}

pub fn fn_number<'a, 'e>(
    context: FunctionContext<'a, 'e>,
    args: &'a Value<'a>,
) -> Result<&'a Value<'a>> {
    max_args!(context, args, 1);

    let arg = &args[0];

    match arg {
        Value::Undefined => Ok(Value::undefined()),
        Value::Number(..) => Ok(arg),
        Value::Bool(true) => Ok(Value::number(context.arena, 1)),
        Value::Bool(false) => Ok(Value::number(context.arena, 0)),
        Value::String(s) => {
            let result: f64 = s
                .parse()
                .map_err(|_e| Error::D3030NonNumericCast(context.char_index, arg.to_string()))?;

            if !result.is_nan() && !result.is_infinite() {
                Ok(Value::number(context.arena, result))
            } else {
                Ok(Value::undefined())
            }
        }
        _ => bad_arg!(context, 1),
    }
}
