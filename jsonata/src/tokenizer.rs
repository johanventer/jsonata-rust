use std::char::decode_utf16;
use std::str::Chars;
use std::{char, str};

use jsonata_errors::{Error, Result};

use super::json::Number;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Token indicating the end of the token stream
    End,

    // Tokens that are ignored (but could be used for concrete trees later)
    Whitespace,
    Comment,

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
    QuestionMark,
    Plus,
    Minus,
    Asterisk,
    ForwardSlash,
    PercentSign,
    Pipe,
    Equal,
    RightAngleBracket,
    LeftAngleBracket,
    Caret,
    Ampersand,
    ExclamationMark,
    Tilde,

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
        match self {
            TokenKind::End => write!(f, "(end)"),
            TokenKind::Whitespace => write!(f, "(whitespace)"),
            TokenKind::Comment => write!(f, "(comment"),
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
            TokenKind::QuestionMark => write!(f, "?"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Asterisk => write!(f, "*"),
            TokenKind::ForwardSlash => write!(f, "/"),
            TokenKind::PercentSign => write!(f, "%"),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::Equal => write!(f, "="),
            TokenKind::RightAngleBracket => write!(f, ">"),
            TokenKind::LeftAngleBracket => write!(f, "<"),
            TokenKind::Caret => write!(f, "^"),
            TokenKind::Ampersand => write!(f, "&"),
            TokenKind::ExclamationMark => write!(f, "!"),
            TokenKind::Tilde => write!(f, "~"),
            TokenKind::Range => write!(f, ".."),
            TokenKind::Bind => write!(f, ":="),
            TokenKind::NotEqual => write!(f, "!="),
            TokenKind::GreaterEqual => write!(f, ">="),
            TokenKind::LessEqual => write!(f, "<="),
            TokenKind::Descendent => write!(f, "**"),
            TokenKind::Apply => write!(f, "~>"),
            TokenKind::Or => write!(f, "or"),
            TokenKind::In => write!(f, "in"),
            TokenKind::And => write!(f, "and"),
            TokenKind::Null => write!(f, "null"),
            TokenKind::Bool(v) => write!(f, "{}", v),
            TokenKind::Str(v) => write!(f, "\"{}\"", v),
            TokenKind::Num(v) => write!(f, "{}", v),
            TokenKind::Name(v) => write!(f, "{}", v),
            TokenKind::Var(v) => write!(f, "${}", v),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub char_index: usize,
    pub byte_index: usize,
    pub len: usize,
}

/// Tokenizer for JSONata syntax.
#[derive(Debug)]
pub struct Tokenizer<'a> {
    input: &'a str,
    chars: Chars<'a>,

    /// Internal buffer used for building strings
    buffer: Vec<char>,

    /// The current bytes index into the input
    byte_index: usize,

    /// The current char index into the input (used for errors)
    char_index: usize,

    /// The starting byte index of the current token being generated (used for errors)
    start_byte_index: usize,

    /// The starting char index of the current token being generated (used for errors)
    start_char_index: usize,
}

const NULL: char = '\0';

/// The mantissa in a json::Number is a u64, but we know that f64 has 53 bits for mantissa
/// (52 in the mantissa field, and the implict 1 at the start), so at this point we have
/// already blown the range of f64, so it's just to prevent u64 overflow.
const MAX_PRECISION: u64 = u64::pow(2, 59);

#[inline]
fn is_whitespace(c: char) -> bool {
    matches!(
        c,
        '\u{0009}'   // \t
        | '\u{000A}' // \n
        | '\u{000B}' // vertical tab
        | '\u{000C}' // form feed
        | '\u{000D}' // \r
        | '\u{0020}' // space
    )
}

#[inline]
fn is_name_start(c: char) -> bool {
    c.is_alphabetic() || c == '$'
}

