#![cfg(test)]
extern crate test_generator;

use bumpalo::Bump;
use std::fs;
use std::path;
use test_generator::test_resources;

use jsonata::json;
use jsonata::value::ArrayFlags;
use jsonata::{JsonAta, Value};

// TODO: timelimit, depth
#[test_resources("jsonata/tests/testsuite/groups/*/*.json")]
fn t(resource: &str) {
    let working_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    std::env::set_current_dir(working_dir).unwrap();

    let test = fs::read_to_string(resource)
        .unwrap_or_else(|_| panic!("Failed to read test case: {}", resource));

    let arena = Bump::new();

    let test = json::parse(&test, &arena).unwrap();

    let test = Value::wrap_in_array_if_needed(&arena, test, ArrayFlags::empty());

    for case in test.members() {
        let expr = &case["expr"];
        let expr_file = &case["expr-file"];

        let expr = if expr.is_string() {
            expr.as_str().to_string()
        } else if expr_file.is_string() {
            fs::read_to_string(
                path::Path::new(resource)
                    .parent()
                    .unwrap()
                    .join(expr_file.as_str().to_string()),
            )
            .unwrap_or_else(|_| panic!("Failed to read expr-file: {}", expr_file.as_str()))
        } else {
            panic!("No expression")
        };

        let data = &case["data"];
        let dataset = &case["dataset"];

        let data = if dataset.is_string() {
            let dataset = format!("jsonata/tests/testsuite/datasets/{}.json", dataset.as_str());
            fs::read_to_string(&dataset)
                .unwrap_or_else(|_e| panic!("Could not read dataset file: {}", dataset))
        } else if data.is_undefined() {
            "".to_string()
        } else {
            data.dump()
        };

        let jsonata = JsonAta::new(&expr);

        match jsonata {
            Ok(jsonata) => {
                if case["bindings"].is_object() {
                    for (key, value) in case["bindings"].entries() {
                        jsonata.assign_var(key, from(value, &arena));
                    }
                }

                let data = if data.is_empty() {
                    None
                } else {
                    Some(data.as_str())
                };

                let result = jsonata.evaluate(data);

                match result {
                    Ok(result) => {
                        let expected_result = from(&case["result"], &arena);

                        if case["undefinedResult"] == true {
                            assert!(result.is_undefined());
                        } else if case["result"].is_number() {
                            assert!(result.is_number());
                            assert!(
                                f64::abs(expected_result.as_f64() - result.as_f64())
                                    <= f64::EPSILON
                            );
                        } else {
                            assert_eq!(result, expected_result);
                        }
                    }
                    Err(error) => {
                        println!("{}", error);
                        assert_eq!(case["code"], error.code());
                    }
                }
            }
            Err(error) => {
                println!("{}", error);
                assert_eq!(case["code"], error.code());
            }
        }
    }
}

pub fn from<'a>(value: &Value, arena: &'a Bump) -> &'a Value<'a> {
    match value {
        Value::Undefined => Value::undefined(),
        Value::Null => Value::null(arena),
        Value::Number(n) => Value::number(arena, *n),
        Value::Bool(b) => Value::bool(arena, *b),
        Value::String(s) => Value::string(arena, s),
        Value::Array(a, f) => {
            let array = Value::array_with_capacity(arena, a.len(), *f);
            a.iter().for_each(|i| array.push(from(i, arena)));
            array
        }
        Value::Object(o) => {
            let obj = Value::object_with_capacity(arena, o.len());
            o.iter().for_each(|(k, v)| obj.insert(k, from(v, arena)));
            obj
        }
        _ => panic!("Can't call Value::from on functions"),
    }
}
