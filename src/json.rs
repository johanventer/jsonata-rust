// An absolutely, terrible, no-good JSON parser.
// This is completely naive, probably broken in more ways than I already know, and slow as molasses.
// See an actually good JSON parser in Rust here: https://github.com/maciejhirsz/json-rust/blob/master/src/parser.rs
// TODO: Replace this with an actually good parser.

use crate::evaluator::Value;

const TRUE: &str = "true";
const FALSE: &str = "false";
const NULL: &str = "null";

fn is_space(c: char) -> bool {
    c.is_whitespace() || c == '\t' || c == '\n' || c == '\r'
}

fn skip_whitespace(src: &[char], index: &mut usize) {
    while src.len() > *index && is_space(src[*index]) {
        *index += 1;
    }
}

fn check_eof(src: &[char], index: &mut usize) -> Option<()> {
    if *index >= src.len() {
        return None;
    }
    Some(())
}

fn expect(src: &[char], index: &mut usize, c: char) -> Option<()> {
    if src[*index] != c {
        return None;
    }
    *index += 1;
    check_eof(src, index)
}

fn _parse(src: &[char], index: &mut usize) -> Option<Value> {
    skip_whitespace(src, index);
    check_eof(src, index)?;
    match src[*index] {
        '{' => parse_object(src, index),
        '[' => parse_array(src, index),
        't' => parse_true(src, index),
        'f' => parse_false(src, index),
        'n' => parse_null(src, index),
        '"' => parse_string(src, index).map(Value::String),
        '-' => parse_number(src, index),
        _ => {
            if src[*index].is_ascii_digit() {
                parse_number(src, index)
            } else {
                None
            }
        }
    }
}

pub(crate) fn parse(src: &str) -> Option<Value> {
    let mut index: usize = 0;
    let src_chars: Vec<char> = src.chars().collect();
    let result = _parse(&src_chars, &mut index);
    skip_whitespace(&src_chars, &mut index);
    if index != src_chars.len() {
        return None;
    }
    result
}

fn parse_object(src: &[char], index: &mut usize) -> Option<Value> {
    expect(src, index, '{')?;
    let mut result = Value::new_object();
    while src.len() > *index {
        skip_whitespace(src, index);
        check_eof(src, index)?;
        if src[*index] == '}' {
            *index += 1;
            return Some(result);
        }
        let key = parse_string(src, index)?;
        skip_whitespace(src, index);
        check_eof(src, index)?;
        expect(src, index, ':')?;
        skip_whitespace(src, index);
        check_eof(src, index)?;
        let value = _parse(src, index)?;
        result.insert(key, value);
        skip_whitespace(src, index);
        check_eof(src, index)?;
        match src[*index] {
            ',' => *index += 1,
            '}' => {
                *index += 1;
                return Some(result);
            }
            _ => return None,
        }
    }
    None
}

fn parse_array(src: &[char], index: &mut usize) -> Option<Value> {
    expect(src, index, '[')?;
    let mut result = Value::new_array();
    while src.len() > *index {
        skip_whitespace(src, index);
        check_eof(src, index)?;
        if src[*index] == ']' {
            *index += 1;
            return Some(result);
        }
        let item = _parse(src, index)?;
        result.push(item);
        skip_whitespace(src, index);
        check_eof(src, index)?;
        match src[*index] {
            ',' => *index += 1,
            ']' => {
                *index += 1;
                return Some(result);
            }
            _ => return None,
        }
    }
    None
}

fn expect_word(src: &[char], index: &mut usize, word: &str) -> Option<()> {
    let mut chars = word.chars();
    loop {
        let c = chars.next();
        if c.is_none() {
            return Some(());
        }
        check_eof(src, index)?;
        if src[*index] != c.unwrap() {
            return None;
        }
        *index += 1;
    }
}

fn parse_true(src: &[char], index: &mut usize) -> Option<Value> {
    expect_word(src, index, TRUE).map(|_| Value::Bool(true))
}

fn parse_false(src: &[char], index: &mut usize) -> Option<Value> {
    expect_word(src, index, FALSE).map(|_| Value::Bool(false))
}

fn parse_null(src: &[char], index: &mut usize) -> Option<Value> {
    expect_word(src, index, NULL).map(|_| Value::Null)
}

// TODO: This doesn't handle UTF-16 surrogate pairs
fn parse_string_unicode(src: &[char], index: &mut usize) -> Option<char> {
    if src.len() <= *index + 4 {
        return None;
    }
    let mut v: u32 = 0;
    for i in 1..5 {
        let d = src[*index + i].to_digit(16).unwrap_or(16);
        if d == 16 {
            return None;
        }
        v = v * 16 + d;
    }
    *index += 4;

    unsafe { Some(char::from_u32_unchecked(v)) }
}

fn parse_string(src: &[char], index: &mut usize) -> Option<String> {
    expect(src, index, '"')?;
    let mut string = String::new();
    let mut escaped = false;
    while src.len() > *index {
        if escaped {
            let c = match src[*index] {
                'b' => '\u{0008}',
                'f' => '\u{000c}',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '\n' => '\0',
                '\r' => '\0',
                'u' => parse_string_unicode(src, index).unwrap_or('\u{fffd}'),
                _ => src[*index],
            };
            if c != '\0' {
                string.push(c);
            }
            escaped = false;
        } else {
            match src[*index] {
                '\\' => escaped = true,
                '"' => {
                    *index += 1;
                    return Some(string);
                }
                _ => string.push(src[*index]),
            }
        }
        *index += 1;
    }
    None
}

