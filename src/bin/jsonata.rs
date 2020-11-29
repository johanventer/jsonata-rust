use json::stringify_pretty;
use std::path::PathBuf;
use structopt::StructOpt;

use jsonata::JsonAta;

/// A command line JSON processor using JSONata
#[derive(StructOpt)]
#[structopt(name = "jsonata")]
struct Opt {
    /// Parse the given expression, print the AST and exit
    #[structopt(short, long)]
    ast: bool,

    /// File containing the JSONata expression to evaluate (overrides expr on command line)
    #[structopt(short, long, parse(from_os_str))]
    expr_file: Option<PathBuf>,

    /// Input JSON file (if not specified, STDIN)
    #[structopt(short, long, parse(from_os_str))]
    input_file: Option<PathBuf>,

    /// JSONata expression to evaluate
    expr: Option<String>,

    /// JSON input
    input: Option<String>,
}

fn main() {
    let opt = Opt::from_args();

    let expr = match opt.expr_file {
        Some(expr_file) => {
            let expr = std::fs::read(expr_file).expect("Could not read expression input file");
            String::from_utf8_lossy(&expr).to_string()
        }
        None => opt.expr.expect("No JSONata expression provided."),
    };

    let jsonata = JsonAta::new(&expr).expect("Could not parse JSONata expression");

    if opt.ast {
        // TODO(johan): JSON formatting of the AST
        println!("{:#?}", jsonata.ast());
        return;
    }

    let input = match opt.input_file {
        Some(input_file) => {
            let input = std::fs::read(input_file).expect("Could not read the JSON input file");
            Some(json::parse(&String::from_utf8_lossy(&input)).expect("Could not parse input JSON"))
        }
        None => match opt.input {
            Some(input) => Some(json::parse(&input).expect("Could not parse input JSON")),
            None => None,
        },
    };

    let result = jsonata
        .evaluate(input.as_ref())
        .expect("Failed to evaluate JSONata");

    match result {
        Some(value) => println!("{}", stringify_pretty(value, 2)),
        None => println!("undefined"),
    };
}
