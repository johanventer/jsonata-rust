// TODO: Fix visibility of all these modules, they're all pub for now
pub mod ast;
pub mod error;
pub mod evaluator;
pub mod frame;
pub mod functions;
pub mod json;
pub mod parser;
pub mod position;
pub mod symbol;
pub mod tokenizer;
pub mod value;

pub use error::Error;

/// The Result type used by this crate
pub type Result<T> = std::result::Result<T, Error>;

use ast::Ast;
use evaluator::Evaluator;
use frame::Frame;
use functions::*;
use value::{ArrayFlags, Value, ValuePool};

pub struct JsonAta {
    ast: Ast,
    pool: ValuePool,
    frame: Frame,
}

impl JsonAta {
    pub fn new(expr: &str) -> Result<JsonAta> {
        Ok(Self {
            ast: parser::parse(expr)?,
            pool: ValuePool::new(),
            frame: Frame::new(),
        })
    }

    pub fn new_with_pool(expr: &str, pool: ValuePool) -> Result<JsonAta> {
        Ok(Self {
            ast: parser::parse(expr)?,
            pool,
            frame: Frame::new(),
        })
    }

    pub fn ast(&self) -> &Ast {
        &self.ast
    }

    pub fn assign_var(&self, name: &str, value: &Value) {
        self.frame.bind(name, value)
    }

    pub fn evaluate(&self, input: Option<&str>) -> Result<Value> {
        let input = match input {
            Some(input) => json::parse_with_pool(input, self.pool.clone()).unwrap(),
            None => self.pool.undefined(),
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
                self.frame.bind($name, &self.pool.$new($name, $fn));
            };
        }

        self.frame.bind("$", &input);
        bind!("lookup", nativefn2, fn_lookup);
        bind!("append", nativefn2, fn_append);
        bind!("boolean", nativefn1, fn_boolean);
        bind!("filter", nativefn2, fn_filter);
        bind!("string", nativefn1, fn_string);
        bind!("count", nativefn1, fn_count);

        let evaluator = Evaluator::new(self.pool.clone());
        evaluator.evaluate(&self.ast, &input, &self.frame)
    }
}
