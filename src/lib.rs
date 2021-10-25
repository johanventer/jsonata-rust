use json::JsonValue;
use std::rc::Rc;

mod error;
mod evaluator;
mod functions;
mod parser;

pub use error::Error;
use evaluator::evaluate;
pub use evaluator::Value;
use evaluator::{Frame, FramePtr};
pub use parser::ast::*;
use parser::parse;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub struct JsonAta {
    ast: Box<Node>,
    frame: FramePtr,
}

impl JsonAta {
    pub fn new(expr: &str) -> Result<Self> {
        Ok(Self {
            ast: parse(expr)?,
            frame: Frame::new_ptr(),
        })
    }

    pub fn ast(&self) -> &Node {
        self.ast.as_ref()
    }

    pub fn assign_var(&mut self, name: &str, value: &JsonValue) {
        self.frame
            .borrow_mut()
            .bind(name, Rc::new(Value::from_raw(Some(value))));
    }

    pub fn evaluate(&self, input: Option<&JsonValue>) -> Result<Option<JsonValue>> {
        let mut input = Rc::new(Value::from_raw(input));
        if input.is_array() {
            input = Rc::new(Value::wrap(Rc::clone(&input)));
        }

        // TODO: Apply statics
        // self.frame
        //     .borrow_mut()
        //     .bind("string", Rc::new(Value::NativeFn(functions::string)))
        //     .bind("boolean", Rc::new(Value::NativeFn(functions::boolean)));

        self.frame.borrow_mut().bind("$", Rc::clone(&input));

        let result = evaluate(&self.ast, input, Rc::clone(&self.frame))?;

        Ok(result.as_json())
    }
}
