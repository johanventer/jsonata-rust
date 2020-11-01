use json::stringify_pretty;
use std::env;

use jsonata::JsonAta;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: jsonata <expr> <input>");
        return;
    }

    let jsonata = JsonAta::new(&args[1]);
    let result = jsonata.evaluate(args[2].clone());

    match result {
        Ok(value) => match value {
            Some(value) => println!("{}", stringify_pretty(value, 2)),
            None => println!("undefined"),
        },
        Err(e) => println!("ERROR: {}", e),
    }
}
