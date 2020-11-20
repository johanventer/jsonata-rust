use json::JsonValue;
use std::cell::RefCell;
use std::rc::Rc;

use crate::evaluator::{evaluate, Binding, Frame, Value};
use crate::parser::{parse, Node};
use crate::JsonAtaResult;

pub struct JsonAta {
    ast: Node,
    frame: Rc<RefCell<Frame>>,
}

impl JsonAta {
    pub fn new(expr: &str) -> JsonAtaResult<Self> {
        let frame = Rc::new(RefCell::new(Frame::new()));

        // // TODO: Apply statics to the environment
        // environment.bind("sum", Binding::Function(&sum, "<a<n>:n>"));

        // TODO: Probably could just do this once somewhere to avoid doing it every time

        Ok(Self {
            ast: parse(expr)?,
            frame,
        })
    }

    pub fn evaluate(&mut self, input: Option<&JsonValue>) -> JsonAtaResult<Option<JsonValue>> {
        self.frame
            .borrow_mut()
            .bind("$", Binding::Var(Rc::new(input.into())));

        let mut input: Value = input.into();
        if input.is_array() {
            input = Value::wrap(&input);
        }

        let result = evaluate(&self.ast, &input, Rc::clone(&self.frame))?;

        //println!("{:#?}", result);

        Ok(result.into())
    }

    pub fn assign_var(&mut self, name: &str, value: &JsonValue) {
        self.frame
            .borrow_mut()
            .bind(name, Binding::Var(Rc::new(value.into())));
    }

    pub fn ast(&self) -> &Node {
        &self.ast
    }

    // pub fn ast(&self) -> JsonValue {
    //     self.ast.to_json()
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add() {
        let mut jsonata = JsonAta::new("1 + 3").unwrap();
        let result = jsonata.evaluate(None).unwrap().unwrap();
        assert_eq!(result, json::from(4));
    }

    #[test]
    fn sub() {
        let mut jsonata = JsonAta::new("1 - 3").unwrap();
        let result = jsonata.evaluate(None).unwrap().unwrap();
        assert_eq!(result, json::from(-2));
    }

    #[test]
    fn mul() {
        let mut jsonata = JsonAta::new("4 * 7").unwrap();
        let result = jsonata.evaluate(None).unwrap().unwrap();
        assert_eq!(result, json::from(28));
    }

    #[test]
    fn div() {
        let mut jsonata = JsonAta::new("10 / 2").unwrap();
        let result = jsonata.evaluate(None).unwrap().unwrap();
        assert_eq!(result, json::from(5));
    }

    #[test]
    fn modulo() {
        let mut jsonata = JsonAta::new("10 % 8").unwrap();
        let result = jsonata.evaluate(None).unwrap().unwrap();
        assert_eq!(result, json::from(2));
    }

    #[test]
    fn less_than_num_true() {
        let mut jsonata = JsonAta::new("3 < 4").unwrap();
        let result = jsonata.evaluate(None).unwrap().unwrap();
        assert_eq!(result, json::from(true));
    }

    #[test]
    fn less_than_num_false() {
        let mut jsonata = JsonAta::new("4 < 3").unwrap();
        let result = jsonata.evaluate(None).unwrap().unwrap();
        assert_eq!(result, json::from(false));
    }

    #[test]
    fn less_than_str_true() {
        let mut jsonata = JsonAta::new("\"3\" < \"4\"").unwrap();
        let result = jsonata.evaluate(None).unwrap().unwrap();
        assert_eq!(result, json::from(true));
    }

    #[test]
    fn less_than_str_false() {
        let mut jsonata = JsonAta::new("\"4\" < \"3\"").unwrap();
        let result = jsonata.evaluate(None).unwrap().unwrap();
        assert_eq!(result, json::from(false));
    }

    #[test]
    fn str_concat() {
        let mut jsonata = JsonAta::new("\"hello\" & \" world\"").unwrap();
        let result = jsonata.evaluate(None).unwrap().unwrap();
        assert_eq!(result, json::from("hello world"));
    }

    #[test]
    fn eq() {
        let mut jsonata = JsonAta::new("1 = 1").unwrap();
        let result = jsonata.evaluate(None).unwrap().unwrap();
        assert_eq!(result, json::from(true));
    }

    #[test]
    fn neq() {
        let mut jsonata = JsonAta::new("1 != 2").unwrap();
        let result = jsonata.evaluate(None).unwrap().unwrap();
        assert_eq!(result, json::from(true));
    }

    #[test]
    fn math() {
        let mut jsonata = JsonAta::new("(2 + 3) * 4 + 2").unwrap();
        let result = jsonata.evaluate(None).unwrap().unwrap();
        assert_eq!(result, json::from(22));
    }
}
