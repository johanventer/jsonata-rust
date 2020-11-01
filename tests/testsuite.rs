#![cfg(test)]
extern crate test_generator;

use std::fs;
use test_generator::test_resources;

use jsonata::JsonAta;

#[test_resources("tests/testsuite/groups/*/*.json")]
fn t(resource: &str) {
    let json = fs::read_to_string(resource).expect("Could not read test case");
    let json = json::parse(&json).unwrap();

    let data = if !json["data"].is_null() {
        json["data"].to_string()
    } else if !json["dataset"].is_null() {
        let dataset =
            fs::read_to_string(format!("tests/testsuite/datasets/{}.json", json["dataset"]))
                .expect("Could not read dataset file");
        json::parse(&dataset).unwrap().to_string()
    } else {
        "undefined".to_string()
    };

    let jsonata = JsonAta::new(&json["expr"].to_string()).unwrap();

    if json["undefinedResult"].is_boolean() && json["undefinedResult"] == true {
        assert_eq!(None, jsonata.evaluate(data).unwrap())
    } else if !json["result"].is_null() {
        assert_eq!(json["result"], jsonata.evaluate(data).unwrap().unwrap());
    } else {
        assert!(!json["code"].is_null());
        // TODO: Handle tests that should fail
    }
}
