use lazy_static;

use jsonata_errors::{Error, Result};
use jsonata_signature_macro::signature;

use super::evaluator::Evaluator;
use super::frame::Frame;
use super::value::{ArrayFlags, Value, ValueArena, ValueKind};

#[derive(Clone)]
pub struct FunctionContext<'a> {
    pub name: &'a str,
    pub char_index: usize,
    pub arena: ValueArena,
    pub input: Value,
    pub frame: Frame,
    pub evaluator: &'a Evaluator,
}

impl<'a> FunctionContext<'a> {
    pub fn evaluate_function(&self, proc: &Value, args: &Value) -> Result<Value> {
        self.evaluator
            .apply_function(self.char_index, &self.input, proc, args, &self.frame)
    }
}

pub fn fn_lookup_internal(context: &FunctionContext, input: &Value, key: &str) -> Value {
    match **input {
        ValueKind::Array { .. } => {
            let mut result = context.arena.array(ArrayFlags::SEQUENCE);

            for input in input.members() {
                let res = fn_lookup_internal(context, &input, key);
                match *res {
                    ValueKind::Undefined => {}
                    ValueKind::Array { .. } => {
                        res.members().for_each(|item| result.push(&item));
                    }
                    _ => result.push(&res),
                };
            }

            result
        }
        ValueKind::Object(..) => input.get_entry(key),
        _ => context.arena.undefined(),
    }
}

#[signature("<x-s:x>")]
pub fn fn_lookup(context: &FunctionContext, input: &Value, key: &Value) -> Result<Value> {
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

#[signature("<xx:a>")]
pub fn fn_append(context: &FunctionContext, arg1: &Value, arg2: &Value) -> Result<Value> {
    if arg1.is_undefined() {
        return Ok(arg2.clone());
    }

    if arg2.is_undefined() {
        return Ok(arg1.clone());
    }

    let result = context.arena.value((**arg1).clone());
    let mut result = result.wrap_in_array_if_needed(ArrayFlags::SEQUENCE);
    let arg2 = arg2.wrap_in_array_if_needed(ArrayFlags::empty());
    arg2.members().for_each(|m| result.push(&m));

    Ok(result)
}

#[signature("<x-:b>")]
pub fn fn_boolean(context: &FunctionContext, arg: &Value) -> Result<Value> {
    Ok(match **arg {
        ValueKind::Undefined => context.arena.undefined(),
        ValueKind::Null => context.arena.bool(false),
        ValueKind::Bool(b) => context.arena.bool(b),
        ValueKind::Number(num) => context.arena.bool(num != 0.0),
        ValueKind::String(ref str) => context.arena.bool(!str.is_empty()),
        ValueKind::Object(ref obj) => context.arena.bool(!obj.is_empty()),
        ValueKind::Array { .. } => match arg.len() {
            0 => context.arena.bool(false),
            1 => fn_boolean(context, &arg.get_member(0))?,
            _ => {
                for item in arg.members() {
                    if fn_boolean(context, &item)?.as_bool() {
                        return Ok(context.arena.bool(true));
                    }
                }
                context.arena.bool(false)
            }
        },
        ValueKind::Lambda { .. }
        | ValueKind::NativeFn0 { .. }
        | ValueKind::NativeFn1 { .. }
        | ValueKind::NativeFn2 { .. }
        | ValueKind::NativeFn3 { .. } => context.arena.bool(false),
    })
}

#[signature("<af>")]
pub fn fn_filter(context: &FunctionContext, arr: &Value, func: &Value) -> Result<Value> {
    if arr.is_undefined() {
        return Ok(context.arena.undefined());
    }

    let arr = arr.wrap_in_array_if_needed(ArrayFlags::empty());

    if !func.is_function() {
        return Err(Error::T0410ArgumentNotValid(
            context.char_index,
            2,
            context.name.to_string(),
        ));
    }

    let mut result = context.arena.array(ArrayFlags::SEQUENCE);

    for (index, item) in arr.members().enumerate() {
        let mut args = context.arena.array(ArrayFlags::empty());
        let arity = func.arity();

        args.push(&item);
        if arity >= 2 {
            args.push_new(ValueKind::Number(index.into()));
        }
        if arity >= 3 {
            args.push(&arr);
        }

        let include = context.evaluate_function(func, &args)?;

        if include.is_truthy() {
            result.push(&item);
        }
    }

    Ok(result)
}

#[signature("<x-b?:s>")]
pub fn fn_string(context: &FunctionContext, arg: &Value) -> Result<Value> {
    if arg.is_undefined() {
        return Ok(context.arena.undefined());
    }

    if arg.is_string() {
        Ok(arg.clone())
    } else if arg.is_function() {
        Ok(context.arena.string(String::from("")))

    // TODO: Check for infinite numbers
    // } else if arg.is_number() && arg.is_infinite() {
    //     // TODO: D3001
    //     unreachable!()

    // TODO: pretty printing
    } else {
        Ok(context.arena.string(arg.dump()))
    }
}

#[signature("<a:n>")]
pub fn fn_count(context: &FunctionContext, arg: &Value) -> Result<Value> {
    Ok(context.arena.number(if arg.is_undefined() {
        0
    } else if arg.is_array() {
        arg.len()
    } else {
        1
    }))
}

#[signature("<x-:b>")]
pub fn fn_not(context: &FunctionContext, arg: &Value) -> Result<Value> {
    Ok(if arg.is_undefined() {
        context.arena.undefined()
    } else {
        context.arena.bool(!arg.is_truthy())
    })
}

#[signature("<s-:s>")]
pub fn fn_lowercase(context: &FunctionContext, arg: &Value) -> Result<Value> {
    Ok(if !arg.is_string() {
        context.arena.undefined()
    } else {
        context.arena.string(arg.as_str().to_lowercase())
    })
}

#[signature("<s-:s>")]
pub fn fn_uppercase(context: &FunctionContext, arg: &Value) -> Result<Value> {
    if !arg.is_string() {
        Ok(context.arena.undefined())
    } else {
        Ok(context.arena.string(arg.as_str().to_uppercase()))
    }
}

#[signature("<s-nn?:s>")]
pub fn fn_substring(
    context: &FunctionContext,
    string: &Value,
    start: &Value,
    length: &Value,
) -> Result<Value> {
    if string.is_undefined() {
        return Ok(context.arena.undefined());
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
        Ok(context.arena.string(string[start as usize..].to_string()))
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
            Ok(context.arena.string(String::from("")))
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

            Ok(context.arena.string(substring))
        }
    }
}

