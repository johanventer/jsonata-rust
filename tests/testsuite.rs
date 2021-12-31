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
        let expr = if case.get("expr").is_string() {
            case.get("expr").as_string()
        } else if case.get("expr-file").is_string() {
            let expr_file = path::Path::new(resource)
                .parent()
                .unwrap()
                .join(case.get("expr-file").as_string());
            fs::read_to_string(expr_file).expect("Could not read expr-file")
        } else {
            panic!("No expression")
        };

        let data = if !case.get("data").is_undefined() && !case.get("data").is_null() {
            case.get("data")
        } else if case.get("dataset").is_string() {
            let dataset_file = format!(
                "tests/testsuite/datasets/{}.json",
                case.get("dataset").as_string()
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
                if case.get("bindings").is_object() {
                    for (key, value) in case.get("bindings").entries() {
                        jsonata.assign_var(&key, value);
                    }
                }

                let result = jsonata.evaluate_with_value(data);

                match result {
                    Ok(result) => {
                        if case.get("undefinedResult").is_bool()
                            && case.get("undefinedResult") == true
                        {
                            assert!(result.is_undefined())
                        } else if !case.get("result").is_undefined() {
                            // For numeric results, we can't compare directly due to floating point
                            // error
                            if case.get("result").is_number() {
                                assert!(
                                    (case.get("result").as_f64() - result.as_f64()).abs() < 1e-10
                                );
                            } else {
                                assert_eq!(case.get("result"), result);
                            }
                        }
                    }
                    Err(error) => {
                        assert!(!case.get("code").is_null());
                        assert_eq!(case.get("code"), error.code());
                    }
                }
            }
            Err(error) => {
                // The parsing error is expected, let's make sure it matches
                assert!(!case.get("code").is_null());
                assert_eq!(case.get("code"), error.code());
            }
        }
    }
}
