#![cfg(test)]
extern crate test_generator;

use std::fs;
use std::path;
use test_generator::test_resources;

use jsonata::json;
use jsonata::JsonAta;

// TODO: timelimit, depth

#[test_resources("tests/testsuite/groups/*/*.json")]
fn t(resource: &str) {
    let test = fs::read_to_string(resource).expect("Could not read test case");
    let mut test = json::parse(&test).unwrap();

    // If it's not an array, make it an array
    if !test.is_array() {
        test = test.wrap_in_array();
    }

    for case in test.members() {
        let expr = if case.get_entry("expr").is_string() {
            case.get_entry("expr").as_string()
        } else if case.get_entry("expr-file").is_string() {
            let expr_file = path::Path::new(resource)
                .parent()
                .unwrap()
                .join(case.get_entry("expr-file").as_string());
            fs::read_to_string(expr_file).expect("Could not read expr-file")
        } else {
            panic!("No expression")
        };

        let data = if !case.get_entry("data").is_undefined() && !case.get_entry("data").is_null() {
            case.get_entry("data")
        } else if case.get_entry("dataset").is_string() {
            let dataset_file = format!(
                "tests/testsuite/datasets/{}.json",
                case.get_entry("dataset").as_string()
            );
            let json = fs::read_to_string(&dataset_file)
                .unwrap_or_else(|_e| panic!("Could not read dataset file: {}", dataset_file));
            json::parse(&json).unwrap()
        } else {
            case.pool.undefined()
        };

        let jsonata = JsonAta::new(&expr);

        match jsonata {
            Ok(mut jsonata) => {
                if case.get_entry("bindings").is_object() {
                    for (key, value) in case.get_entry("bindings").entries() {
                        jsonata.assign_var(key, value);
                    }
                }

                let result = jsonata.evaluate_with_value(data);

                match result {
                    Ok(result) => {
                        if case.get_entry("undefinedResult").is_bool()
                            && case.get_entry("undefinedResult") == true
                        {
                            assert!(result.is_undefined())
                        } else if !case.get_entry("result").is_undefined() {
                            // For numeric results, we can't compare directly due to floating point
                            // error
                            if case.get_entry("result").is_number() {
                                assert!(
                                    (case.get_entry("result").as_f64() - result.as_f64()).abs()
                                        < 1e-10
                                );
                            } else {
                                assert!(result == case.get_entry("result"));
                            }
                        }
                    }
                    Err(error) => {
                        assert!(!case.get_entry("code").is_null());
                        assert!(case.get_entry("code") == error.code());
                    }
                }
            }
            Err(error) => {
                // The parsing error is expected, let's make sure it matches
                assert!(!case.get_entry("code").is_null());
                assert!(case.get_entry("code") == error.code());
            }
        }
    }
}
