use std::{char, str};

#[derive(Debug)]
pub struct Error {
    code: &'static str,
    // stack,
    position: usize,
}

#[derive(Debug)]
enum Token {
    End,
    Range,
    Assignment,
    NotEqual,
    GreaterEqual,
    LessEqual,
    DescendantWildcard,
    ChainFunction,
    Or,
    In,
    And,
    Null,
    Boolean(bool),
    Operator(char),
    String(String),
    Number(f64),
    Name(String),
    Variable(String),
}

pub struct Tokenizer<'a> {
    position: usize,
    source: &'a str,
}

impl<'a> Tokenizer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            position: 0,
            source,
        }
    }

    /// Returns the next token in the stream and its position as a tuple
    fn next_token(&mut self, prefix: bool) -> Result<(Token, usize), Error> {
        loop {
            match self.source.as_bytes()[self.position..] {
                [] => {
                    break Ok((Token::End, self.position));
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
                    break Ok((Token::Range, self.position));
                }
                // := Assignment
                [b':', b'=', ..] => {
                    self.position += 2;
                    break Ok((Token::Assignment, self.position));
                }
                // !=
                [b'!', b'=', ..] => {
                    self.position += 2;
                    break Ok((Token::NotEqual, self.position));
                }
                // >=
                [b'>', b'=', ..] => {
                    self.position += 2;
                    break Ok((Token::GreaterEqual, self.position));
                }
                // <=
                [b'<', b'=', ..] => {
                    self.position += 2;
                    break Ok((Token::LessEqual, self.position));
                }
                // ** Descendent wildcard
                [b'*', b'*', ..] => {
                    self.position += 2;
                    break Ok((Token::DescendantWildcard, self.position));
                }
                // ~> Chain function
                [b'~', b'>', ..] => {
                    self.position += 2;
                    break Ok((Token::ChainFunction, self.position));
                }
                // Numbers
                [b'0'..=b'9', ..] | [b'-', b'0'..=b'9', ..] => {
                    let number_start = self.position;
                    self.position += 1;
                    // TODO(johan): Improve this lexing, it's pretty ordinary and allows all sorts
                    // of invalid stuff
                    let result = loop {
                        match self.source.as_bytes()[self.position..] {
                            [b'0'..=b'9' | b'.' | b'e' | b'E' | b'-' | b'+', ..] => {
                                self.position += 1;
                            }
                            _ => {
                                if let Some(number) = str::from_utf8(
                                    &self.source.as_bytes()[number_start..self.position],
                                )
                                .ok()
                                .and_then(|s| s.parse::<f64>().ok())
                                {
                                    break Ok((Token::Number(number), self.position));
                                } else {
                                    break Err(Error {
                                        code: "S0102",
                                        position: self.position,
                                    });
                                }
                            }
                        }
                    };

                    break result;
                }
                // Single character operators
                [c
                @
                (b'.' | b'[' | b']' | b'{' | b'}' | b'(' | b')' | b',' | b'@' | b'#' | b';'
                | b':' | b'?' | b'+' | b'-' | b'*' | b'/' | b'%' | b'|' | b'=' | b'<'
                | b'>' | b'^' | b'&' | b'!' | b'~'), ..] => {
                    self.position += 1;
                    break Ok((Token::Operator(c as char), self.position));
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
                            break Ok((Token::String(value), self.position));
                        }
                    }
                }
                // Quoted names (backticks)
                [b'`', ..] => {
                    self.position += 1;
                    // Find the closing backtick and convert to a string
                    if let Some(value) = self.source.as_bytes()[self.position..]
                        .iter()
                        .position(|byte| *byte == b'`')
                        .and_then(|index| {
                            String::from_utf8(
                                self.source.as_bytes()[self.position..self.position + index]
                                    .to_vec(),
                            )
                            .ok()
                        })
                    {
                        self.position += value.len() + 1;
                        break Ok((Token::Name(value), self.position));
                    } else {
                        break Err(Error {
                            code: "S0105",
                            position: self.position,
                        });
                    }
                }
                // Names
                [c, ..] => {
                    let name_start = self.position;
                    let result = loop {
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

                                    break Ok((Token::Variable(name), self.position));
                                } else {
                                    // TODO(johan): This could fail to unwrap
                                    let name = String::from_utf8(
                                        self.source.as_bytes()[name_start..self.position].to_vec(),
                                    )
                                    .unwrap();

                                    let token = match &name[..] {
                                        "or" => (Token::Or, self.position),
                                        "in" => (Token::In, self.position),
                                        "and" => (Token::And, self.position),
                                        "true" => (Token::Boolean(true), self.position),
                                        "false" => (Token::Boolean(false), self.position),
                                        "null" => (Token::Null, self.position),
                                        _ => (Token::Name(name), self.position),
                                    };

                                    break Ok(token);
                                }
                            }

                            _ => {
                                self.position += 1;
                            }
                        }
                    };

                    break result;
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

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Operator('@'), _)
        ));

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Operator('#'), _)
        ));

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Operator('+'), _)
        ));

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::LessEqual, _)
        ));

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::GreaterEqual, _)
        ));
        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Operator('?'), _)
        ));

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Operator('-'), _)
        ));

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Operator('*'), _)
        ));
    }

    #[test]
    fn strings() {
        let mut tokenizer = Tokenizer::new("\"There's a string here\" 'and another here'");

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::String(s), _) if s == "There's a string here"
        ));

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::String(s), _) if s == "and another here"
        ));
    }

    #[test]
    fn unicode_escapes() {
        let mut tokenizer = Tokenizer::new("\"\\u2d63\\u2d53\\u2d4d\"");
        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::String(s), _) if s ==  "ⵣⵓⵍ"
        ));
    }

    #[test]
    fn backtick_names() {
        let mut tokenizer = Tokenizer::new("  `hello`    `world`");

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Name(s), _) if s == "hello"
        ));

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Name(s), _) if s == "world"
        ));
    }

    #[test]
    fn variables() {
        let mut tokenizer = Tokenizer::new("  $one   $two   $three  ");

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Variable(s), _) if s == "one"
        ));

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Variable(s), _) if s == "two"
        ));

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Variable(s), _) if s == "three"
        ));
    }

    #[test]
    fn name_operators() {
        let mut tokenizer = Tokenizer::new("or in and");

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Or, _)
        ));

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::In, _)
        ));

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::And, _)
        ));
    }

    #[test]
    fn values() {
        let mut tokenizer = Tokenizer::new("true false null");

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Boolean(true), _)
        ));

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Boolean(false), _)
        ));

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Null, _)
        ));
    }

    #[test]
    fn numbers() {
        let mut tokenizer =
            Tokenizer::new("0 1 0.234 5.678 -0 -1 -0.234 -5.678 0e0 1e1 1e-1 1e+1 -2.234E-2");

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Number(n), _) if (n - 0.0 as f64).abs() < f64::EPSILON
        ));

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Number(n), _) if (n - 1.0 as f64).abs() < f64::EPSILON
        ));

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Number(n), _) if (n - 0.234 as f64).abs() < f64::EPSILON
        ));

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Number(n), _) if (n - 5.678 as f64).abs() < f64::EPSILON
        ));

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Number(n), _) if (n - -0.0 as f64).abs() < f64::EPSILON
        ));

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Number(n), _) if (n - -1.0 as f64).abs() < f64::EPSILON
        ));

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Number(n), _) if (n - -0.234 as f64).abs() < f64::EPSILON
        ));

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Number(n), _) if (n - -5.678 as f64).abs() < f64::EPSILON
        ));

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Number(n), _) if (n - 0e0 as f64).abs() < f64::EPSILON
        ));

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Number(n), _) if (n - 1e1 as f64).abs() < f64::EPSILON
        ));

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Number(n), _) if (n - 1e-1 as f64).abs() < f64::EPSILON
        ));

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Number(n), _) if (n - 1e+1 as f64).abs() < f64::EPSILON
        ));

        assert!(matches!(
            tokenizer.next_token(false).unwrap(),
            (Token::Number(n), _) if (n - -2.234E-2 as f64).abs() < f64::EPSILON
        ));
    }
}