fn parse_number_integer(src: &[char], index: &mut usize) -> f64 {
    let mut v: f64 = 0 as f64;
    while src.len() > *index && src[*index].is_ascii_digit() {
        v = v * 10.0 + src[*index].to_digit(10).unwrap() as f64;
        *index += 1;
    }
    v
}

fn parse_number_decimal(src: &[char], index: &mut usize) -> f64 {
    let head = *index;
    let v = parse_number_integer(src, index);
    v * f64::powi(0.1, (*index - head) as i32)
}

fn parse_number(src: &[char], index: &mut usize) -> Option<Value> {
    let mut result = 0 as f64;
    let mut sign = 1;
    check_eof(src, index)?;

    // Handle sign
    if src[*index] == '-' {
        sign = -1;
        *index += 1;
        check_eof(src, index)?;
    }

    if src[*index] != '0' {
        result += parse_number_integer(src, index);
    } else {
        *index += 1;
    }

    if src.len() <= *index {
        return Some(Value::Number(result * sign as f64));
    }

    if src[*index] == '.' {
        *index += 1;
        result += parse_number_decimal(src, index);
        if src.len() <= *index {
            return Some(Value::Number(result * sign as f64));
        }
    }

    if src[*index] == 'e' || src[*index] == 'E' {
        *index += 1;
        check_eof(src, index)?;
        let mut e_sign = 1;
        if src[*index] == '-' || src[*index] == '+' {
            e_sign = if src[*index] == '-' { -1 } else { 1 };
            *index += 1;
            check_eof(src, index)?;
        }
        let e = parse_number_integer(src, index);
        result *= f64::powi(10.0, e as i32 * e_sign);
    }

    Some(Value::Number(result * sign as f64))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn parse_true() {
        let result = parse("true");
        assert!(result.is_some());
        assert_eq!(result, Some(Value::Bool(true)));
    }

    #[test]
    fn parse_false() {
        let result = parse("false");
        assert!(result.is_some());
        assert_eq!(result, Some(Value::Bool(false)));
    }

    #[test]
    fn parse_null() {
        let result = parse("null");
        assert!(result.is_some());
        assert_eq!(result, Some(Value::Null));
    }

    #[test]
    fn parse_string() {
        let result = parse(r#""string""#);
        assert!(result.is_some());
        assert_eq!(result, Some(Value::String("string".to_owned())));
    }

    #[test]
    fn parse_integer() {
        let result = parse(r#"123"#);
        assert!(result.is_some());
        assert_eq!(result, Some(Value::Number(123.0)));
    }

    #[test]
    fn parse_float() {
        let result = parse(r#"123.123"#);
        assert!(result.is_some());
        assert_eq!(result, Some(Value::Number(123.123)));
    }

    #[test]
    fn parse_exponent() {
        let inputs = vec!["1e10", "1E10", "2E-10", "2e-10"];
        let results = vec![1e10, 1E10, 2E-10, 2e-10];
        for (i, input) in inputs.iter().enumerate() {
            let result = parse(input);
            assert!(result.is_some());
            assert_eq!(result, Some(Value::Number(results[i])));
        }
    }

    #[test]
    fn parse_string_unicode() {
        let result = parse(r#""\u0376""#);
        assert!(result.is_some());
        assert_eq!(result, Some(Value::String("Ͷ".to_owned())));
    }

    #[test]
    fn parse_empty_object() {
        let result = parse("{}");
        assert!(result.is_some());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn parse_empty_array() {
        let result = parse("[]");
        assert!(result.is_some());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn parse_object() {
        let input = r#"
            {
                "null": null,
                "true": true,
                "false": false
            }
        "#;
        let result = parse(input);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.get("null"), Some(&Value::Null));
        assert_eq!(result.get("true"), Some(&Value::Bool(true)));
        assert_eq!(result.get("false"), Some(&Value::Bool(false)));
    }

    #[test]
    fn parse_nested_object() {
        let input = r#"
            {
                "null": null,
                "true": true,
                "false": false,
                "other": {
                    "string": "hello",
                    "number": 123,
                }
            }
        "#;
        let result = parse(input);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.get("null"), Some(&Value::Null));
        assert_eq!(result.get("true"), Some(&Value::Bool(true)));
        assert_eq!(result.get("false"), Some(&Value::Bool(false)));
        assert_eq!(
            result.get("other"),
            Some(&Value::Object(BTreeMap::from([
                ("string".to_owned(), Value::String("hello".to_owned())),
                ("number".to_owned(), Value::Number(123.0))
            ])))
        )
    }

    #[test]
    fn parse_array() {
        let input = r#"[null, true, false, "hello", 123]"#;
        let result = parse(input);
        assert!(result.is_some());
        assert_eq!(
            result,
            Some(Value::Array(vec![
                Value::Null,
                Value::Bool(true),
                Value::Bool(false),
                Value::String("hello".to_owned()),
                Value::Number(123.0)
            ]))
        );
    }

    #[test]
    fn parse_nested_array() {
        let input = r#"[null, true, false, "hello", 123, [null, true, false, "hello", 123]]"#;
        let result = parse(input);
        assert!(result.is_some());
        assert_eq!(
            result,
            Some(Value::Array(vec![
                Value::Null,
                Value::Bool(true),
                Value::Bool(false),
                Value::String("hello".to_owned()),
                Value::Number(123.0),
                Value::Array(vec![
                    Value::Null,
                    Value::Bool(true),
                    Value::Bool(false),
                    Value::String("hello".to_owned()),
                    Value::Number(123.0),
                ])
            ]))
        );
    }
}
