use jsonata_errors::{Error, Result};
use jsonata_signatures;

use super::ast::*;
use super::error::*;
use super::parser::Parser;
use super::tokenizer::{Token, TokenKind};

pub trait Symbol {
    fn left_binding_power(&self) -> u32;
    fn null_denotation(&self, parser: &mut Parser) -> Result<Ast>;
    fn left_denotation(&self, parser: &mut Parser, left: Ast) -> Result<Ast>;
}

impl Symbol for Token {
    fn left_binding_power(&self) -> u32 {
        use TokenKind::*;
        match &self.kind {
            End | Range | Colon | Comma | SemiColon | RightParen | RightBracket | RightBrace
            | Pipe | Not | Tilde | Null | Bool(..) | Str(..) | Num(..) | Name(..) | Var(..) => 0,
            Signature(..) => 0,
            Bind => 10,
            Question => 20,
            Or => 25,
            And => 30,
            NotEqual | GreaterEqual | LessEqual | Apply | In | Equal | RightCaret | LeftCaret
            | Caret => 40,
            Ampersand | Plus | Minus => 50,
            Wildcard | Descendent | ForwardSlash | Percent => 60,
            LeftBrace => 70,
            Period => 75,
            LeftBracket | LeftParen => 80,
            At | Hash => 80,
        }
    }

    fn null_denotation(&self, parser: &mut Parser) -> Result<Ast> {
        match self.kind {
            TokenKind::Null => Ok(Ast::new(AstKind::Null, self.position)),
            TokenKind::Bool(ref v) => Ok(Ast::new(AstKind::Bool(*v), self.position)),
            TokenKind::Str(ref v) => Ok(Ast::new(AstKind::String(v.clone()), self.position)),
            TokenKind::Num(ref v) => Ok(Ast::new(AstKind::Number(*v), self.position)),
            TokenKind::Name(ref v) => Ok(Ast::new(AstKind::Name(v.clone()), self.position)),
            TokenKind::Var(ref v) => Ok(Ast::new(AstKind::Var(v.clone()), self.position)),
            TokenKind::And => Ok(Ast::new(AstKind::Name(String::from("and")), self.position)),
            TokenKind::Or => Ok(Ast::new(AstKind::Name(String::from("or")), self.position)),
            TokenKind::In => Ok(Ast::new(AstKind::Name(String::from("in")), self.position)),
            TokenKind::Minus => Ok(Ast::new(
                AstKind::Unary(UnaryOp::Minus(Box::new(parser.expression(70, false)?))),
                self.position,
            )),
            TokenKind::Wildcard => Ok(Ast::new(AstKind::Wildcard, self.position)),
            TokenKind::Descendent => Ok(Ast::new(AstKind::Descendent, self.position)),
            TokenKind::Percent => Ok(Ast::new(AstKind::Parent, self.position)),

            // Block of expressions
            TokenKind::LeftParen => {
                let mut expressions = Vec::new();

                while parser.token().kind != TokenKind::RightParen {
                    expressions.push(parser.expression(0, false)?);
                    if parser.token().kind != TokenKind::SemiColon {
                        break;
                    }
                    parser.expect(TokenKind::SemiColon, false, false)?;
                }
                parser.expect(TokenKind::RightParen, true, false)?;

                Ok(Ast::new(AstKind::Block(expressions), self.position))
            }

            // Array constructor
            TokenKind::LeftBracket => {
                let mut expressions = Vec::new();

                if parser.token().kind != TokenKind::RightBracket {
                    loop {
                        let mut item = parser.expression(0, false)?;

                        if parser.token().kind == TokenKind::Range {
                            parser.expect(TokenKind::Range, false, false)?;
                            item = Ast::new(
                                AstKind::Binary(
                                    BinaryOp::Range,
                                    Box::new(item),
                                    Box::new(parser.expression(0, false)?),
                                ),
                                self.position,
                            )
                        }

                        expressions.push(item);

                        if parser.token().kind != TokenKind::Comma {
                            break;
                        }

                        parser.expect(TokenKind::Comma, false, false)?;
                    }
                }
                parser.expect(TokenKind::RightBracket, true, false)?;

                Ok(Ast::new(
                    AstKind::Unary(UnaryOp::ArrayConstructor(expressions)),
                    self.position,
                ))
            }

            // Object constructor
            TokenKind::LeftBrace => Ok(Ast::new(
                AstKind::Unary(UnaryOp::ObjectConstructor(parse_object(parser)?)),
                self.position,
            )),

            // Object transformer
            TokenKind::Pipe => {
                let pattern = Box::new(parser.expression(0, false)?);

                parser.expect(TokenKind::Pipe, false, false)?;

                let update = Box::new(parser.expression(0, false)?);

                let delete = if parser.token().kind == TokenKind::Comma {
                    parser.expect(TokenKind::Comma, false, false)?;
                    Some(Box::new(parser.expression(0, false)?))
                } else {
                    None
                };

                parser.expect(TokenKind::Pipe, false, false)?;

                Ok(Ast::new(
                    AstKind::Transform {
                        pattern,
                        update,
                        delete,
                    },
                    self.position,
                ))
            }

            TokenKind::LeftCaret => {
                if let TokenKind::Signature(ref s) = parser.token().kind {
                    Ok(Ast::new(AstKind::String(s.clone()), self.position))
                } else {
                    Err(invalid_unary(self.position, &self.kind))
                }
            }

            _ => Err(invalid_unary(self.position, &self.kind)),
        }
    }

