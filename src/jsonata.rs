use json::JsonValue;

use crate::evaluator::{evaluate, Binding, Frame, Value};
use crate::parser::{parse, Node};
use crate::JsonAtaResult;

pub struct JsonAta<'a> {
    root_frame: Frame<'a>,
    ast: Node,
}

impl<'a> JsonAta<'a> {
    pub fn new(expr: &str) -> JsonAtaResult<Self> {
        let root_frame = Frame::new();

        // // TODO: Apply statics to the environment
        // environment.bind("sum", Binding::Function(&sum, "<a<n>:n>"));

        // TODO: Probably could just do this once somewhere to avoid doing it every time

        Ok(Self {
            root_frame,
            ast: parse(expr)?,
        })
    }

    pub fn evaluate(&mut self, input: Option<&JsonValue>) -> JsonAtaResult<Option<JsonValue>> {
        self.root_frame.bind("$", Binding::Var(input.into()));

        let mut input: Value = input.into();
        if input.is_array() {
            input = Value::wrap(&input);
        }

        let result = evaluate(&self.ast, &input, &mut self.root_frame)?;

        //println!("{:#?}", result);

        Ok(result.into())
    }

    pub fn assign_var(&mut self, name: &str, value: &JsonValue) {
        self.root_frame.bind(name, Binding::Var(value.into()));
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