#[signature("<n-:n>")]
pub fn fn_abs(context: &FunctionContext, arg: &Value) -> Result<Value> {
    if arg.is_undefined() {
        Ok(context.arena.undefined())
    } else if !arg.is_number() {
        Err(Error::T0410ArgumentNotValid(
            context.char_index,
            1,
            context.name.to_string(),
        ))
    } else {
        Ok(context.arena.number(arg.as_f64().abs()))
    }
}

#[signature("<n-:n>")]
pub fn fn_floor(context: &FunctionContext, arg: &Value) -> Result<Value> {
    if arg.is_undefined() {
        Ok(context.arena.undefined())
    } else if !arg.is_number() {
        Err(Error::T0410ArgumentNotValid(
            context.char_index,
            1,
            context.name.to_string(),
        ))
    } else {
        Ok(context.arena.number(arg.as_f64().floor()))
    }
}

#[signature("<n-:n>")]
pub fn fn_ceil(context: &FunctionContext, arg: &Value) -> Result<Value> {
    if arg.is_undefined() {
        Ok(context.arena.undefined())
    } else if !arg.is_number() {
        Err(Error::T0410ArgumentNotValid(
            context.char_index,
            1,
            context.name.to_string(),
        ))
    } else {
        Ok(context.arena.number(arg.as_f64().ceil()))
    }
}

#[signature("<a<n>:n>")]
pub fn fn_max(context: &FunctionContext, args: &Value) -> Result<Value> {
    if args.is_undefined() || (args.is_array() && args.is_empty()) {
        return Ok(context.arena.undefined());
    }
    let args = args.wrap_in_array_if_needed(ArrayFlags::empty());
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
    Ok(context.arena.number(max))
}

#[signature("<a<n>:n>")]
pub fn fn_min(context: &FunctionContext, args: &Value) -> Result<Value> {
    if args.is_undefined() || (args.is_array() && args.is_empty()) {
        return Ok(context.arena.undefined());
    }
    let args = args.wrap_in_array_if_needed(ArrayFlags::empty());
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
    Ok(context.arena.number(min))
}

#[signature("<a<n>:n>")]
pub fn fn_sum(context: &FunctionContext, args: &Value) -> Result<Value> {
    if args.is_undefined() || (args.is_array() && args.is_empty()) {
        return Ok(context.arena.undefined());
    }
    let args = args.wrap_in_array_if_needed(ArrayFlags::empty());
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
    Ok(context.arena.number(sum))
}
