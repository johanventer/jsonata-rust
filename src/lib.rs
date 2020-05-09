#![feature(or_patterns)]

#[macro_use]
extern crate lazy_static;

use chrono::{DateTime, Utc};
use json::JsonValue;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Error {
    code: &'static str,
    // stack,
    position: usize,
}

/// A binding in a stack frame
pub enum Binding<'a> {
    Variable(JsonValue),
    Function(&'a dyn Fn(Vec<&JsonValue>) -> JsonValue, &'a str),
}

impl Binding<'_> {
    pub fn as_var(&self) -> &JsonValue {
        match self {
            Binding::Variable(value) => &value,
            _ => panic!("Binding is not a variable"),
        }
    }

    pub fn as_func(&self) -> &dyn Fn(Vec<&JsonValue>) -> JsonValue {
        match self {
            Binding::Function(func, _) => func,
            _ => panic!("Binding is not a function"),
        }
    }
}

fn sum(args: Vec<&JsonValue>) -> JsonValue {
    json::from("todo")
}

/// A stack frame of the expression evaluation
struct Frame<'a> {
    /// Stores the bindings for the frame
    bindings: HashMap<String, Binding<'a>>,

    /// The parent frame of this frame
    parent_frame: Option<&'a Frame<'a>>,

    /// The local timestamp in this frame
    timestamp: DateTime<Utc>,
    // TODO: async, global
}

impl<'a> Frame<'a> {
    /// Creates a new empty frame
    fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent_frame: None,
            timestamp: Utc::now(),
        }
    }

    /// Creates a new empty frame, with a parent frame for lookups
    fn new_from(parent_frame: &'a Frame<'a>) -> Self {
        Self {
            bindings: HashMap::new(),
            parent_frame: Some(parent_frame),
            timestamp: parent_frame.timestamp.clone(),
        }
    }

    /// Bind a value to a name in a frame
    fn bind(&mut self, name: &str, value: Binding<'a>) {
        &self.bindings.insert(name.to_string(), value);
    }

    /// Lookup a value by name in a frame
    fn lookup(&self, name: &str) -> Option<&Binding> {
        match &self.bindings.get(name) {
            Some(value) => Some(value),
            None => match &self.parent_frame {
                Some(parent) => parent.lookup(name),
                None => None,
            },
        }
    }
}

lazy_static! {
    static ref OPERATORS: HashMap<&'static str, u8> = [
        (".", 75),
        ("[", 80),
        ("]", 0),
        ("{", 70),
        ("}", 0),
        ("(", 80),
        (")", 0),
        (",", 0),
        ("@", 80),
        ("#", 80),
        (";", 80),
        (",", 80),
        ("?", 20),
        ("+", 50),
        ("-", 50),
        ("*", 60),
        ("/", 60),
        ("%", 60),
        ("|", 20),
        ("=", 40),
        ("<", 40),
        (">", 40),
        ("^", 40),
        ("**", 60),
        ("..", 20),
        (",=", 10),
        ("!=", 40),
        ("<=", 40),
        (">=", 40),
        ("~>", 40),
        ("and", 30),
        ("or", 25),
        ("in", 40),
        ("&", 50),
        ("!", 0),
        ("~", 0)
    ]
    .iter()
    .copied()
    .collect();
}

#[derive(Debug)]
enum TokenKind {
    End,
    Operator,
    Regex,
    String,
}

#[derive(Debug)]
pub struct Token {
    kind: TokenKind,
    value: String,
    position: usize,
}

pub struct Tokenizer<'a> {
    position: usize,
    source: &'a str,
    length: usize,
}

