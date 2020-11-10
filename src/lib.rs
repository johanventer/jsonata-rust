#![feature(or_patterns)]
#![feature(box_syntax)]

use json::JsonValue;

#[macro_use]
mod error;
mod ast;
mod evaluator;
mod frame;
mod functions;
mod parser;
mod symbol;
mod tokenizer;

pub use frame::Binding;

pub type JsonAtaResult<T> = std::result::Result<T, Box<dyn error::JsonAtaError>>;

pub struct JsonAta<'a> {
    root_frame: frame::Frame<'a>,
    ast: ast::Node,
}

impl<'a> JsonAta<'a> {
    pub fn new(expr: &str) -> JsonAtaResult<Self> {
        let root_frame = frame::Frame::new();

        // // TODO: Apply statics to the environment
        // environment.bind("sum", Binding::Function(&sum, "<a<n>:n>"));

        // TODO: Probably could just do this once somewhere to avoid doing it every time

        Ok(Self {
            root_frame,
            ast: parser::parse(expr)?,
        })
    }

    pub fn evaluate(&mut self, input: Option<&JsonValue>) -> JsonAtaResult<Option<JsonValue>> {
        let input = evaluator::Value::new(input);
        let result = evaluator::evaluate(&self.ast, &input, &mut self.root_frame)?;
        Ok(result.into())
    }

    pub fn assign(&mut self, name: &str, value: Binding) {
        self.root_frame.bind(name, value);
    }

    pub fn ast(&self) -> &ast::Node {
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
