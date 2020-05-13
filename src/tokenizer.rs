use std::fmt;
use std::{char, str};

use crate::error::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Double character operators
    Range,
    Assignment,
    NotEqual,
    GreaterEqual,
    LessEqual,
    DescendantWildcard,
    ChainFunction,
    // Named operators
    Or,
    In,
    And,
    // Single character operators
    Period,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Comma,
    At,
    Hash,
    SemiColon,
    Colon,
    Question,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pipe,
    Equ,
    RightCaret,
    LeftCaret,
    Pow,
    Ampersand,
    Not,
    Tilde,
    // Literal values
    Null,
    Boolean(bool),
    String(String),
    Number(f64),
    // Identifiers
    Name(String),
    Variable(String),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub position: usize,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Range => write!(f, ".."),
            TokenKind::Assignment => write!(f, ":="),
            TokenKind::NotEqual => write!(f, "!="),
            TokenKind::GreaterEqual => write!(f, ">="),
            TokenKind::LessEqual => write!(f, "<="),
            TokenKind::DescendantWildcard => write!(f, "**"),
            TokenKind::ChainFunction => write!(f, "~>"),
            TokenKind::Or => write!(f, "or"),
            TokenKind::In => write!(f, "in"),
            TokenKind::And => write!(f, "and"),
            TokenKind::Period => write!(f, "."),
            TokenKind::LeftBracket => write!(f, "["),
            TokenKind::RightBracket => write!(f, "]"),
            TokenKind::LeftBrace => write!(f, "{{"),
            TokenKind::RightBrace => write!(f, "}}"),
            TokenKind::LeftParen => write!(f, "("),
            TokenKind::RightParen => write!(f, ")"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::At => write!(f, "@"),
            TokenKind::Hash => write!(f, "#"),
            TokenKind::SemiColon => write!(f, ";"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Question => write!(f, "?"),
            TokenKind::Add => write!(f, "+"),
            TokenKind::Sub => write!(f, "-"),
            TokenKind::Mul => write!(f, "*"),
            TokenKind::Div => write!(f, "/"),
            TokenKind::Mod => write!(f, "%"),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::Equ => write!(f, "="),
            TokenKind::RightCaret => write!(f, ">"),
            TokenKind::LeftCaret => write!(f, "<"),
            TokenKind::Pow => write!(f, "^"),
            TokenKind::Ampersand => write!(f, "&"),
            TokenKind::Not => write!(f, "!"),
            TokenKind::Tilde => write!(f, "~"),
            TokenKind::Null => write!(f, "null"),
            TokenKind::Boolean(v) => write!(f, "{}", v),
            TokenKind::String(v) => write!(f, "{}", v),
            TokenKind::Number(v) => write!(f, "{}", v),
            TokenKind::Name(v) => write!(f, "{}", v),
            TokenKind::Variable(v) => write!(f, "{}", v),
        }
    }
}

