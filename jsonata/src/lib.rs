// TODO: Fix visibility of all these modules, they're all pub for now
pub mod ast;
pub mod evaluator;
pub mod frame;
pub mod functions;
pub mod parser;
pub mod symbol;
pub mod tokenizer;
pub mod value;

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
    arena: &'a Bump,
}

impl<'a> JsonAta<'a> {
    pub fn new(expr: &str, arena: &'a Bump) -> Result<JsonAta<'a>> {
        Ok(Self {
            ast: parser::parse(expr)?,
            frame: Frame::new(),
            arena,
        })
    }

    pub fn ast(&self) -> &Ast {
        &self.ast
    }

    pub fn assign_var(&self, name: &str, value: &'a Value<'a>) {
        self.frame.bind(name, value)
    }

    pub fn evaluate(&self, input: Option<&str>) -> Result<&'a Value<'a>> {
        let input = match input {
            Some(input) => {
                let input_ast = parser::parse(input)?;
                let evaluator = Evaluator::new(None, self.arena);
                evaluator.evaluate(&input_ast, Value::undefined(), &Frame::new())?
            }
            None => Value::undefined(),
        };

        // If the input is an array, wrap it in an array so that it gets treated as a single input
        let input = if input.is_array() {
            Value::wrap_in_array(self.arena, input, ArrayFlags::WRAPPED)
        } else {
            input
        };

        macro_rules! bind {
            ($name:literal, $new:ident, $fn:ident) => {
                self.frame.bind($name, Value::$new(self.arena, $name, $fn));
            };
        }

        self.frame.bind("$", input);
        bind!("lookup", nativefn2, fn_lookup);
        bind!("append", nativefn2, fn_append);
        bind!("boolean", nativefn1, fn_boolean);
        bind!("filter", nativefn2, fn_filter);
        bind!("string", nativefn1, fn_string);
        bind!("count", nativefn1, fn_count);
        bind!("not", nativefn1, fn_not);
        bind!("uppercase", nativefn1, fn_uppercase);
        bind!("lowercase", nativefn1, fn_lowercase);
        bind!("substring", nativefn3, fn_substring);
        bind!("abs", nativefn1, fn_abs);
        bind!("max", nativefn1, fn_max);
        bind!("min", nativefn1, fn_min);
        bind!("ceil", nativefn1, fn_ceil);
        bind!("floor", nativefn1, fn_floor);
        bind!("sum", nativefn1, fn_sum);

        let chain_ast = parser::parse("function($f, $g) { function($x){ $g($f($x)) } }")?;
        let evaluator = Evaluator::new(Some(chain_ast), self.arena);
        evaluator.evaluate(&self.ast, input, &self.frame)
    }
}
