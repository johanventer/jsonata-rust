#![cfg(test)]
extern crate test_generator;

use std::fs;
use std::path;
use test_generator::test_resources;

use jsonata::json;
use jsonata::JsonAta;
use jsonata::Value;

// TODO: timelimit, depth

#[test_resources("tests/testsuite/groups/*/*.json")]
fn t(resource: &str) {
    if resource.contains("skip") {
        return;
    }

    let test = fs::read_to_string(resource).expect("Could not read test case");
    let mut test = json::parse(&test).unwrap();

    // If it's not an array, make it an array
    if !test.is_array() {
        let original_test = test;
        test = Value::new_array();
        test.push(original_test);
    }

    for case in test.iter() {
        let expr = if !case["expr"].is_undefined() {
            case["expr"].to_string()
        } else if !case["expr-file"].is_undefined() {
            let expr_file = path::Path::new(resource)
                .parent()
                .unwrap()
                .join(case["expr-file"].to_string());
            fs::read_to_string(expr_file).expect("Could not read expr-file")
        } else {
            panic!("No expression")
        };

        let data = if !case["data"].is_undefined() {
            case["data"].clone()
        } else if !case["dataset"].is_undefined() {
            let dataset_file = format!("tests/testsuite/datasets/{}.json", case["dataset"]);
            let json = fs::read_to_string(&dataset_file)
                .unwrap_or_else(|_e| panic!("Could not read dataset file: {}", dataset_file));
            json::parse(&json).unwrap()
        } else {
            Value::Undefined
        };

        let jsonata = JsonAta::new(&expr);

        match jsonata {
            Ok(jsonata) => {
                // for (key, value) in case["bindings"].entries() {
                //     jsonata.assign_var(key, value);
                // }

                let result = jsonata.evaluate_with_value(data);

                match result {
                    Ok(result) => {
                        if case["undefinedResult"].is_bool() && case["undefinedResult"] == true {
                            assert_eq!(Value::Undefined, result)
                        } else if !case["result"].is_undefined() {
                            // For numeric results, we can't compare directly due to floating point
                            // error
                            if case["result"].is_number() {
                                assert!((case["result"].as_f64() - result.as_f64()).abs() < 1e-10);
                            } else {
                                assert_eq!(case["result"], result);
                            }
                        }
                    }
                    Err(error) => {
                        assert!(!case["code"].is_null());
                        assert_eq!(case["code"], error.code());
                    }
                }
            }
            Err(error) => {
                // The parsing error is expected, let's make sure it matches
                assert!(!case["code"].is_null());
                assert_eq!(case["code"], error.code());
            }
        }
    }
}