impl<'a> Tokenizer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            position: 0,
            length: source.len(),
            source,
        }
    }
    fn next_token(&mut self) -> Result<Token, Error> {
        let mut prefix = false;

        loop {
            match self.source.as_bytes()[self.position..] {
                [] => {
                    break Ok(Token {
                        kind: TokenKind::End,
                        value: "".to_string(),
                        position: self.position,
                    });
                }
                // Skip whitespace
                [b' ' | b'\r' | b'\n' | b'\t' | b'\x0b', ..] => {
                    self.position += 1;
                    continue;
                }
                // Skip comments
                [b'/', b'*', ..] => {
                    let comment_start = self.position;
                    self.position += 2;
                    let result: Result<(), Error> = loop {
                        match self.source.as_bytes()[self.position..] {
                            [] => {
                                break Err(Error {
                                    code: "S0106",
                                    position: comment_start,
                                });
                            }
                            [b'*', b'/', ..] => {
                                self.position += 2;
                                break Ok(());
                            }
                            _ => {
                                self.position += 1;
                            }
                        }
                    };
                    if let Err(e) = result {
                        break Err(e);
                    }
                }
                // Regex
                [b'/', ..] if !prefix => unimplemented!("regex scanning is not yet implemented"),
                // Double-dot range operator
                [b'.', b'.', ..] => {
                    self.position += 2;
                    break Ok(Token {
                        kind: TokenKind::Operator,
                        value: "..".to_string(),
                        position: self.position,
                    });
                }
                // := Assignment
                [b':', b'=', ..] => {
                    self.position += 2;
                    break Ok(Token {
                        kind: TokenKind::Operator,
                        value: ":=".to_string(),
                        position: self.position,
                    });
                }
                // !=
                [b'!', b'=', ..] => {
                    self.position += 2;
                    break Ok(Token {
                        kind: TokenKind::Operator,
                        value: "!=".to_string(),
                        position: self.position,
                    });
                }
                // >=
                [b'>', b'=', ..] => {
                    self.position += 2;
                    break Ok(Token {
                        kind: TokenKind::Operator,
                        value: ">=".to_string(),
                        position: self.position,
                    });
                }
                // <=
                [b'<', b'=', ..] => {
                    self.position += 2;
                    break Ok(Token {
                        kind: TokenKind::Operator,
                        value: "<=".to_string(),
                        position: self.position,
                    });
                }
                // ** Descendent wildcard
                [b'*', b'*', ..] => {
                    self.position += 2;
                    break Ok(Token {
                        kind: TokenKind::Operator,
                        value: "**".to_string(),
                        position: self.position,
                    });
                }
                // ~> Chain function
                [b'~', b'>', ..] => {
                    self.position += 2;
                    break Ok(Token {
                        kind: TokenKind::Operator,
                        value: "~>".to_string(),
                        position: self.position,
                    });
                }
                // Single character operators
                [c @ (b'.' | b'[' | b']' | b'{' | b'}' | b'(' | b')' | b',' | b'@' | b'#'
                | b';' | b':' | b'?' | b'+' | b'-' | b'*' | b'/' | b'%' | b'|' | b'='
                | b'<' | b'>' | b'^' | b'&' | b'!' | b'~'), ..] => {
                    self.position += 1;
                    break Ok(Token {
                        kind: TokenKind::Operator,
                        value: String::from_utf8(vec![c]).unwrap(),
                        position: self.position,
                    });
                }
                // String literals
                [quote_type @ (b'\'' | b'"'), ..] => {
                    self.position += 1;
                    let mut string = String::new();
                    let string_start = self.position;
                    let result: Result<String, Error> = loop {
                        match self.source.as_bytes()[self.position..] {
                            // End of string missing
                            [] => {
                                break Err(Error {
                                    code: "S0101",
                                    position: string_start,
                                });
                            }
                            // Escape sequence
                            [b'\\', escape_char, ..] => {
                                self.position += 1;

                                match escape_char {
                                    // Basic escape sequence
                                    b'"' => string.push_str("\""),
                                    b'\\' => string.push_str("\\"),
                                    b'/' => string.push_str("/"),
                                    b'b' => string.push_str("\x08"),
                                    b'f' => string.push_str("\x0c"),
                                    b'n' => string.push_str("\n"),
                                    b'r' => string.push_str("\r"),
                                    b't' => string.push_str("\t"),
                                    // Unicode escape sequence
                                    b'u' => {
                                        // \u should be followed by 4 hex digits, which needs to
                                        // parsed to a codepoint and then turned into a char to be
                                        // appended
                                        if let Some(character) = std::str::from_utf8(
                                            &self.source.as_bytes()
                                                [self.position + 1..self.position + 5],
                                        )
                                        .ok()
                                        .and_then(|octets| u32::from_str_radix(octets, 16).ok())
                                        .and_then(|codepoint| std::char::from_u32(codepoint))
                                        {
                                            string.push(character);
                                            self.position += 5;
                                        } else {
                                            break Err(Error {
                                                code: "S0104",
                                                position: self.position,
                                            });
                                        }
                                    }
                                    // Invalid escape sequence
                                    _ => {
                                        break Err(Error {
                                            code: "S0103",
                                            position: self.position,
                                        });
                                    }
                                }
                            }
                            // Any other char
                            [c, ..] => {
                                // Check for the end of the string
                                if c == quote_type {
                                    self.position += 1;
                                    break Ok(string);
                                }

                                // Otherwise add to the string
                                // TODO(johan): This method of building strings byte by byte is
                                // probably slow
                                string.push_str(&String::from_utf8(vec![c]).unwrap());
                                self.position += 1;
                                continue;
                            }
                        }
                    };

                    match result {
                        Err(e) => break Err(e),
                        Ok(value) => {
                            break Ok(Token {
                                kind: TokenKind::String,
                                value,
                                position: self.position,
                            })
                        }
                    }
                }
                // Numbers
                // var numregex = /^-?(0|([1-9][0-9]*))(\.[0-9]+)?([Ee][-+]?[0-9]+)?/;
                // Quoted names (backticks)
                // Names
                [c, ..] => {
                    unreachable!(format!("Unknown token: '{}'", c as char));
                }
            }
        }
    }
}

