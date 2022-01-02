use std::{char, str};

use super::json::Number;
use super::position::Position;
use super::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    End,
    // Double character operators
    Range,
    Bind,
    NotEqual,
    GreaterEqual,
    LessEqual,
    Descendent,
    Apply,
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
    Plus,
    Minus,
    Wildcard,
    ForwardSlash,
    Percent,
    Pipe,
    Equal,
    RightCaret,
    LeftCaret,
    Caret,
    Ampersand,
    Not,
    Tilde,
    // Literal values
    Null,
    Bool(bool),
    Str(String),
    Num(Number),
    // Identifiers
    Name(String),
    Var(String),
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TokenKind::*;
        write!(
            f,
            "{}",
            match self {
                End => "".to_string(),
                Range => "..".to_string(),
                Bind => ":=".to_string(),
                NotEqual => "!=".to_string(),
                GreaterEqual => ">=".to_string(),
                LessEqual => "<=".to_string(),
                Descendent => "**".to_string(),
                Apply => "~>".to_string(),
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
                Plus => "+".to_string(),
                Minus => "-".to_string(),
                Wildcard => "*".to_string(),
                ForwardSlash => "/".to_string(),
                Percent => "%".to_string(),
                Pipe => "|".to_string(),
                Equal => "=".to_string(),
                RightCaret => ">".to_string(),
                LeftCaret => "<".to_string(),
                Caret => "^".to_string(),
                Ampersand => "&".to_string(),
                Not => "!".to_string(),
                Tilde => "~".to_string(),
                Null => "null".to_string(),
                Str(v) => v.to_string(),
                Name(v) => v.to_string(),
                Var(v) => v.to_string(),
                Bool(v) => format!("{}", v),
                Num(v) => format!("{}", v),
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub position: Position,
}

impl Token {
    fn new(kind: TokenKind, position: Position) -> Self {
        Self { kind, position }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}

pub struct Tokenizer {
    position: Position,
    // TODO: Hate that this is a Vec, can't index into an iterator of Char
    source: Vec<char>,
}

impl Tokenizer {
    pub fn new(source: &str) -> Self {
        Self {
            position: Position::default(),
            source: source.chars().collect(),
        }
    }

    fn emit(&self, kind: TokenKind) -> Result<Token> {
        Ok(Token::new(kind, self.position))
    }

    /// Returns the next token in the stream and its position as a tuple
    pub fn next(&mut self, infix: bool) -> Result<Token> {
        use TokenKind::*;

        // Convenience for single character operators
        macro_rules! op1 {
            ($t:tt) => {{
                self.position.advance1();
                break self.emit($t);
            }};
        }

        // Convenience for double character operators
        macro_rules! op2 {
            ($t:tt) => {{
                self.position.advance2();
                break self.emit($t);
            }};
        }

        loop {
            match self.source[self.position.source_pos..] {
                [] => break self.emit(End),
                // Skip whitespace
                [' ' | '\r' | '\n' | '\t' | '\x0b', ..] => {
                    self.position.advance1();
                    continue;
                }
                // Skip comments
                ['/', '*', ..] => {
                    let comment_start = self.position;
                    self.position.advance2();
                    loop {
                        match self.source[self.position.source_pos..] {
                            [] => return Err(Error::UnterminatedComment(comment_start)),
                            ['*', '/', ..] => {
                                self.position.advance2();
                                break;
                            }
                            _ => {
                                self.position.advance1();
                            }
                        }
                    }
                }
                // Regex
                ['/', ..] if !infix => {
                    unimplemented!("TODO: regex scanning is not yet implemented")
                }
                ['.', '.', ..] => op2!(Range),
                [':', '=', ..] => op2!(Bind),
                ['!', '=', ..] => op2!(NotEqual),
                ['>', '=', ..] => op2!(GreaterEqual),
                ['<', '=', ..] => op2!(LessEqual),
                ['*', '*', ..] => op2!(Descendent),
                ['~', '>', ..] => op2!(Apply),
                // Numbers
                ['0'..='9', ..] => {
                    let number_start = self.position.source_pos;
                    self.position.advance1();

                    // TODO(johan): Improve this lexing, it's pretty ordinary and allows all sorts
                    // of invalid stuff
                    loop {
                        match &self.source[self.position.source_pos..] {
                            // Range operator
                            ['.', '.', ..] => break,
                            ['e' | 'E', ..] => {
                                self.position.advance1();
                                loop {
                                    match self.source[self.position.source_pos..] {
                                        ['+' | '-', ..] => {
                                            self.position.advance1();
                                        }
                                        ['0'..='9', ..] => {
                                            self.position.advance1();
                                        }
                                        _ => break,
                                    }
                                }
                            }
                            ['.', ..] => {
                                self.position.advance1();
                                while let ['0'..='9', ..] = self.source[self.position.source_pos..]
                                {
                                    self.position.advance1();
                                }
                            }
                            ['0'..='9', ..] => {
                                self.position.advance1();
                            }
                            _ => break,
                        }
                    }

                    let token = &self.source[number_start..self.position.source_pos];
                    let number: String = token.iter().collect();
                    let number_f64 = number.parse::<f64>().unwrap_or(f64::NAN);
                    if number_f64.is_nan() || number_f64.is_infinite() {
                        break Err(Error::LexedNumberOutOfRange(self.position, number));
                    } else {
                        // FIXME: HACK: This number lexing is so bad, but we want integers to be
                        // integers in the Number instances that are created, so this will do for now.
                        let mantissa = number_f64.floor() as u64;
                        if number_f64 - mantissa as f64 == 0.0 {
                            break self.emit(Num(mantissa.into()));
                        }
                        break self.emit(Num(number_f64.into()));
                    }
                }
                ['.', ..] => op1!(Period),
                ['[', ..] => op1!(LeftBracket),
                [']', ..] => op1!(RightBracket),
                ['{', ..] => op1!(LeftBrace),
                ['}', ..] => op1!(RightBrace),
                ['(', ..] => op1!(LeftParen),
                [')', ..] => op1!(RightParen),
                [',', ..] => op1!(Comma),
                ['@', ..] => op1!(At),
                ['#', ..] => op1!(Hash),
                [';', ..] => op1!(SemiColon),
                [':', ..] => op1!(Colon),
                ['?', ..] => op1!(Question),
                ['+', ..] => op1!(Plus),
                ['-', ..] => op1!(Minus),
                ['*', ..] => op1!(Wildcard),
                ['/', ..] => op1!(ForwardSlash),
                ['%', ..] => op1!(Percent),
                ['|', ..] => op1!(Pipe),
                ['=', ..] => op1!(Equal),
                ['<', ..] => op1!(LeftCaret),
                ['>', ..] => op1!(RightCaret),
                ['^', ..] => op1!(Caret),
                ['&', ..] => op1!(Ampersand),
                ['!', ..] => op1!(Not),
                ['~', ..] => op1!(Tilde),
                // String literals
                [quote_type @ ('\'' | '"'), ..] => {
                    self.position.advance1();
                    let mut string = String::new();
                    let string_start = self.position;
                    break loop {
                        match self.source[self.position.source_pos..] {
                            // End of string missing
                            [] => break Err(Error::UnterminatedStringLiteral(string_start)),
                            // Escape sequence
                            ['\\', escape_char, ..] => {
                                self.position.advance1();

                                match escape_char {
                                    // Basic escape sequence
                                    '"' => {
                                        string.push('"');
                                        self.position.advance1();
                                    }
                                    '\\' => {
                                        string.push('\\');
                                        self.position.advance1();
                                    }
                                    '/' => {
                                        string.push('/');
                                        self.position.advance1();
                                    }
                                    'b' => {
                                        string.push('\x08');
                                        self.position.advance1();
                                    }
                                    'f' => {
                                        string.push('\x0c');
                                        self.position.advance1();
                                    }
                                    'n' => {
                                        string.push('\n');
                                        self.position.advance1();
                                    }
                                    'r' => {
                                        string.push('\r');
                                        self.position.advance1();
                                    }
                                    't' => {
                                        string.push('\t');
                                        self.position.advance1();
                                    }
                                    // Unicode escape sequence
                                    'u' => {
                                        // \u should be followed by 4 hex digits, which needs to
                                        // parsed to a codepoint and then turned into a char to be
                                        // appended
                                        if self.source.len() < self.position.source_pos + 5 {
                                            break Err(Error::InvalidUnicodeEscape(self.position));
                                        }

                                        let chars: &String = &self.source[self.position.source_pos
                                            + 1
                                            ..self.position.source_pos + 5]
                                            .iter()
                                            .cloned()
                                            .collect::<String>();

                                        if let Some(character) = str::from_utf8(chars.as_bytes())
                                            .ok()
                                            .and_then(|octets| u32::from_str_radix(octets, 16).ok())
                                            .and_then(char::from_u32)
                                        {
                                            string.push(character);
                                            self.position.advance(5);
                                        } else {
                                            break Err(Error::InvalidUnicodeEscape(self.position));
                                        }
                                    }
                                    // Invalid escape sequence
                                    c => break Err(Error::unsupported_escape(self.position, c)),
                                }
                            }
                            // Any other char
                            [c, ..] => {
                                // Check for the end of the string
                                if c == quote_type {
                                    self.position.advance1();
                                    break self.emit(Str(string));
                                }

                                // Otherwise add to the string
                                // TODO(johan): This method of building strings byte by byte is
                                // probably slow
                                string.push(c);
                                self.position.advance1();
                                continue;
                            }
                        }
                    };
                }
                // Quoted names (backticks)
                ['`', ..] => {
                    self.position.advance1();
                    // Find the closing backtick and convert to a string
                    match self.source[self.position.source_pos..]
                        .iter()
                        .position(|c| *c == '`')
                        .map(|index| {
                            self.source[self.position.source_pos..self.position.source_pos + index]
                                .iter()
                                .cloned()
                                .collect::<String>()
                        }) {
                        Some(value) => {
                            self.position.advance(value.len() + 1);
                            break self.emit(Name(value));
                        }
                        None => break Err(Error::UnterminatedQuoteProp(self.position)),
                    }
                }
                // Names
                [c, ..] => {
                    let name_start = self.position.source_pos;
                    break loop {
                        match self.source[self.position.source_pos..] {
                            // Match end of source, whitespace characters or a single-char operator
                            // to find the end of the name
                            []
                            | [' ' | '\r' | '\n' | '\t' | '\x0b', ..]
                            | ['.' | '[' | ']' | '{' | '}' | '(' | ')' | ',' | '@' | '#' | ';'
                            | ':' | '?' | '+' | '-' | '*' | '/' | '%' | '|' | '=' | '<' | '>'
                            | '^' | '&' | '!' | '~', ..] => {
                                if c == '$' {
                                    // Variable reference
                                    let name = self.source
                                        [name_start + 1..self.position.source_pos]
                                        .iter()
                                        .cloned()
                                        .collect::<String>();

                                    break self.emit(Var(name));
                                } else {
                                    let name = self.source[name_start..self.position.source_pos]
                                        .iter()
                                        .cloned()
                                        .collect::<String>();

                                    let token = match &name[..] {
                                        "or" => self.emit(Or),
                                        "in" => self.emit(In),
                                        "and" => self.emit(And),
                                        "true" => self.emit(Bool(true)),
                                        "false" => self.emit(Bool(false)),
                                        "null" => self.emit(Null),
                                        _ => self.emit(Name(name)),
                                    };

                                    break token;
                                }
                            }
                            _ => {
                                self.position.advance1();
                            }
                        }
                    };
                } //_ => Err(Box::new(S0204 { position: self.position, token: }))
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
            TokenKind::Plus
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
            TokenKind::Minus
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Wildcard
        ));
    }

    #[test]
    fn strings() {
        let mut tokenizer = Tokenizer::new("\"There's a string here\" 'and another here'");
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Str(s) if s == "There's a string here"
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Str(s) if s == "and another here"
        ));
    }

    #[test]
    fn unicode_escapes() {
        let mut tokenizer = Tokenizer::new("\"\\u2d63\\u2d53\\u2d4d\"");
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Str(s) if s ==  "ⵣⵓⵍ"
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
            TokenKind::Var(s) if s == "one"
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Var(s) if s == "two"
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Var(s) if s == "three"
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
            TokenKind::Bool(true)
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Bool(false)
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Null
        ));
    }

    #[test]
    fn numbers() {
        let mut tokenizer = Tokenizer::new("0 1 0.234 5.678 0e0 1e1 1e-1 1e+1 2.234E-2");
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Num(n) if (f64::from(n) - 0.0_f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Num(n) if (f64::from(n) - 1.0_f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Num(n) if (f64::from(n) - 0.234_f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Num(n) if (f64::from(n) - 5.678_f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Num(n) if (f64::from(n) - 0e0_f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Num(n) if (f64::from(n) - 1e1_f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Num(n) if (f64::from(n) - 1e-1_f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Num(n) if (f64::from(n) - 1e+1_f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            tokenizer.next(false).unwrap().kind,
            TokenKind::Num(n) if (f64::from(n) - 2.234E-2_f64).abs() < f64::EPSILON
        ));
    }
}
