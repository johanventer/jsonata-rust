use std::char::decode_utf16;
use std::str::Chars;
use std::{char, str};

use jsonata_errors::{Error, Result};
use jsonata_shared::Position;

use super::error::*;
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
    ExlamationMark,
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
    Signature(String),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
}

#[derive(Debug)]
struct Tokenizer<'a> {
    input: &'a str,
    chars: Chars<'a>,

    buffer: Vec<char>,

    byte_index: usize,
    char_index: usize,
}

const NULL: char = '\0';
const MAX_PRECISION: u64 = 576460752303423500;

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

    fn second(&mut self) -> char {
        let mut iter = self.chars.clone();
        iter.next();
        iter.next().unwrap_or(NULL)
    }

    fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while predicate(self.peek()) && !self.eof() {
            self.bump();
        }
    }

    // TODO: remove
    fn pos(&self) -> Position {
        Position {
            line: 0,
            column: 0,
            source_pos: 0,
        }
    }

    fn get_hex_digit(&mut self) -> Result<u16> {
        let ch = self.bump();
        if ch.len_utf8() != 1 {
            // Not a single byte
            return Err(Error::InvalidUnicodeEscape(self.pos()));
        }
        let ch = ch as u8;
        Ok(match ch {
            b'0'..=b'9' => (ch - b'0'),
            b'a'..=b'f' => (ch + 10 - b'a'),
            b'A'..=b'F' => (ch + 10 - b'A'),
            _ => return Err(Error::InvalidUnicodeEscape(self.pos())),
        } as u16)
    }

    fn get_codepoint(&mut self) -> Result<u16> {
        Ok(self.get_hex_digit()? << 12
            | self.get_hex_digit()? << 8
            | self.get_hex_digit()? << 4
            | self.get_hex_digit()?)
    }

    pub fn next(&mut self) -> Result<Token> {
        use TokenKind::*;

        let (kind, start_byte_index, start_char_index) = loop {
            let start_byte_index = self.byte_index;
            let start_char_index = self.char_index;

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
                                return Err(Error::UnterminatedComment(self.pos()));
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
                    _ => ExlamationMark,
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
                    _ => {
                        // TODO: Could be a signature
                        LeftAngleBracket
                    }
                },

                // Minus or negative number
                '-' => match self.peek() {
                    '0' => {
                        self.bump();
                        let mut mantissa = 0;
                        let mut exponent = 0;
                        let num = self.number_extensions(&mut mantissa, &mut exponent)?;
                        Num(-num)
                    }
                    c @ '1'..='9' => {
                        self.bump();
                        let num = self.number(c)?;
                        Num(-num)
                    }
                    _ => Minus,
                },

                '[' => LeftBracket,
                ']' => RightBracket,
                '{' => LeftBrace,
                '}' => RightBrace,
                '(' => LeftParen,
                ')' => RightParen,
                ',' => Comma,
                '@' => Ampersand,
                '#' => Hash,
                ';' => SemiColon,
                '?' => QuestionMark,
                '+' => Plus,
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
                        return Err(Error::UnterminatedQuoteProp(self.pos()));
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
                                                        return Err(Error::InvalidUnicodeEscape(
                                                            self.pos(),
                                                        ))
                                                    }
                                                }
                                            }
                                            _ => {
                                                return Err(Error::InvalidUnicodeEscape(self.pos()))
                                            }
                                        },
                                    };

                                    self.buffer.push(unicode);
                                }
                                c => {
                                    return Err(unsupported_escape(self.pos(), c));
                                }
                            },

                            // End of string
                            c if c == quote => {
                                break;
                            }

                            c => {
                                // Check for unterminated strings
                                if self.eof() {
                                    return Err(Error::UnterminatedStringLiteral(self.pos()));
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
                c if c.is_alphabetic() => {
                    self.eat_while(|c| !(is_whitespace(c) || is_operator(c)));

                    match &self.input[start_byte_index..self.byte_index] {
                        "or" => Or,
                        "in" => In,
                        "and" => And,
                        "true" => Bool(true),
                        "false" => Bool(false),
                        "null" => Null,
                        _ => Name(String::from(&self.input[start_byte_index..self.byte_index])),
                    }
                }

                c => {
                    // TODO: An error about unknown characters
                    eprintln!("UNHANDLED CHAR: {}", c);
                    unimplemented!()
                }
            };

            if !matches!(kind, Whitespace | Comment) {
                break (kind, start_byte_index, start_char_index);
            }
        };

        let token = Token { kind };

        if let Num(n) = token.kind {
            eprintln!("NUMBER: {}", n);
        } else {
            eprintln!(
                "{:?}, byte_len: {}, char_len: {}",
                token,
                self.byte_index - start_byte_index,
                self.char_index - start_char_index
            );
        }

        Ok(token)
    }

    // NOTE: Much of this number parsing was stolen from the json create, see json/README.md.

    fn number(&mut self, first_char: char) -> Result<Number> {
        let mut mantissa = (first_char as u8 - b'0') as u64;

        let result: Number;

        loop {
            if mantissa >= MAX_PRECISION {
                // TODO: Big numbers
                return Err(Error::NumberOfOutRange(0.0));
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
        match self.bump() {
            '.' => self.number_fraction(mantissa, exponent),
            'e' | 'E' => self.number_exponent(mantissa, exponent),
            _ => Ok((*mantissa).into()),
        }
    }

    fn number_fraction(&mut self, mantissa: &mut u64, exponent: &mut i16) -> Result<Number> {
        let result: Number;

        // Have to have at least one fractional digit
        match self.bump() {
            c @ '0'..='9' => {
                if *mantissa < MAX_PRECISION {
                    *mantissa = *mantissa * 10 + (c as u8 - b'0') as u64;
                    *exponent -= 1;
                } else if let Some(result) = mantissa
                    .checked_mul(10)
                    .and_then(|m| m.checked_add((c as u8 - b'0') as u64))
                {
                    *mantissa = result;
                    *exponent -= 1;
                }
            }
            _ => {
                // TODO
                unimplemented!()
            }
        }

        // Get the rest of the fractional digits
        loop {
            if self.eof() {
                result = unsafe { Number::from_parts_unchecked(true, *mantissa, *exponent) };
                break;
            }

            match self.bump() {
                c @ '0'..='9' => {
                    if *mantissa < MAX_PRECISION {
                        *mantissa = *mantissa * 10 + (c as u8 - b'0') as u64;
                        *exponent -= 1;
                    } else if let Some(result) = mantissa
                        .checked_mul(10)
                        .and_then(|m| m.checked_add((c as u8 - b'0') as u64))
                    {
                        *mantissa = result;
                        *exponent -= 1;
                    }
                }
                'e' | 'E' => {
                    result = self.number_exponent(mantissa, exponent)?;
                    break;
                }
                _ => {
                    result = unsafe { Number::from_parts_unchecked(true, *mantissa, *exponent) };
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

        let mut exponent = match self.bump() {
            c @ '0'..='9' => (c as u8 - b'0') as i16,
            _ => {
                // TODO
                unimplemented!()
            }
        };

        loop {
            if self.eof() {
                break;
            }

            match self.bump() {
                c @ '0'..='9' => {
                    exponent = exponent
                        .saturating_mul(10)
                        .saturating_add((c as u8 - b'0') as i16);
                }
                _ => break,
            }
        }

        Ok(unsafe {
            Number::from_parts_unchecked(
                true,
                *mantissa,
                original_exponent.saturating_add(exponent * sign),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comment() {
        let mut t = Tokenizer::new("/* This is a comment */");
        assert!(matches!(t.next().unwrap().kind, TokenKind::Comment));
        assert!(matches!(t.next().unwrap().kind, TokenKind::End));
    }

    #[test]
    fn drive() {
        let mut t = Tokenizer::new(
            //            "!= := : = @ # *   -   +  or in and Product ( ) { } [  ]  .. Account 'hello \\uD83D\\uDE02 \\n world' `b` /*   */ !=",
            "-1.1234",
        );
        loop {
            let token = t.next().unwrap();
            if token.kind == TokenKind::End {
                break;
            }
        }
    }

    // #[test]
    // fn operators() {
    //     let mut tokenizer = Tokenizer::new("  @   # +  <=>= /* This is a comment */ ? -*");
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::At
    //     ));
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::Hash
    //     ));
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::Plus
    //     ));
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::LessEqual
    //     ));
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::GreaterEqual
    //     ));
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::Question
    //     ));
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::Minus
    //     ));
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::Wildcard
    //     ));
    // }

    // #[test]
    // fn strings() {
    //     let mut tokenizer = Tokenizer::new("\"There's a string here\" 'and another here'");
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::Str(s) if s == "There's a string here"
    //     ));
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::Str(s) if s == "and another here"
    //     ));
    // }

    // #[test]
    // fn unicode_escapes() {
    //     let mut tokenizer = Tokenizer::new("\"\\u2d63\\u2d53\\u2d4d\"");
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::Str(s) if s ==  "ⵣⵓⵍ"
    //     ));
    // }

    // #[test]
    // fn backtick_names() {
    //     let mut tokenizer = Tokenizer::new("  `hello`    `world`");
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::Name(s) if s == "hello"
    //     ));
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::Name(s) if s == "world"
    //     ));
    // }

    // #[test]
    // fn variables() {
    //     let mut tokenizer = Tokenizer::new("  $one   $two   $three  ");
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::Var(s) if s == "one"
    //     ));
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::Var(s) if s == "two"
    //     ));
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::Var(s) if s == "three"
    //     ));
    // }

    // #[test]
    // fn name_operators() {
    //     let mut tokenizer = Tokenizer::new("or in and");
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::Or
    //     ));
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::In
    //     ));
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::And
    //     ));
    // }

    // #[test]
    // fn values() {
    //     let mut tokenizer = Tokenizer::new("true false null");
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::Bool(true)
    //     ));
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::Bool(false)
    //     ));
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::Null
    //     ));
    // }

    // #[test]
    // fn numbers() {
    //     let mut tokenizer = Tokenizer::new("0 1 0.234 5.678 0e0 1e1 1e-1 1e+1 2.234E-2");
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::Num(n) if (f64::from(n) - 0.0_f64).abs() < f64::EPSILON
    //     ));
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::Num(n) if (f64::from(n) - 1.0_f64).abs() < f64::EPSILON
    //     ));
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::Num(n) if (f64::from(n) - 0.234_f64).abs() < f64::EPSILON
    //     ));
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::Num(n) if (f64::from(n) - 5.678_f64).abs() < f64::EPSILON
    //     ));
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::Num(n) if (f64::from(n) - 0e0_f64).abs() < f64::EPSILON
    //     ));
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::Num(n) if (f64::from(n) - 1e1_f64).abs() < f64::EPSILON
    //     ));
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::Num(n) if (f64::from(n) - 1e-1_f64).abs() < f64::EPSILON
    //     ));
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::Num(n) if (f64::from(n) - 1e+1_f64).abs() < f64::EPSILON
    //     ));
    //     assert!(matches!(
    //         tokenizer.next(false, false).unwrap().kind,
    //         TokenKind::Num(n) if (f64::from(n) - 2.234E-2_f64).abs() < f64::EPSILON
    //     ));
    // }
}
