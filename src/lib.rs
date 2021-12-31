pub mod ast;
pub mod error;
pub mod evaluator;
pub mod functions;
pub mod json;
pub mod node_pool;
pub mod parser;
pub mod position;
pub mod process;
pub mod symbol;
pub mod tokenizer;
pub mod value;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

use ast::Node;
use evaluator::Evaluator;
use value::{Value, ValuePool};

pub struct JsonAta {
    ast: Node,
    pool: ValuePool,
    // frame: FrameLink,
}

impl JsonAta {
    pub fn new(expr: &str) -> Result<JsonAta> {
        Ok(Self {
            ast: parser::parse(expr)?,
            pool: ValuePool::new(),
            // frame: Frame::new(),
        })
    }

    pub fn ast(&self) -> &Node {
        &self.ast
    }

    pub fn assign_var(&mut self, name: &str, value: Value) {
        // self.frame.borrow_mut().bind(name, value)
        todo!()
    }

    pub fn evaluate(&self, input: Option<&str>) -> Result<Value> {
        let input = match input {
            Some(input) => json::parse(input).unwrap(),
            None => Value::new_undefined(self.pool.clone()),
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

        // self.frame.borrow_mut().bind("$", Rc::clone(&input));

        let evaluator = Evaluator::new(self.pool.clone());
        evaluator.evaluate()
    }
}