    fn left_denotation(&self, parser: &mut Parser, mut left: Ast) -> Result<Ast> {
        macro_rules! binary {
            ($n:tt) => {
                Ok(Ast::new(
                    AstKind::Binary(
                        BinaryOp::$n,
                        Box::new(left),
                        Box::new(parser.expression(self.left_binding_power(), false)?),
                    ),
                    self.position,
                ))
            };
        }

        match self.kind {
            TokenKind::Period => binary!(Map),
            TokenKind::Plus => binary!(Add),
            TokenKind::Minus => binary!(Subtract),
            TokenKind::Wildcard => binary!(Multiply),
            TokenKind::ForwardSlash => binary!(Divide),
            TokenKind::Percent => binary!(Modulus),
            TokenKind::Equal => binary!(Equal),
            TokenKind::LeftCaret => binary!(LessThan),
            TokenKind::RightCaret => binary!(GreaterThan),
            TokenKind::NotEqual => binary!(NotEqual),
            TokenKind::LessEqual => binary!(LessThanEqual),
            TokenKind::GreaterEqual => binary!(GreaterThanEqual),
            TokenKind::Ampersand => binary!(Concat),
            TokenKind::And => binary!(And),
            TokenKind::Or => binary!(Or),
            TokenKind::In => binary!(In),
            TokenKind::Apply => binary!(Apply),

            // Function calls or lambda definitions
            TokenKind::LeftParen => {
                let mut args = Vec::new();
                let mut is_partial = false;
                let mut is_lambda = false;

                if parser.token().kind != TokenKind::RightParen {
                    loop {
                        match parser.token().kind {
                            TokenKind::Question => {
                                is_partial = true;
                                args.push(Ast::new(AstKind::PartialArg, parser.token().position));
                                parser.expect(TokenKind::Question, false, false)?;
                            }
                            _ => {
                                args.push(parser.expression(0, false)?);
                            }
                        }
                        if parser.token().kind != TokenKind::Comma {
                            break;
                        }
                        parser.expect(TokenKind::Comma, false, false)?;
                    }
                }
                parser.expect(TokenKind::RightParen, true, true)?;

                let name = match left.kind {
                    AstKind::Name(ref name) => {
                        // If the name of the function is 'function' or λ, then this is a function definition (lambda function)
                        if name == "function" || name == "λ" {
                            is_lambda = true;

                            // All of the args must be Variable nodes
                            for arg in &args {
                                if !matches!(arg.kind, AstKind::Var(..)) {
                                    return Err(invalid_function_param(arg.position, &self.kind));
                                }
                            }
                        }
                        name.clone()
                    }
                    AstKind::Var(ref name) => name.clone(),
                    _ => unreachable!(),
                };

                let func: Ast;

                if is_lambda {
                    let signature = if parser.token().kind == TokenKind::LeftCaret {
                        Some(parser.expression(0, true)?)
                    } else {
                        None
                    };

                    let signature = match signature {
                        None => None,
                        Some(Ast {
                            kind: AstKind::String(ref s),
                            ..
                        }) => {
                            parser.next(false, false)?;
                            Some(jsonata_signatures::parse(s)?)
                        }
                        _ => None,
                    };

                    parser.expect(TokenKind::LeftBrace, false, false)?;
                    let body = Box::new(parser.expression(0, false)?);
                    func = Ast::new(
                        AstKind::Lambda {
                            name,
                            args,
                            body,
                            signature,
                        },
                        self.position,
                    );
                    parser.expect(TokenKind::RightBrace, false, false)?;
                } else {
                    func = Ast::new(
                        AstKind::Function {
                            name,
                            proc: Box::new(left),
                            args,
                            is_partial,
                        },
                        self.position,
                    );
                }

                Ok(func)
            }

            // Variable assignment
            TokenKind::Bind => {
                if !matches!(left.kind, AstKind::Var(..)) {
                    return Err(Error::ExpectedVarLeft(left.position));
                }

                Ok(Ast::new(
                    AstKind::Binary(
                        BinaryOp::Bind,
                        Box::new(left),
                        Box::new(parser.expression(self.left_binding_power() - 1, false)?),
                    ),
                    self.position,
                ))
            }

            // Order by expression
            TokenKind::Caret => {
                let mut terms = Vec::new();

                parser.expect(TokenKind::LeftParen, false, false)?;
                loop {
                    let mut descending = false;
                    if parser.token().kind == TokenKind::LeftCaret {
                        parser.expect(TokenKind::LeftCaret, false, false)?;
                    } else if parser.token().kind == TokenKind::RightCaret {
                        parser.expect(TokenKind::RightCaret, false, false)?;
                        descending = true;
                    }

                    terms.push((parser.expression(0, false)?, descending));

                    if parser.token().kind != TokenKind::Comma {
                        break;
                    }
                    parser.expect(TokenKind::Comma, false, false)?;
                }
                parser.expect(TokenKind::RightParen, false, false)?;

                Ok(Ast::new(
                    AstKind::OrderBy(Box::new(left), terms),
                    self.position,
                ))
            }

            // Context variable bind
            TokenKind::At => {
                let rhs = parser.expression(self.left_binding_power(), false)?;

                if !matches!(rhs.kind, AstKind::Var(..)) {
                    return Err(expected_var_right(rhs.position, "@"));
                }

                Ok(Ast::new(
                    AstKind::Binary(BinaryOp::ContextBind, Box::new(left), Box::new(rhs)),
                    self.position,
                ))
            }

            // Positional variable bind
            TokenKind::Hash => {
                let rhs = parser.expression(self.left_binding_power(), false)?;

                if !matches!(rhs.kind, AstKind::Var(..)) {
                    return Err(expected_var_right(rhs.position, "#"));
                }

                Ok(Ast::new(
                    AstKind::Binary(BinaryOp::PositionalBind, Box::new(left), Box::new(rhs)),
                    self.position,
                ))
            }

            // Ternary conditional
            TokenKind::Question => {
                let truthy = Box::new(parser.expression(0, false)?);

                let falsy = if parser.token().kind == TokenKind::Colon {
                    parser.expect(TokenKind::Colon, false, false)?;
                    Some(Box::new(parser.expression(0, false)?))
                } else {
                    None
                };

                Ok(Ast::new(
                    AstKind::Ternary {
                        cond: Box::new(left),
                        truthy,
                        falsy,
                    },
                    self.position,
                ))
            }

            // Object group by
            TokenKind::LeftBrace => Ok(Ast::new(
                AstKind::GroupBy(Box::new(left), parse_object(parser)?),
                self.position,
            )),

            // Array predicate or index
            TokenKind::LeftBracket => {
                if parser.token().kind == TokenKind::RightBracket {
                    // Empty predicate means maintain singleton arrays in the output

                    let mut step = &mut left;

                    // Walk back through left hand sides to find something that's not an array
                    // predicate
                    while let AstKind::Binary(BinaryOp::Predicate, ref mut left, ..) = step.kind {
                        step = left
                    }

                    step.keep_array = true;

                    parser.expect(TokenKind::RightBracket, false, false)?;

                    Ok(left)
                } else {
                    let rhs = parser.expression(0, false)?;
                    parser.expect(TokenKind::RightBracket, true, false)?;
                    Ok(Ast::new(
                        AstKind::Binary(BinaryOp::Predicate, Box::new(left), Box::new(rhs)),
                        self.position,
                    ))
                }
            }

            _ => Err(syntax_error(self.position, &self.kind)),
        }
    }
}

/// Parses an object definition.
fn parse_object(parser: &mut Parser) -> Result<Object> {
    let mut object: Object = Vec::new();
    if parser.token().kind != TokenKind::RightBrace {
        loop {
            let key = parser.expression(0, false)?;
            parser.expect(TokenKind::Colon, false, false)?;
            let value = parser.expression(0, false)?;
            object.push((key, value));
            if parser.token().kind != TokenKind::Comma {
                break;
            }
            parser.expect(TokenKind::Comma, false, false)?;
        }
    }
    parser.expect(TokenKind::RightBrace, true, false)?;
    Ok(object)
}