#[inline]
fn is_operator(c: char) -> bool {
    matches!(
        c,
        '.' | '['
            | ']'
            | '{'
            | '}'
            | '('
            | ')'
            | ','
            | '@'
            | '#'
            | ';'
            | ':'
            | '?'
            | '+'
            | '-'
            | '*'
            | '/'
            | '%'
            | '|'
            | '='
            | '<'
            | '>'
            | '^'
            | '&'
            | '!'
            | '~'
    )
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars(),
            buffer: Vec::with_capacity(32),
            byte_index: 0,
            char_index: 0,
            start_byte_index: 0,
            start_char_index: 0,
        }
    }

    pub fn eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    fn bump(&mut self) -> char {
        let c = self.chars.next().unwrap_or(NULL);
        self.byte_index += c.len_utf8();
        self.char_index += 1;
        c
    }

    fn peek(&mut self) -> char {
        self.chars.clone().next().unwrap_or(NULL)
    }

    fn peek_second(&mut self) -> char {
        let mut iter = self.chars.clone();
        iter.next();
        iter.next().unwrap_or(NULL)
    }

    fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while predicate(self.peek()) && !self.eof() {
            self.bump();
        }
    }

    fn token_string(&self) -> String {
        String::from(&self.input[self.start_byte_index..self.byte_index])
    }

    pub fn string_from_token(&self, token: &Token) -> String {
        String::from(&self.input[token.byte_index..token.byte_index + token.len])
    }

    fn get_hex_digit(&mut self) -> Result<u16> {
        let ch = self.bump();
        if ch.len_utf8() != 1 {
            // Not a single byte
            return Err(Error::S0104InvalidUnicodeEscape(self.start_char_index));
        }
        let ch = ch as u8;
        Ok(match ch {
            b'0'..=b'9' => (ch - b'0'),
            b'a'..=b'f' => (ch + 10 - b'a'),
            b'A'..=b'F' => (ch + 10 - b'A'),
            _ => return Err(Error::S0104InvalidUnicodeEscape(self.start_char_index)),
        } as u16)
    }

    fn get_codepoint(&mut self) -> Result<u16> {
        Ok(self.get_hex_digit()? << 12
            | self.get_hex_digit()? << 8
            | self.get_hex_digit()? << 4
            | self.get_hex_digit()?)
    }

    pub fn next_token(&mut self) -> Result<Token> {
        use TokenKind::*;

        let kind = loop {
            self.start_byte_index = self.byte_index;
            self.start_char_index = self.char_index;

            let kind = match self.bump() {
                NULL => End,

                c if is_whitespace(c) => {
                    self.eat_while(is_whitespace);
                    Whitespace
                }

                // Comments, forward-slashes or regexp
                // TODO: Regexp
                '/' => match self.peek() {
                    '*' => {
                        // Skip the *
                        self.bump();

                        loop {
                            // Eat until the next *
                            self.eat_while(|c| c != '*');

                            // Skip the *
                            self.bump();

                            // Check for unterminated comments
                            if self.eof() {
                                return Err(Error::S0106UnterminatedComment(self.start_char_index));
                            }

                            // Is this the end of the comment?
                            if self.bump() == '/' {
                                break;
                            }
                        }

                        Comment
                    }
                    _ => ForwardSlash,
                },

                '.' => match self.peek() {
                    '.' => {
                        self.bump();
                        Range
                    }
                    _ => Period,
                },

                ':' => match self.peek() {
                    '=' => {
                        self.bump();
                        Bind
                    }
                    _ => Colon,
                },

                '!' => match self.peek() {
                    '=' => {
                        self.bump();
                        NotEqual
                    }
                    _ => ExclamationMark,
                },

                '*' => match self.peek() {
                    '*' => {
                        self.bump();
                        Descendent
                    }
                    _ => Asterisk,
                },

                '~' => match self.peek() {
                    '>' => {
                        self.bump();
                        Apply
                    }
                    _ => Tilde,
                },

                '>' => match self.peek() {
                    '=' => {
                        self.bump();
                        GreaterEqual
                    }
                    _ => RightAngleBracket,
                },

                '<' => match self.peek() {
                    '=' => {
                        self.bump();
                        LessEqual
                    }
                    _ => LeftAngleBracket,
                },

                '[' => LeftBracket,
                ']' => RightBracket,
                '{' => LeftBrace,
                '}' => RightBrace,
                '(' => LeftParen,
                ')' => RightParen,
                ',' => Comma,
                '@' => At,
                '#' => Hash,
                ';' => SemiColon,
                '?' => QuestionMark,
                '+' => Plus,
                '-' => Minus,
                '%' => PercentSign,
                '|' => Pipe,
                '=' => Equal,
                '^' => Caret,
                '&' => Ampersand,

                // Backtick identifiers like a.`b`.c
                '`' => {
                    let start_byte_index = self.byte_index;

                    // Eat until the next `
                    self.eat_while(|c| c != '`');

                    // Check for unterminated quotes
                    if self.eof() {
                        return Err(Error::S0105UnterminatedQuoteProp(self.start_char_index));
                    }

                    let token = Name(String::from(&self.input[start_byte_index..self.byte_index]));

                    // Skip the final `
                    self.bump();

                    token
                }

                // String literals
                quote @ ('\'' | '"') => {
                    loop {
                        match self.bump() {
                            // Supported escape sequences
                            '\\' => match self.bump() {
                                '\\' => self.buffer.push('\\'),
                                '"' => self.buffer.push('"'),
                                'b' => self.buffer.push('\x08'),
                                'f' => self.buffer.push('\x0c'),
                                'n' => self.buffer.push('\n'),
                                'r' => self.buffer.push('\r'),
                                't' => self.buffer.push('\t'),

                                // 2-byte hex UTF-16 escape like \u0010.
                                // Note that UTF-16 surrogate pairs (for characters outside of the Basic Multilingual Plane)
                                // are represented as two escape sequences which can't be directly converted to a UTF-8 char.
                                // Example: \\uD83D\\uDE02 => 😂
                                'u' => {
                                    let codepoint = self.get_codepoint()?;

                                    let unicode = match char::try_from(codepoint as u32) {
                                        Ok(code) => code,
                                        Err(_) => match (self.bump(), self.bump()) {
                                            // The codepoint was not valid UTF-8, look for another one that could be part
                                            // of a surrogate pair
                                            ('\\', 'u') => {
                                                match decode_utf16(
                                                    [codepoint, self.get_codepoint()?]
                                                        .iter()
                                                        .copied(),
                                                )
                                                .next()
                                                {
                                                    Some(Ok(code)) => code,
                                                    _ => {
                                                        return Err(
                                                            Error::S0104InvalidUnicodeEscape(
                                                                self.start_char_index,
                                                            ),
                                                        )
                                                    }
                                                }
                                            }
                                            _ => {
                                                return Err(Error::S0104InvalidUnicodeEscape(
                                                    self.start_char_index,
                                                ))
                                            }
                                        },
                                    };

                                    self.buffer.push(unicode);
                                }
                                c => {
                                    return Err(Error::S0103UnsupportedEscape(
                                        self.start_char_index,
                                        c,
                                    ));
                                }
                            },

                            // End of string
                            c if c == quote => {
                                break;
                            }

                            c => {
                                // Check for unterminated strings
                                if self.eof() {
                                    return Err(Error::S0101UnterminatedStringLiteral(
                                        self.start_char_index,
                                    ));
                                }

                                self.buffer.push(c);
                            }
                        }
                    }

                    let s = String::from_iter(self.buffer.clone());
                    let token = Str(s);

                    // The buffer gets cleared for the next string
                    self.buffer.clear();

                    token
                }

                // Numbers
                '0' => {
                    if self.eof() {
                        Num(0.into())
                    } else {
                        let mut mantissa = 0;
                        let mut exponent = 0;
                        let num = self.number_extensions(&mut mantissa, &mut exponent)?;
                        Num(num)
                    }
                }
                c @ '1'..='9' => {
                    let num = self.number(c)?;
                    Num(num)
                }

                // Names
                c if is_name_start(c) => {
                    self.eat_while(|c| !(is_whitespace(c) || is_operator(c)));

                    if c == '$' {
                        Var(String::from(
                            &self.input[self.start_byte_index + 1..self.byte_index],
                        ))
                    } else {
                        match &self.input[self.start_byte_index..self.byte_index] {
                            "or" => Or,
                            "in" => In,
                            "and" => And,
                            "true" => Bool(true),
                            "false" => Bool(false),
                            "null" => Null,
                            _ => Name(String::from(
                                &self.input[self.start_byte_index..self.byte_index],
                            )),
                        }
                    }
                }

                _ => {
                    return Err(Error::S0204UnknownOperator(
                        self.start_char_index,
                        self.token_string(),
                    ));
                }
            };

            if !matches!(kind, Whitespace | Comment) {
                break kind;
            }
        };

        let token = Token {
            kind,
            char_index: self.start_char_index,
            byte_index: self.start_byte_index,
            len: self.byte_index - self.start_byte_index,
        };

        Ok(token)
    }

    // NOTE: Much of this number parsing was stolen from the json create, and modified
    // as needed. See json/README.md.

    fn number(&mut self, first_char: char) -> Result<Number> {
        let mut mantissa = (first_char as u8 - b'0') as u64;

        let result: Number;

        loop {
            if mantissa > MAX_PRECISION {
                return Err(Error::D1001NumberOfOutRange(0.0));
            }

            if self.eof() {
                result = mantissa.into();
                break;
            }

            match self.peek() {
                c @ '0'..='9' => {
                    self.bump();
                    mantissa = mantissa * 10 + (c as u8 - b'0') as u64;
                }
                _ => {
                    let mut exponent = 0;
                    result = self.number_extensions(&mut mantissa, &mut exponent)?;
                    break;
                }
            }
        }

        Ok(result)
    }

    fn number_extensions(&mut self, mantissa: &mut u64, exponent: &mut i16) -> Result<Number> {
        match self.peek() {
            '.' => match self.peek_second() {
                // Range operator
                '.' => Ok((*mantissa).into()),
                _ => {
                    self.bump();
                    self.number_fraction(mantissa, exponent)
                }
            },
            'e' | 'E' => {
                self.bump();
                self.number_exponent(mantissa, exponent)
            }
            _ => Ok((*mantissa).into()),
        }
    }

    fn number_fraction(&mut self, mantissa: &mut u64, exponent: &mut i16) -> Result<Number> {
        let result: Number;

        // Have to have at least one fractional digit
        match self.peek() {
            c @ '0'..='9' => {
                self.bump();
                if *mantissa < MAX_PRECISION {
                    *mantissa = *mantissa * 10 + (c as u8 - b'0') as u64;
                    *exponent -= 1;
                } else {
                    match mantissa
                        .checked_mul(10)
                        .and_then(|m| m.checked_add((c as u8 - b'0') as u64))
                    {
                        Some(result) => {
                            *mantissa = result;
                            *exponent -= 1;
                        }
                        None => return Err(Error::D1001NumberOfOutRange(0.0)),
                    }
                }
            }
            _ => {
                return Err(Error::S0201SyntaxError(
                    self.start_char_index,
                    self.token_string(),
                ));
            }
        }

        // Get the rest of the fractional digits
        loop {
            if self.eof() {
                result = self.finalize_number(*mantissa, *exponent)?;
                break;
            }

            match self.peek() {
                c @ '0'..='9' => {
                    self.bump();
                    if *mantissa < MAX_PRECISION {
                        *mantissa = *mantissa * 10 + (c as u8 - b'0') as u64;
                        *exponent -= 1;
                    } else {
                        match mantissa
                            .checked_mul(10)
                            .and_then(|m| m.checked_add((c as u8 - b'0') as u64))
                        {
                            Some(result) => {
                                *mantissa = result;
                                *exponent -= 1;
                            }
                            None => return Err(Error::D1001NumberOfOutRange(0.0)),
                        }
                    }
                }
                'e' | 'E' => {
                    self.bump();
                    result = self.number_exponent(mantissa, exponent)?;
                    break;
                }
                _ => {
                    result = self.finalize_number(*mantissa, *exponent)?;
                    break;
                }
            }
        }

        Ok(result)
    }

    fn number_exponent(
        &mut self,
        mantissa: &mut u64,
        original_exponent: &mut i16,
    ) -> Result<Number> {
        let sign = match self.peek() {
            '-' => {
                self.bump();
                -1
            }
            '+' => {
                self.bump();
                1
            }
            _ => 1,
        };

        let mut exponent = match self.peek() {
            c @ '0'..='9' => {
                self.bump();
                (c as u8 - b'0') as i16
            }
            _ => {
                return Err(Error::S0201SyntaxError(
                    self.start_char_index,
                    self.token_string(),
                ));
            }
        };

        loop {
            if self.eof() {
                break;
            }

            match self.peek() {
                c @ '0'..='9' => {
                    self.bump();
                    exponent = exponent
                        .saturating_mul(10)
                        .saturating_add((c as u8 - b'0') as i16);
                }
                _ => break,
            }
        }

        self.finalize_number(*mantissa, original_exponent.saturating_add(exponent * sign))
    }

    /// Final checks on a number for out of range conditions
    fn finalize_number(&self, mantissa: u64, exponent: i16) -> Result<Number> {
        let result = unsafe { Number::from_parts_unchecked(true, mantissa, exponent) };
        match f64::try_from(result) {
            Ok(f) => match f.classify() {
                std::num::FpCategory::Infinite
                | std::num::FpCategory::Nan
                | std::num::FpCategory::Subnormal => {
                    return Err(Error::S0102LexedNumberOutOfRange(
                        self.start_byte_index,
                        self.token_string(),
                    ))
                }
                _ => {}
            },
            _ => {
                return Err(Error::S0102LexedNumberOutOfRange(
                    self.start_byte_index,
                    self.token_string(),
                ))
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comment() {
        let mut t = Tokenizer::new("/* This is a comment */");
        assert!(matches!(t.next_token().unwrap().kind, TokenKind::End));
    }

    #[test]
    fn operators() {
        let mut t = Tokenizer::new("@..[]{}()=^&,~>#+<=:=>=!=?-***");
        assert!(matches!(t.next_token().unwrap().kind, TokenKind::At));
        assert!(matches!(t.next_token().unwrap().kind, TokenKind::Range));
        assert!(matches!(
            t.next_token().unwrap().kind,
            TokenKind::LeftBracket
        ));
        assert!(matches!(
            t.next_token().unwrap().kind,
            TokenKind::RightBracket
        ));
        assert!(matches!(t.next_token().unwrap().kind, TokenKind::LeftBrace));
        assert!(matches!(
            t.next_token().unwrap().kind,
            TokenKind::RightBrace
        ));
        assert!(matches!(t.next_token().unwrap().kind, TokenKind::LeftParen));
        assert!(matches!(
            t.next_token().unwrap().kind,
            TokenKind::RightParen
        ));
        assert!(matches!(t.next_token().unwrap().kind, TokenKind::Equal));
        assert!(matches!(t.next_token().unwrap().kind, TokenKind::Caret));
        assert!(matches!(t.next_token().unwrap().kind, TokenKind::Ampersand));
        assert!(matches!(t.next_token().unwrap().kind, TokenKind::Comma));
        assert!(matches!(t.next_token().unwrap().kind, TokenKind::Apply));
        assert!(matches!(t.next_token().unwrap().kind, TokenKind::Hash));
        assert!(matches!(t.next_token().unwrap().kind, TokenKind::Plus));
        assert!(matches!(t.next_token().unwrap().kind, TokenKind::LessEqual));
        assert!(matches!(t.next_token().unwrap().kind, TokenKind::Bind));
        assert!(matches!(
            t.next_token().unwrap().kind,
            TokenKind::GreaterEqual
        ));
        assert!(matches!(t.next_token().unwrap().kind, TokenKind::NotEqual));
        assert!(matches!(
            t.next_token().unwrap().kind,
            TokenKind::QuestionMark
        ));
        assert!(matches!(t.next_token().unwrap().kind, TokenKind::Minus));
        assert!(matches!(
            t.next_token().unwrap().kind,
            TokenKind::Descendent
        ));
        assert!(matches!(t.next_token().unwrap().kind, TokenKind::Asterisk));
        assert!(matches!(t.next_token().unwrap().kind, TokenKind::End));
    }

    #[test]
    fn strings() {
        let mut t = Tokenizer::new("\"There's a string here\" 'and another here'");
        assert!(matches!(
            t.next_token().unwrap().kind,
            TokenKind::Str(s) if s == "There's a string here"
        ));
        assert!(matches!(
            t.next_token().unwrap().kind,
            TokenKind::Str(s) if s == "and another here"
        ));
        assert!(matches!(t.next_token().unwrap().kind, TokenKind::End));
    }

    #[test]
    fn unicode_escapes() {
        let mut t = Tokenizer::new("\"\\u2d63\\u2d53\\u2d4d\"");
        assert!(matches!(
            t.next_token().unwrap().kind,
            TokenKind::Str(s) if s ==  "ⵣⵓⵍ"
        ));
    }

    #[test]
    fn backtick_names() {
        let mut t = Tokenizer::new("  `hello`    `world`");
        assert!(matches!(
            t.next_token().unwrap().kind,
            TokenKind::Name(s) if s == "hello"
        ));
        assert!(matches!(
            t.next_token().unwrap().kind,
            TokenKind::Name(s) if s == "world"
        ));
    }

    #[test]
    fn variables() {
        let mut t = Tokenizer::new("  $one   $two   $three  ");
        assert!(matches!(
            t.next_token().unwrap().kind,
            TokenKind::Var(s) if s == "one"
        ));
        assert!(matches!(
            t.next_token().unwrap().kind,
            TokenKind::Var(s) if s == "two"
        ));
        assert!(matches!(
            t.next_token().unwrap().kind,
            TokenKind::Var(s) if s == "three"
        ));
    }

    #[test]
    fn name_operators() {
        let mut t = Tokenizer::new("or in and");
        assert!(matches!(t.next_token().unwrap().kind, TokenKind::Or));
        assert!(matches!(t.next_token().unwrap().kind, TokenKind::In));
        assert!(matches!(t.next_token().unwrap().kind, TokenKind::And));
    }

    #[test]
    fn values() {
        let mut t = Tokenizer::new("true false null");
        assert!(matches!(
            t.next_token().unwrap().kind,
            TokenKind::Bool(true)
        ));
        assert!(matches!(
            t.next_token().unwrap().kind,
            TokenKind::Bool(false)
        ));
        assert!(matches!(t.next_token().unwrap().kind, TokenKind::Null));
    }

    #[test]
    fn numbers() {
        let mut t = Tokenizer::new("0 1 0.234 5.678 0e0 1e1 1e-1 1e+1 2.234E-2");
        assert!(matches!(
            t.next_token().unwrap().kind,
            TokenKind::Num(n) if (f64::from(n) - 0.0_f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            t.next_token().unwrap().kind,
            TokenKind::Num(n) if (f64::from(n) - 1.0_f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            t.next_token().unwrap().kind,
            TokenKind::Num(n) if (f64::from(n) - 0.234_f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            t.next_token().unwrap().kind,
            TokenKind::Num(n) if (f64::from(n) - 5.678_f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            t.next_token().unwrap().kind,
            TokenKind::Num(n) if (f64::from(n) - 0e0_f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            t.next_token().unwrap().kind,
            TokenKind::Num(n) if (f64::from(n) - 1e1_f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            t.next_token().unwrap().kind,
            TokenKind::Num(n) if (f64::from(n) - 1e-1_f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            t.next_token().unwrap().kind,
            TokenKind::Num(n) if (f64::from(n) - 1e+1_f64).abs() < f64::EPSILON
        ));
        assert!(matches!(
            t.next_token().unwrap().kind,
            TokenKind::Num(n) if (f64::from(n) - 2.234E-2_f64).abs() < f64::EPSILON
        ));
    }
}
