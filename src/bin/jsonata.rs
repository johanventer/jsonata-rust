use json::stringify_pretty;
use std::env;

use jsonata::{JsonAta, JsonAtaResult};

fn main() -> JsonAtaResult<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: jsonata <expr> <input>");
        return Ok(());
    }

    let jsonata = JsonAta::new(&args[1])?;

    println!("{:#?}", jsonata.ast());

    let input = json::parse(&args[2]).unwrap();
    let result = jsonata.evaluate(Some(&input));

    match result {
        Ok(value) => match value {
            Some(value) => println!("{}", stringify_pretty(value, 2)),
            None => println!("undefined"),
        },
        Err(e) => println!("ERROR: {}", e),
    };

    Ok(())
}
