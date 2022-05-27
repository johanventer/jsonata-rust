// TODO: Fix visibility of all these modules, they're all pub for now
pub mod ast;
pub mod evaluator;
pub mod frame;
pub mod functions;
pub mod json;
pub mod parser;
pub mod symbol;
pub mod tokenizer;
pub mod value;

pub use functions::FunctionContext;
pub use jsonata_errors::{Error, Result};
pub use value::Value;

use bumpalo::Bump;

use ast::Ast;
use evaluator::Evaluator;
use frame::Frame;
use functions::*;
use value::ArrayFlags;

pub struct JsonAta<'a> {
    ast: Ast,
    frame: Frame<'a>,
    arena: Bump,
}

impl<'a> JsonAta<'a> {
    pub fn new(expr: &str) -> Result<JsonAta<'a>> {
        Ok(Self {
            ast: parser::parse(expr)?,
            frame: Frame::new(),
            arena: Bump::new(),
        })
    }

    pub fn ast(&self) -> &Ast {
        &self.ast
    }

    pub fn assign_var<'other>(&'other self, name: &str, value: &'other Value<'other>)
    where
        'other: 'a,
    {
        self.frame.bind(name, value)
    }

    pub fn evaluate(&'a self, input: Option<&str>) -> Result<&'a Value<'a>> {
        let input = match input {
            Some(input) => json::parse(input, &self.arena).unwrap(),
            None => Value::undefined(),
        };

        // If the input is an array, wrap it in an array so that it gets treated as a single input
        let input = if input.is_array() {
            Value::wrap_in_array(&self.arena, input, ArrayFlags::WRAPPED)
        } else {
            input
        };

        macro_rules! bind_native {
            ($name:literal, $arity:literal, $fn:ident) => {
                self.frame
                    .bind($name, Value::nativefn(&self.arena, $name, $arity, $fn));
            };
        }

        self.frame.bind("$", input);
        bind_native!("abs", 1, fn_abs);
        bind_native!("append", 2, fn_append);
        bind_native!("boolean", 1, fn_boolean);
        bind_native!("ceil", 1, fn_ceil);
        bind_native!("count", 1, fn_count);
        bind_native!("filter", 2, fn_filter);
        bind_native!("floor", 1, fn_floor);
        bind_native!("lookup", 2, fn_lookup);
        bind_native!("lowercase", 1, fn_lowercase);
        bind_native!("max", 1, fn_max);
        bind_native!("min", 1, fn_min);
        bind_native!("not", 1, fn_not);
        bind_native!("string", 1, fn_string);
        bind_native!("substring", 3, fn_substring);
        bind_native!("sum", 1, fn_sum);
        bind_native!("uppercase", 1, fn_uppercase);

        let chain_ast = parser::parse("function($f, $g) { function($x){ $g($f($x)) } }")?;
        let evaluator = Evaluator::new(chain_ast, &self.arena);
        evaluator.evaluate(&self.ast, input, &self.frame)
    }
}