pub struct Tokenizer<'a> {
    position: usize,
    source: &'a str,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            position: 0,
            source,
        }
    }

    /// Returns the next token in the stream and its position as a tuple
    pub fn next(&mut self, infix: bool) -> Option<Token> {
        loop {
            match self.source.as_bytes()[self.position..] {
                [] => {
                    break None;
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
                    loop {
                        match self.source.as_bytes()[self.position..] {
                            [] => panic!(format!(
                                "{:#?}",
                                Error {
                                    code: "S0106",
                                    position: comment_start,
                                    message: "Comment has no closing tag".to_string()
                                }
                            )),
                            [b'*', b'/', ..] => {
                                self.position += 2;
                                break;
                            }
                            _ => {
                                self.position += 1;
                            }
                        }
                    }
                }
                // Regex
                [b'/', ..] if !infix => unimplemented!("regex scanning is not yet implemented"),
                // Double-dot range operator
                [b'.', b'.', ..] => {
                    self.position += 2;
                    break Some(Token {
                        kind: TokenKind::Range,
                        position: self.position,
                    });
                }
                // := Assignment
                [b':', b'=', ..] => {
                    self.position += 2;
                    break Some(Token {
                        kind: TokenKind::Assignment,
                        position: self.position,
                    });
                }
                // !=
                [b'!', b'=', ..] => {
                    self.position += 2;
                    break Some(Token {
                        kind: TokenKind::NotEqual,
                        position: self.position,
                    });
                }
                // >=
                [b'>', b'=', ..] => {
                    self.position += 2;
                    break Some(Token {
                        kind: TokenKind::GreaterEqual,
                        position: self.position,
                    });
                }
                // <=
                [b'<', b'=', ..] => {
                    self.position += 2;
                    break Some(Token {
                        kind: TokenKind::LessEqual,
                        position: self.position,
                    });
                }
                // ** Descendent wildcard
                [b'*', b'*', ..] => {
                    self.position += 2;
                    break Some(Token {
                        kind: TokenKind::DescendantWildcard,
                        position: self.position,
                    });
                }
                // ~> Chain function
                [b'~', b'>', ..] => {
                    self.position += 2;
                    break Some(Token {
                        kind: TokenKind::ChainFunction,
                        position: self.position,
                    });
                }
                // Numbers
                [b'0'..=b'9', ..] | [b'-', b'0'..=b'9', ..] => {
                    let number_start = self.position;
                    self.position += 1;
                    // TODO(johan): Improve this lexing, it's pretty ordinary and allows all sorts
                    // of invalid stuff
                    break loop {
                        match self.source.as_bytes()[self.position..] {
                            [b'0'..=b'9' | b'.' | b'e' | b'E' | b'-' | b'+', ..] => {
                                self.position += 1;
                            }
                            _ => {
                                let token = &self.source.as_bytes()[number_start..self.position];
                                if let Some(number) = str::from_utf8(token)
                                    .ok()
                                    .and_then(|s| s.parse::<f64>().ok())
                                {
                                    break Some(Token {
                                        kind: TokenKind::Number(number),
                                        position: self.position,
                                    });
                                } else {
                                    panic!(format!(
                                        "{:#?}",
                                        Error {
                                            code: "S0102",
                                            position: self.position,
                                            message: "Number of out range".to_string() // TODO:
                                                                                       //format!(
                                                                                       //    "Number out of range: {}",
                                                                                       //    token as &[char]
                                                                                       //)
                                        }
                                    ))
                                }
                            }
                        }
                    };
                }
                // Single character operators
                [b'.', ..] => {
                    self.position += 1;
                    break Some(Token {
                        kind: TokenKind::Period,
                        position: self.position,
                    });
                }
                [b'[', ..] => {
                    self.position += 1;
                    break Some(Token {
                        kind: TokenKind::LeftBracket,
                        position: self.position,
                    });
                }
                [b']', ..] => {
                    self.position += 1;
                    break Some(Token {
                        kind: TokenKind::RightBracket,
                        position: self.position,
                    });
                }
                [b'{', ..] => {
                    self.position += 1;
                    break Some(Token {
                        kind: TokenKind::LeftBrace,
                        position: self.position,
                    });
                }
                [b'}', ..] => {
                    self.position += 1;
                    break Some(Token {
                        kind: TokenKind::RightBrace,
                        position: self.position,
                    });
                }
                [b'(', ..] => {
                    self.position += 1;
                    break Some(Token {
                        kind: TokenKind::LeftParen,
                        position: self.position,
                    });
                }
                [b')', ..] => {
                    self.position += 1;
                    break Some(Token {
                        kind: TokenKind::RightParen,
                        position: self.position,
                    });
                }
                [b',', ..] => {
                    self.position += 1;
                    break Some(Token {
                        kind: TokenKind::Comma,
                        position: self.position,
                    });
                }
                [b'@', ..] => {
                    self.position += 1;
                    break Some(Token {
                        kind: TokenKind::At,
                        position: self.position,
                    });
                }
                [b'#', ..] => {
                    self.position += 1;
                    break Some(Token {
                        kind: TokenKind::Hash,
                        position: self.position,
                    });
                }
                [b';', ..] => {
                    self.position += 1;
                    break Some(Token {
                        kind: TokenKind::SemiColon,
                        position: self.position,
                    });
                }
                [b':', ..] => {
                    self.position += 1;
                    break Some(Token {
                        kind: TokenKind::Colon,
                        position: self.position,
                    });
                }
                [b'?', ..] => {
                    self.position += 1;
                    break Some(Token {
                        kind: TokenKind::Question,
                        position: self.position,
                    });
                }
                [b'+', ..] => {
                    self.position += 1;
                    break Some(Token {
                        kind: TokenKind::Add,
                        position: self.position,
                    });
                }
                [b'-', ..] => {
                    self.position += 1;
                    break Some(Token {
                        kind: TokenKind::Sub,
                        position: self.position,
                    });
                }
                [b'*', ..] => {
                    self.position += 1;
                    break Some(Token {
                        kind: TokenKind::Mul,
                        position: self.position,
                    });
                }
                [b'/', ..] => {
                    self.position += 1;
                    break Some(Token {
                        kind: TokenKind::Div,
                        position: self.position,
                    });
                }
                [b'%', ..] => {
                    self.position += 1;
                    break Some(Token {
                        kind: TokenKind::Mod,
                        position: self.position,
                    });
                }
                [b'|', ..] => {
                    self.position += 1;
                    break Some(Token {
                        kind: TokenKind::Pipe,
                        position: self.position,
                    });
                }
                [b'=', ..] => {
                    self.position += 1;
                    break Some(Token {
                        kind: TokenKind::Equ,
                        position: self.position,
                    });
                }
                [b'<', ..] => {
                    self.position += 1;
                    break Some(Token {
                        kind: TokenKind::LeftCaret,
                        position: self.position,
                    });
                }
                [b'>', ..] => {
                    self.position += 1;
                    break Some(Token {
                        kind: TokenKind::RightCaret,
                        position: self.position,
                    });
                }
                [b'^', ..] => {
                    self.position += 1;
                    break Some(Token {
                        kind: TokenKind::Pow,
                        position: self.position,
                    });
                }
                [b'&', ..] => {
                    self.position += 1;
                    break Some(Token {
                        kind: TokenKind::Ampersand,
                        position: self.position,
                    });
                }
                [b'!', ..] => {
                    self.position += 1;
                    break Some(Token {
                        kind: TokenKind::Not,
                        position: self.position,
                    });
                }
                [b'~', ..] => {
                    self.position += 1;
                    break Some(Token {
                        kind: TokenKind::Tilde,
                        position: self.position,
                    });
                }
                // String literals
                [quote_type @ (b'\'' | b'"'), ..] => {
                    self.position += 1;
                    let mut string = String::new();
                    let string_start = self.position;
                    break loop {
                        match self.source.as_bytes()[self.position..] {
                            // End of string missing
                            [] => panic!(format!(
                                "{:#?}",
                                Error {
                                    code: "S0101",
                                    position: string_start,
                                    message:
                                        "String literal must be terminated by a matching quote"
                                            .to_string()
                                }
                            )),
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
                                        if let Some(character) = str::from_utf8(
                                            &self.source.as_bytes()
                                                [self.position + 1..self.position + 5],
                                        )
                                        .ok()
                                        .and_then(|octets| u32::from_str_radix(octets, 16).ok())
                                        .and_then(char::from_u32)
                                        {
                                            string.push(character);
                                            self.position += 5;
                                        } else {
                                            panic!(format!(
                                                "{:#?}",
                                                Error {
                                                    code: "S0104",
                                                    position: self.position,
                                                    message: "The escape sequence \\u must be followed by 4 hex digits".to_string()
                                                }
                                            ));
                                        }
                                    }
                                    // Invalid escape sequence
                                    c => {
                                        panic!(format!(
                                            "{:#?}",
                                            Error {
                                                code: "S0104",
                                                position: self.position,
                                                message: format!(
                                                    "Unsupported escape sequence: \\{}",
                                                    c as char
                                                )
                                            }
                                        ));
                                    }
                                }
                            }
                            // Any other char
                            [c, ..] => {
                                // Check for the end of the string
                                if c == quote_type {
                                    self.position += 1;
                                    break Some(Token {
                                        kind: TokenKind::String(string),
                                        position: self.position,
                                    });
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
                }
                // Quoted names (backticks)
                [b'`', ..] => {
                    self.position += 1;
                    // Find the closing backtick and convert to a string
                    match self.source.as_bytes()[self.position..]
                        .iter()
                        .position(|byte| *byte == b'`')
                        .and_then(|index| {
                            String::from_utf8(
                                self.source.as_bytes()[self.position..self.position + index]
                                    .to_vec(),
                            )
                            .ok()
                        }) {
                        Some(value) => {
                            self.position += value.len() + 1;
                            break Some(Token {
                                kind: TokenKind::Name(value),
                                position: self.position,
                            });
                        }
                        None => panic!(format!(
                            "{:#?}",
                            Error {
                                code: "S0105",
                                position: self.position,
                                message:
                                    "Quoted property name must be terminated with a backquote (`)"
                                        .to_string()
                            }
                        )),
                    }
                }
                // Names
                [c, ..] => {
                    let name_start = self.position;
                    break loop {
                        match self.source.as_bytes()[self.position..] {
                            // Match end of source, whitespace characters or a single-char operator
                            // to find the end of the name
                            []
                            | [b' ' | b'\r' | b'\n' | b'\t' | b'\x0b', ..]
                            | [b'.' | b'[' | b']' | b'{' | b'}' | b'(' | b')' | b',' | b'@' | b'#'
                            | b';' | b':' | b'?' | b'+' | b'-' | b'*' | b'/' | b'%' | b'|'
                            | b'=' | b'<' | b'>' | b'^' | b'&' | b'!' | b'~', ..] => {
                                if c == b'$' {
                                    // Variable reference
                                    // TODO(johan): This could fail to unwrap
                                    let name = String::from_utf8(
                                        self.source.as_bytes()[name_start + 1..self.position]
                                            .to_vec(),
                                    )
                                    .unwrap();

                                    break Some(Token {
                                        kind: TokenKind::Variable(name),
                                        position: self.position,
                                    });
                                } else {
                                    // TODO(johan): This could fail to unwrap
                                    let name = String::from_utf8(
                                        self.source.as_bytes()[name_start..self.position].to_vec(),
                                    )
                                    .unwrap();

                                    let token = match &name[..] {
                                        "or" => Token {
                                            kind: TokenKind::Or,
                                            position: self.position,
                                        },
                                        "in" => Token {
                                            kind: TokenKind::In,
                                            position: self.position,
                                        },
                                        "and" => Token {
                                            kind: TokenKind::And,
                                            position: self.position,
                                        },
                                        "true" => Token {
                                            kind: TokenKind::Boolean(true),
                                            position: self.position,
                                        },
                                        "false" => Token {
                                            kind: TokenKind::Boolean(false),
                                            position: self.position,
                                        },
                                        "null" => Token {
                                            kind: TokenKind::Null,
                                            position: self.position,
                                        },
                                        _ => Token {
                                            kind: TokenKind::Name(name),
                                            position: self.position,
                                        },
                                    };

                                    break Some(token);
                                }
                            }
                            _ => {
                                self.position += 1;
                            }
                        }
                    };
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operators() {
        let mut tokenizer = Tokenizer::new("  @   # +  <=>= /* This is a comment */ ? -*");
        assert!(matches!(tokenizer.next(false).unwrap().kind, TokenKind::At));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Hash
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Add
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::LessEqual
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::GreaterEqual
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Question
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Sub
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Mul
        ));
    }

    #[test]
    fn strings() {
        let mut tokenizer = Tokenizer::new("\"There's a string here\" 'and another here'");
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::String(s) if s == "There's a string here"
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::String(s) if s == "and another here"
        ));
    }

    #[test]
    fn unicode_escapes() {
        let mut tokenizer = Tokenizer::new("\"\\u2d63\\u2d53\\u2d4d\"");
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::String(s) if s ==  "ⵣⵓⵍ"
        ));
    }

    #[test]
    fn backtick_names() {
        let mut tokenizer = Tokenizer::new("  `hello`    `world`");
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Name(s) if s == "hello"
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Name(s) if s == "world"
        ));
    }

    #[test]
    fn variables() {
        let mut tokenizer = Tokenizer::new("  $one   $two   $three  ");
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Variable(s) if s == "one"
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Variable(s) if s == "two"
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Variable(s) if s == "three"
        ));
    }

    #[test]
    fn name_operators() {
        let mut tokenizer = Tokenizer::new("or in and");
        assert!(matches!(tokenizer.next(false).unwrap().kind, TokenKind::Or));
        assert!(matches!(tokenizer.next(false).unwrap().kind, TokenKind::In));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::And
        ));
    }

    #[test]
    fn values() {
        let mut tokenizer = Tokenizer::new("true false null");
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Boolean(true)
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Boolean(false)
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Null
        ));
    }

    #[test]
    fn numbers() {
        let mut tokenizer =
            Tokenizer::new("0 1 0.234 5.678 -0 -1 -0.234 -5.678 0e0 1e1 1e-1 1e+1 -2.234E-2");
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Number(n) if (n - 0.0 as f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Number(n) if (n - 1.0 as f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Number(n) if (n - 0.234 as f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Number(n) if (n - 5.678 as f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Number(n) if (n - -0.0 as f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Number(n) if (n - -1.0 as f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Number(n) if (n - -0.234 as f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Number(n) if (n - -5.678 as f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Number(n) if (n - 0e0 as f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Number(n) if (n - 1e1 as f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Number(n) if (n - 1e-1 as f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Number(n) if (n - 1e+1 as f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Number(n) if (n - -2.234E-2 as f64).abs() < f64::EPSILON
        ));
    }
}
