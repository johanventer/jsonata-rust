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
        None => opt.expr.expect("No JSONata expression provided"),
    };

    let jsonata = JsonAta::new(&expr);

    match jsonata {
        Ok(jsonata) => {
            if opt.ast {
                println!("{:#?}", jsonata.ast());
                return;
            }

            let input = match opt.input_file {
                Some(input_file) => {
                    std::fs::read_to_string(input_file).expect("Could not read the JSON input file")
                }
                None => opt.input.unwrap_or_else(|| "{}".to_string()),
            };

            match jsonata.evaluate(Some(&input)) {
                Ok(result) => println!("{}", result.pretty(4)),
                Err(error) => println!("{}", error),
            }
        }
        Err(error) => println!("{}", error),
    }
}
