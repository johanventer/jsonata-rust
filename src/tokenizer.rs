use std::fmt;
use std::{char, str};

use crate::error::*;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    End,
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
    Caret,
    Ampersand,
    Not,
    Tilde,
    // Literal values
    Null,
    Boolean(bool),
    Str(String),
    Number(f64),
    // Identifiers
    Name(String),
    Variable(String),
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TokenKind::*;
        let value = match self {
            End => "".to_string(),
            Range => "..".to_string(),
            Assignment => ":=".to_string(),
            NotEqual => "!=".to_string(),
            GreaterEqual => ">=".to_string(),
            LessEqual => "<=".to_string(),
            DescendantWildcard => "**".to_string(),
            ChainFunction => "~>".to_string(),
            Or => "or".to_string(),
            In => "in".to_string(),
            And => "and".to_string(),
            Period => ".".to_string(),
            LeftBracket => "[".to_string(),
            RightBracket => "]".to_string(),
            LeftBrace => "{".to_string(),
            RightBrace => "}".to_string(),
            LeftParen => "(".to_string(),
            RightParen => ")".to_string(),
            Comma => ",".to_string(),
            At => "@".to_string(),
            Hash => "#".to_string(),
            SemiColon => ";".to_string(),
            Colon => ":".to_string(),
            Question => "?".to_string(),
            Add => "+".to_string(),
            Sub => "-".to_string(),
            Mul => "*".to_string(),
            Div => "/".to_string(),
            Mod => "%".to_string(),
            Pipe => "|".to_string(),
            Equ => "=".to_string(),
            RightCaret => ">".to_string(),
            LeftCaret => "<".to_string(),
            Caret => "^".to_string(),
            Ampersand => "&".to_string(),
            Not => "!".to_string(),
            Tilde => "~".to_string(),
            Null => "null".to_string(),
            Str(v) => v.to_string(),
            Name(v) => v.to_string(),
            Variable(v) => v.to_string(),
            Boolean(v) => format!("{}", v),
            Number(v) => format!("{}", v),
        };
        write!(f, "{}", value)
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub position: usize,
}

