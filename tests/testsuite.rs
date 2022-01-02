#![cfg(test)]
extern crate test_generator;

use std::fs;
use std::path;
use test_generator::test_resources;

use jsonata::json;
use jsonata::value::{ArrayFlags, ValuePool};
use jsonata::JsonAta;

// TODO: timelimit, depth
#[test_resources("tests/testsuite/groups/*/*.json")]
fn t(resource: &str) {
    let pool = ValuePool::new();

    let test = fs::read_to_string(resource)
        .unwrap_or_else(|_| panic!("Failed to read test case: {}", resource));

    let test = json::parse_with_pool(&test, pool.clone())
        .unwrap()
        .wrap_in_array_if_needed(ArrayFlags::empty());

    for case in test.members() {
        let expr = case.get_entry("expr");
        let expr_file = case.get_entry("expr-file");

        let expr = if expr.is_string() {
            expr.as_string()
        } else if expr_file.is_string() {
            fs::read_to_string(
                path::Path::new(resource)
                    .parent()
                    .unwrap()
                    .join(expr_file.as_string()),
            )
            .unwrap_or_else(|_| panic!("Failed to read expr-file: {}", expr_file.as_string()))
        } else {
            panic!("No expression")
        };

        let data = case.get_entry("data");
        let dataset = case.get_entry("dataset");

        let data = if dataset.is_string() {
            let dataset = format!("tests/testsuite/datasets/{}.json", dataset.as_string());
            let dataset = fs::read_to_string(&dataset)
                .unwrap_or_else(|_e| panic!("Could not read dataset file: {}", dataset));
            json::parse_with_pool(&dataset, pool.clone()).unwrap()
        } else {
            data
        };

        let jsonata = JsonAta::new_with_pool(&expr, pool.clone());

        match jsonata {
            Ok(jsonata) => {
                let bindings = case.get_entry("bindings");
                if bindings.is_object() {
                    for (key, value) in bindings.entries() {
                        jsonata.assign_var(key, value);
                    }
                }

                let result = jsonata.evaluate_with_value(data);

                match result {
                    Ok(result) => {
                        let undefined_result = case.get_entry("undefinedResult");
                        let expected_result = case.get_entry("result");
                        if undefined_result.is_bool() && undefined_result == true {
                            assert!(result.is_undefined())
                        } else if expected_result.is_number() {
                            assert!(result.is_number());
                            println!(
                                "expected: {}, actual: {}",
                                expected_result.as_f64(),
                                result.as_f64()
                            );
                            assert!(
                                (expected_result.as_f64() - result.as_f64()).abs() < f64::EPSILON
                            );
                        } else {
                            assert_eq!(result, expected_result);
                        }
                    }
                    Err(error) => {
                        println!("{}", error);
                        let code = case.get_entry("code");
                        assert_eq!(code, error.code());
                    }
                }
            }
            Err(error) => {
                println!("{}", error);
                let code = case.get_entry("code");
                assert_eq!(code, error.code());
            }
        }
    }
}
