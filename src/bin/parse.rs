use json::stringify_pretty;
use std::env;

use jsonata::JsonAta;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Provide an expression");
        return;
    }

    let jsonata = JsonAta::new(&args[1]);
    println!("{}", stringify_pretty(jsonata.ast(), 2));
}