impl Token {
    fn new(kind: TokenKind, position: usize) -> Self {
        Self { kind, position }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
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

    fn emit(&self, kind: TokenKind) -> Token {
        Token::new(kind, self.position)
    }

    /// Returns the next token in the stream and its position as a tuple
    pub fn next(&mut self, infix: bool) -> Token {
        use TokenKind::*;

        loop {
            match self.source.as_bytes()[self.position..] {
                [] => break self.emit(End),
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
                            [] => error!(s0106, comment_start),
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
                    break self.emit(Range);
                }
                // := Assignment
                [b':', b'=', ..] => {
                    self.position += 2;
                    break self.emit(Assignment);
                }
                // !=
                [b'!', b'=', ..] => {
                    self.position += 2;
                    break self.emit(NotEqual);
                }
                // >=
                [b'>', b'=', ..] => {
                    self.position += 2;
                    break self.emit(GreaterEqual);
                }
                // <=
                [b'<', b'=', ..] => {
                    self.position += 2;
                    break self.emit(LessEqual);
                }
                // ** Descendent wildcard
                [b'*', b'*', ..] => {
                    self.position += 2;
                    break self.emit(DescendantWildcard);
                }
                // ~> Chain function
                [b'~', b'>', ..] => {
                    self.position += 2;
                    break self.emit(ChainFunction);
                }
                // Numbers
                [b'0'..=b'9', ..] => {
                    let number_start = self.position;
                    self.position += 1;
                    // TODO(johan): Improve this lexing, it's pretty ordinary and allows all sorts
                    // of invalid stuff
                    loop {
                        match self.source.as_bytes()[self.position..] {
                            // Range operator
                            [b'.', b'.', ..] => break,
                            [b'0'..=b'9' | b'.' | b'e' | b'E' | b'-' | b'+', ..] => {
                                self.position += 1;
                            }
                            _ => break,
                        }
                    }

                    let token = &self.source.as_bytes()[number_start..self.position];
                    if let Some(number) = str::from_utf8(token)
                        .ok()
                        .and_then(|s| s.parse::<f64>().ok())
                    {
                        break self.emit(Number(number));
                    } else {
                        error!(s0102, self.position, token);
                    }
                }
                // Single character operators
                [b'.', ..] => {
                    self.position += 1;
                    break self.emit(Period);
                }
                [b'[', ..] => {
                    self.position += 1;
                    break self.emit(LeftBracket);
                }
                [b']', ..] => {
                    self.position += 1;
                    break self.emit(RightBracket);
                }
                [b'{', ..] => {
                    self.position += 1;
                    break self.emit(LeftBrace);
                }
                [b'}', ..] => {
                    self.position += 1;
                    break self.emit(RightBrace);
                }
                [b'(', ..] => {
                    self.position += 1;
                    break self.emit(LeftParen);
                }
                [b')', ..] => {
                    self.position += 1;
                    break self.emit(RightParen);
                }
                [b',', ..] => {
                    self.position += 1;
                    break self.emit(Comma);
                }
                [b'@', ..] => {
                    self.position += 1;
                    break self.emit(At);
                }
                [b'#', ..] => {
                    self.position += 1;
                    break self.emit(Hash);
                }
                [b';', ..] => {
                    self.position += 1;
                    break self.emit(SemiColon);
                }
                [b':', ..] => {
                    self.position += 1;
                    break self.emit(Colon);
                }
                [b'?', ..] => {
                    self.position += 1;
                    break self.emit(Question);
                }
                [b'+', ..] => {
                    self.position += 1;
                    break self.emit(Add);
                }
                [b'-', ..] => {
                    self.position += 1;
                    break self.emit(Sub);
                }
                [b'*', ..] => {
                    self.position += 1;
                    break self.emit(Mul);
                }
                [b'/', ..] => {
                    self.position += 1;
                    break self.emit(Div);
                }
                [b'%', ..] => {
                    self.position += 1;
                    break self.emit(Mod);
                }
                [b'|', ..] => {
                    self.position += 1;
                    break self.emit(Pipe);
                }
                [b'=', ..] => {
                    self.position += 1;
                    break self.emit(Equ);
                }
                [b'<', ..] => {
                    self.position += 1;
                    break self.emit(LeftCaret);
                }
                [b'>', ..] => {
                    self.position += 1;
                    break self.emit(RightCaret);
                }
                [b'^', ..] => {
                    self.position += 1;
                    break self.emit(Caret);
                }
                [b'&', ..] => {
                    self.position += 1;
                    break self.emit(Ampersand);
                }
                [b'!', ..] => {
                    self.position += 1;
                    break self.emit(Not);
                }
                [b'~', ..] => {
                    self.position += 1;
                    break self.emit(Tilde);
                }
                // String literals
                [quote_type @ (b'\'' | b'"'), ..] => {
                    self.position += 1;
                    let mut string = String::new();
                    let string_start = self.position;
                    break loop {
                        match self.source.as_bytes()[self.position..] {
                            // End of string missing
                            [] => error!(s0101, string_start),
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
                                            error!(s0104, self.position)
                                        }
                                    }
                                    // Invalid escape sequence
                                    c => error!(s0103, self.position, c),
                                }
                            }
                            // Any other char
                            [c, ..] => {
                                // Check for the end of the string
                                if c == quote_type {
                                    self.position += 1;
                                    break self.emit(Str(string));
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
                            break self.emit(Name(value));
                        }
                        None => error!(s0105, self.position),
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

                                    break self.emit(Variable(name));
                                } else {
                                    // TODO(johan): This could fail to unwrap
                                    let name = String::from_utf8(
                                        self.source.as_bytes()[name_start..self.position].to_vec(),
                                    )
                                    .unwrap();

                                    let token = match &name[..] {
                                        "or" => self.emit(Or),
                                        "in" => self.emit(In),
                                        "and" => self.emit(And),
                                        "true" => self.emit(Boolean(true)),
                                        "false" => self.emit(Boolean(false)),
                                        "null" => self.emit(Null),
                                        _ => self.emit(Name(name)),
                                    };

                                    break token;
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
        assert!(matches!(tokenizer.next(false).kind, TokenKind::At));
        assert!(matches!(tokenizer.next(false).kind, TokenKind::Hash));
        assert!(matches!(tokenizer.next(false).kind, TokenKind::Add));
        assert!(matches!(tokenizer.next(false).kind, TokenKind::LessEqual));
        assert!(matches!(
            tokenizer.next(false).kind,
            TokenKind::GreaterEqual
        ));
        assert!(matches!(tokenizer.next(false).kind, TokenKind::Question));
        assert!(matches!(tokenizer.next(false).kind, TokenKind::Sub));
        assert!(matches!(tokenizer.next(false).kind, TokenKind::Mul));
    }

    #[test]
    fn strings() {
        let mut tokenizer = Tokenizer::new("\"There's a string here\" 'and another here'");
        assert!(matches!(
            tokenizer.next(false).kind,
            TokenKind::Str(s) if s == "There's a string here"
        ));
        assert!(matches!(
            tokenizer.next(false).kind,
            TokenKind::Str(s) if s == "and another here"
        ));
    }

    #[test]
    fn unicode_escapes() {
        let mut tokenizer = Tokenizer::new("\"\\u2d63\\u2d53\\u2d4d\"");
        assert!(matches!(
            tokenizer.next(false).kind,
            TokenKind::Str(s) if s ==  "ⵣⵓⵍ"
        ));
    }

    #[test]
    fn backtick_names() {
        let mut tokenizer = Tokenizer::new("  `hello`    `world`");
        assert!(matches!(
            tokenizer.next(false).kind,
            TokenKind::Name(s) if s == "hello"
        ));
        assert!(matches!(
            tokenizer.next(false).kind,
            TokenKind::Name(s) if s == "world"
        ));
    }

    #[test]
    fn variables() {
        let mut tokenizer = Tokenizer::new("  $one   $two   $three  ");
        assert!(matches!(
            tokenizer.next(false).kind,
            TokenKind::Variable(s) if s == "one"
        ));
        assert!(matches!(
            tokenizer.next(false).kind,
            TokenKind::Variable(s) if s == "two"
        ));
        assert!(matches!(
            tokenizer.next(false).kind,
            TokenKind::Variable(s) if s == "three"
        ));
    }

    #[test]
    fn name_operators() {
        let mut tokenizer = Tokenizer::new("or in and");
        assert!(matches!(tokenizer.next(false).kind, TokenKind::Or));
        assert!(matches!(tokenizer.next(false).kind, TokenKind::In));
        assert!(matches!(tokenizer.next(false).kind, TokenKind::And));
    }

    #[test]
    fn values() {
        let mut tokenizer = Tokenizer::new("true false null");
        assert!(matches!(
            tokenizer.next(false).kind,
            TokenKind::Boolean(true)
        ));
        assert!(matches!(
            tokenizer.next(false).kind,
            TokenKind::Boolean(false)
        ));
        assert!(matches!(tokenizer.next(false).kind, TokenKind::Null));
    }

    #[test]
    fn numbers() {
        let mut tokenizer = Tokenizer::new("0 1 0.234 5.678 0e0 1e1 1e-1 1e+1 2.234E-2");
        assert!(matches!(
            tokenizer.next(false).kind,
            TokenKind::Number(n) if (n - 0.0 as f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).kind,
            TokenKind::Number(n) if (n - 1.0 as f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).kind,
            TokenKind::Number(n) if (n - 0.234 as f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).kind,
            TokenKind::Number(n) if (n - 5.678 as f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).kind,
            TokenKind::Number(n) if (n - 0e0 as f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).kind,
            TokenKind::Number(n) if (n - 1e1 as f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).kind,
            TokenKind::Number(n) if (n - 1e-1 as f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).kind,
            TokenKind::Number(n) if (n - 1e+1 as f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).kind,
            TokenKind::Number(n) if (n - 2.234E-2 as f64).abs() < f64::EPSILON
        ));
    }
}
