#![cfg(test)]
extern crate test_generator;

use bumpalo::Bump;
use std::fs;
use std::path;
use test_generator::test_resources;

use jsonata::value::ArrayFlags;
use jsonata::{JsonAta, Value};

const SKIP: &[&str] = &[
    // The order of object properties in the output is not deterministic, 
    // so string comparison fails. If we were using something like a BTreeMap
    // or an IndexedMap then running these would be possible.
    "jsonata/tests/testsuite/groups/function-string/case018.json",
    "jsonata/tests/testsuite/groups/function-string/case027.json",
    "jsonata/tests/testsuite/groups/function-string/case028.json"
];

#[test_resources("jsonata/tests/testsuite/groups/*/*.json")]
fn t(resource: &str) {
    if SKIP.iter().any(|&s| s == resource) {
        return;
    }

    // Make sure we can find the .json file with the path to `test_resources`
    let working_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    std::env::set_current_dir(working_dir).unwrap();

    test_case(resource);
}

fn test_case(resource: &str) {
    let arena = Bump::new();
    let test_jsonata = JsonAta::new(
        &fs::read_to_string(path::Path::new(resource)).unwrap(),
        &arena,
    )
    .unwrap();
    let test = test_jsonata.evaluate(None).unwrap();
    let test = Value::wrap_in_array_if_needed(&arena, test, ArrayFlags::empty());

    for case in test.members() {
        let timelimit = &case["timelimit"];
        let timelimit = if timelimit.is_integer() {
            Some(timelimit.as_usize())
        } else {
            None
        };

        let depth = &case["depth"];
        let depth = if depth.is_integer() {
            Some(depth.as_usize())
        } else {
            None
        };

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
            .unwrap()
        } else {
            panic!("No expression")
        };

        let data = &case["data"];
        let dataset = &case["dataset"];

        let data = if dataset.is_string() {
            let dataset = format!("jsonata/tests/testsuite/datasets/{}.json", dataset.as_str());
            fs::read_to_string(&dataset).unwrap()
        } else if data.is_undefined() {
            "".to_string()
        } else {
            data.dump()
        };

        let jsonata = JsonAta::new(&expr, &arena);

        match jsonata {
            Ok(jsonata) => {
                if case["bindings"].is_object() {
                    for (key, value) in case["bindings"].entries() {
                        jsonata.assign_var(key, value);
                    }
                }

                let data = if data.is_empty() {
                    None
                } else {
                    Some(data.as_str())
                };

                let result = jsonata.evaluate_timeboxed(data, depth, timelimit);

                match result {
                    Ok(result) => {
                        let expected_result = &case["result"];

                        if case["undefinedResult"] == true {
                            assert!(result.is_undefined());
                        } else if case["unordered"] == true {
                            // Some test cases specify that the expected array result can be unordered
                            // because the order is implementation dependent. To implement that here
                            // we do a pretty bad O(n^2) just to see if the test passes.
                            assert!(expected_result.is_array());
                            assert!(result.is_array());
                            for expected_member in expected_result.members() {
                                let mut found = false;
                                for member in result.members() {
                                    if member == expected_member {
                                        found = true;
                                        break;
                                    }
                                }
                                assert!(found);
                            }
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
