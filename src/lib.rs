// use std::rc::Rc;

mod error;
mod evaluator;
mod functions;
pub mod json;
mod parser;

pub use error::{Error, InvalidJson};
use evaluator::Frame;
pub use evaluator::Value;
pub use parser::ast::*;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub struct JsonAta {
    ast: Node,
    // frame: FramePtr,
}

impl JsonAta {
    pub fn new(expr: &str) -> Result<JsonAta> {
        Ok(Self {
            ast: parser::parse(expr)?,
            // frame: Frame::new_ptr(),
        })
    }

    pub fn ast(&self) -> &Node {
        &self.ast
    }

    // pub fn assign_var(&mut self, name: &str, value: &JsonValue) {
    //     self.frame
    //         .borrow_mut()
    //         .bind(name, Rc::new(Value::from_raw(Some(value))));
    // }

    pub fn evaluate(&self, input: Option<&str>) -> Result<Value> {
        let input = match input {
            Some(input) => json::parse(input).unwrap(),
            None => Value::Undefined,
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

        let frame = Frame::new();
        let result = evaluator::evaluate(&self.ast, &input, frame)?;
        Ok(result)
    }
}