impl Iterator for Tokenizer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(token) = self.next_token().ok() {
            return match token.kind {
                TokenKind::End => None,
                _ => Some(token),
            };
        }
        None
    }
}

pub struct Parser {}

pub struct JsonAta<'a> {
    expr: String,
    environment: Frame<'a>,
}

impl<'a> JsonAta<'a> {
    pub fn new(expr: &'a str) -> Self {
        // Parse the AST

        let mut environment = Frame::new();

        // TODO: Apply statics to the environment
        environment.bind("sum", Binding::Function(&sum, "<a<n>:n>"));

        // TODO: Probably could just do this once somewhere to avoid doing it every time

        Self {
            expr: expr.to_string(),
            environment,
        }
    }

    pub fn evaluate(&self, input: &str, bindings: Vec<Binding>) -> JsonValue {
        json::from("TODO")
    }

    pub fn assign(&mut self, name: &str, value: Binding<'a>) {
        self.environment.bind(name, value);
    }

    pub fn ast() {
        // TODO
    }

    pub fn errors() {
        // TODO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_and_lookup() {
        let mut frame = Frame::new();
        frame.bind("bool", Binding::Variable(json::from(true)));
        frame.bind("number", Binding::Variable(json::from(42)));
        frame.bind("string", Binding::Variable(json::from("hello")));
        frame.bind("array", Binding::Variable(json::from(vec![1, 2, 3])));
        frame.bind("none", Binding::Variable(json::Null));

        assert!(frame.lookup("not_there").is_none());

        assert!(frame.lookup("bool").unwrap().as_var().is_boolean());
        assert!(frame.lookup("number").unwrap().as_var().is_number());
        assert!(frame.lookup("string").unwrap().as_var().is_string());
        assert!(frame.lookup("array").unwrap().as_var().is_array());
        assert!(frame.lookup("none").unwrap().as_var().is_empty());

        assert_eq!(
            frame.lookup("bool").unwrap().as_var().as_bool().unwrap(),
            true
        );
        assert_eq!(
            frame
                .lookup("number")
                .unwrap()
                .as_var()
                .as_number()
                .unwrap(),
            42
        );
        assert_eq!(
            frame.lookup("string").unwrap().as_var().as_str().unwrap(),
            "hello"
        );

        let array = frame.lookup("array");
        assert_eq!(array.unwrap().as_var().len(), 3);
    }

    #[test]
    fn lookup_through_parent() {
        let mut parent = Frame::new();
        parent.bind("value", Binding::Variable(json::from(42)));
        let child = Frame::new_from(&parent);
        assert_eq!(
            child.lookup("value").unwrap().as_var().as_number().unwrap(),
            42
        );
    }

    #[test]
    fn fn_binding() {
        let mut frame = Frame::new();
        frame.bind("sum", Binding::Function(&sum, ""));
        let sum = frame.lookup("sum").unwrap().as_func();
        assert_eq!(sum(vec![]).as_str().unwrap(), "todo");
    }

    #[test]
    fn basic_tokenizer() {
        let source = "  @   # +  <=>= /* This is a comment */ ? +-*";
        let tokenizer = Tokenizer::new(&source);
        let tokens: Vec<String> = tokenizer.map(|token| token.value).collect();
        assert_eq!(tokens, vec!["@", "#", "+", "<=", ">=", "?", "+", "-", "*"]);
    }

    #[test]
    fn string_tokens() {
        let source = "# @        *    \"There's a string here\" /* comment */ + 'and another here'";
        let tokenizer = Tokenizer::new(&source);
        let tokens: Vec<String> = tokenizer.map(|token| token.value).collect();
        assert_eq!(
            tokens,
            vec![
                "#",
                "@",
                "*",
                "There's a string here",
                "+",
                "and another here"
            ]
        );
    }

    #[test]
    fn unicode_escapes() {
        let source = "\"\\u2d63\\u2d53\\u2d4d\"";
        let tokenizer = Tokenizer::new(&source);
        let tokens: Vec<String> = tokenizer.map(|token| token.value).collect();
        assert_eq!(tokens, vec!["ⵣⵓⵍ"]);
    }
}
