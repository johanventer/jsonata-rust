use json::stringify_pretty;
use std::env;

use jsonata::{JsonAta, JsonAtaResult};

fn main() -> JsonAtaResult<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: jsonata <expr> [input] [bindings]");
        return Ok(());
    }

    match JsonAta::new(&args[1]) {
        Ok(mut jsonata) => {
            println!("{:#?}", jsonata.ast());

            let input = if args.len() > 2 && !args[2].is_empty() {
                Some(json::parse(&args[2]).unwrap())
            } else {
                None
            };

            if args.len() > 3 && !args[3].is_empty() {
                let bindings = json::parse(&args[3]).unwrap();
                for (key, value) in bindings.entries() {
                    jsonata.assign_var(key, value);
                }
            }

            let result = jsonata.evaluate(input.as_ref());

            match result {
                Ok(value) => match value {
                    Some(value) => println!("{}", stringify_pretty(value, 2)),
                    None => println!("undefined"),
                },
                Err(e) => println!("{}", e),
            };
        }
        Err(e) => println!("{}", e),
    }

    Ok(())
}
