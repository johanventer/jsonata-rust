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

use jsonata_errors::{Error, Result};

use ast::Ast;
use evaluator::Evaluator;
use frame::Frame;
use functions::*;
use value::{ArrayFlags, Value, ValueArena};

pub struct JsonAta {
    ast: Ast,
    arena: ValueArena,
    frame: Frame,
}

impl JsonAta {
    pub fn new(expr: &str) -> Result<JsonAta> {
        Ok(Self {
            ast: parser::parse(expr)?,
            arena: ValueArena::new(),
            frame: Frame::new(),
        })
    }

    pub fn new_with_arena(expr: &str, arena: ValueArena) -> Result<JsonAta> {
        Ok(Self {
            ast: parser::parse(expr)?,
            arena,
            frame: Frame::new(),
        })
    }

    pub fn ast(&self) -> &Ast {
        &self.ast
    }

    pub fn assign_var(&self, name: &str, value: &Value) {
        self.frame.bind(name, self.arena.clone(), value)
    }

    pub fn evaluate(&self, input: Option<&str>) -> Result<Value> {
        let input = match input {
            Some(input) => json::parse_with_arena(input, self.arena.clone()).unwrap(),
            None => self.arena.undefined(),
        };

        self.evaluate_with_value(input)
    }

    pub fn evaluate_with_value(&self, input: Value) -> Result<Value> {
        // let mut input = Rc::new(Value::from_raw(input));
        // if input.is_array() {
        //     input = Rc::new(Value::wrap(Rc::clone(&input)));
        // }

        // // TODO: Apply statics
        // // self.frame
        // //     .borrow_mut()
        // //     .bind("string", Rc::new(Value::NativeFn(functions::string)))
        // //     .bind("boolean", Rc::new(Value::NativeFn(functions::boolean)));

        // If the input is an array, wrap it in an array so that it gets treated as a single input
        let input = if input.is_array() {
            input.wrap_in_array(ArrayFlags::WRAPPED)
        } else {
            input
        };

        macro_rules! bind {
            ($name:literal, $new:ident, $fn:ident) => {
                self.frame
                    .bind($name, self.arena.clone(), &self.arena.$new($name, $fn));
            };
        }

        self.frame.bind("$", self.arena.clone(), &input);
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

        let evaluator = Evaluator::new(self.arena.clone(), chain_ast);
        evaluator.evaluate(&self.ast, &input, &self.frame)
    }
}
