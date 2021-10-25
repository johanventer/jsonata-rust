use crate::error::*;
use crate::Result;

use super::ast::*;
use super::parser::Parser;
use super::tokenizer::{Token, TokenKind};

pub(crate) trait Symbol {
    fn lbp(&self) -> u32;
    fn nud(&self, parser: &mut Parser) -> Result<Box<Node>>;
    fn led(&self, parser: &mut Parser, left: Box<Node>) -> Result<Box<Node>>;
}

impl Symbol for Token {
    fn lbp(&self) -> u32 {
        use TokenKind::*;
        match &self.kind {
            End | Range | Colon | Comma | SemiColon | RightParen | RightBracket | RightBrace
            | Pipe | Not | Tilde | Null | Bool(..) | Str(..) | Num(..) | Name(..) | Var(..) => 0,
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

    fn nud(&self, parser: &mut Parser) -> Result<Box<Node>> {
        match self.kind {
            TokenKind::Null => Ok(Box::new(Node::new(NodeKind::Null, self.position))),
            TokenKind::Bool(ref v) => Ok(Box::new(Node::new(NodeKind::Bool(*v), self.position))),
            TokenKind::Str(ref v) => {
                Ok(Box::new(Node::new(NodeKind::Str(v.clone()), self.position)))
            }
            TokenKind::Num(ref v) => Ok(Box::new(Node::new(NodeKind::Num(*v), self.position))),
            TokenKind::Name(ref v) => Ok(Box::new(Node::new(
                NodeKind::Name(v.clone()),
                self.position,
            ))),
            TokenKind::Var(ref v) => {
                Ok(Box::new(Node::new(NodeKind::Var(v.clone()), self.position)))
            }
            TokenKind::And => Ok(Box::new(Node::new(
                NodeKind::Name(String::from("and")),
                self.position,
            ))),
            TokenKind::Or => Ok(Box::new(Node::new(
                NodeKind::Name(String::from("or")),
                self.position,
            ))),
            TokenKind::In => Ok(Box::new(Node::new(
                NodeKind::Name(String::from("in")),
                self.position,
            ))),
            TokenKind::Minus => Ok(Box::new(Node::new(
                NodeKind::Unary(UnaryOp::Minus(parser.expression(70)?)),
                self.position,
            ))),
            TokenKind::Wildcard => Ok(Box::new(Node::new(NodeKind::Wildcard, self.position))),
            TokenKind::Descendent => Ok(Box::new(Node::new(NodeKind::Descendent, self.position))),
            TokenKind::Percent => Ok(Box::new(Node::new(NodeKind::Parent, self.position))),

            // Block of expressions
            TokenKind::LeftParen => {
                let mut expressions = Vec::new();

                while parser.token().kind != TokenKind::RightParen {
                    expressions.push(parser.expression(0)?);
                    if parser.token().kind != TokenKind::SemiColon {
                        break;
                    }
                    parser.expect(TokenKind::SemiColon, false)?;
                }
                parser.expect(TokenKind::RightParen, true)?;

                Ok(Box::new(Node::new(
                    NodeKind::Block(expressions),
                    self.position,
                )))
            }

            // Array constructor
            TokenKind::LeftBracket => {
                let mut expressions = Vec::new();

                if parser.token().kind != TokenKind::RightBracket {
                    loop {
                        let mut item = parser.expression(0)?;

                        if parser.token().kind == TokenKind::Range {
                            parser.expect(TokenKind::Range, false)?;
                            item = Box::new(Node::new(
                                NodeKind::Binary(BinaryOp::Range, item, parser.expression(0)?),
                                self.position,
                            ))
                        }

                        expressions.push(item);

                        if parser.token().kind != TokenKind::Comma {
                            break;
                        }

                        parser.expect(TokenKind::Comma, false)?;
                    }
                }
                parser.expect(TokenKind::RightBracket, true)?;

                Ok(Box::new(Node::new(
                    NodeKind::Unary(UnaryOp::ArrayConstructor(expressions)),
                    self.position,
                )))
            }

            // Object constructor
            TokenKind::LeftBrace => Ok(Box::new(Node::new(
                NodeKind::Unary(UnaryOp::ObjectConstructor(parse_object(parser)?)),
                self.position,
            ))),

            // Object transformer
            TokenKind::Pipe => {
                let pattern = parser.expression(0)?;

                parser.expect(TokenKind::Pipe, false)?;

                let update = parser.expression(0)?;

                let delete = if parser.token().kind == TokenKind::Comma {
                    parser.expect(TokenKind::Comma, false)?;
                    Some(parser.expression(0)?)
                } else {
                    None
                };

                parser.expect(TokenKind::Pipe, false)?;

                Ok(Box::new(Node::new(
                    NodeKind::Transform {
                        pattern,
                        update,
                        delete,
                    },
                    self.position,
                )))
            }

            _ => Err(Box::new(S0211 {
                position: self.position,
                symbol: self.to_string(),
            })),
        }
    }

    fn led(&self, parser: &mut Parser, mut left: Box<Node>) -> Result<Box<Node>> {
        macro_rules! binary {
            ($n:tt) => {
                Ok(Box::new(Node::new(
                    NodeKind::Binary(BinaryOp::$n, left, parser.expression(self.lbp())?),
                    self.position,
                )))
            };
        }

        match self.kind {
            TokenKind::Period => binary!(Path),
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
                                args.push(Box::new(Node::new(
                                    NodeKind::PartialArg,
                                    parser.token().position,
                                )));
                                parser.expect(TokenKind::Question, false)?;
                            }
                            _ => {
                                args.push(parser.expression(0)?);
                            }
                        }
                        if parser.token().kind != TokenKind::Comma {
                            break;
                        }
                        parser.expect(TokenKind::Comma, false)?;
                    }
                }
                parser.expect(TokenKind::RightParen, true)?;

                // If the name of the function is 'function' or λ, then this is a function definition (lambda function)
                if let NodeKind::Name(ref name) = left.kind {
                    if name == "function" || name == "λ" {
                        is_lambda = true;

                        // All of the args must be Variable nodes
                        for arg in &args {
                            if !matches!(arg.kind, NodeKind::Var(..)) {
                                return Err(Box::new(S0208 {
                                    position: arg.position,
                                    arg: arg.kind.to_string(),
                                }));
                            }
                        }

                        // TODO: Parse function signatures
                    }
                }

                let func: Box<Node>;

                if is_lambda {
                    parser.expect(TokenKind::LeftBrace, false)?;
                    let body = parser.expression(0)?;
                    func = Box::new(Node::new(NodeKind::Lambda { args, body }, self.position));
                    parser.expect(TokenKind::RightBrace, false)?;
                } else {
                    func = Box::new(Node::new(
                        NodeKind::Function {
                            proc: left,
                            args,
                            is_partial,
                        },
                        self.position,
                    ));
                }

                Ok(func)
            }

            // Variable assignment
            TokenKind::Bind => {
                if !matches!(left.kind, NodeKind::Var(..)) {
                    return Err(Box::new(S0212 {
                        position: left.position,
                    }));
                }

                Ok(Box::new(Node::new(
                    NodeKind::Binary(BinaryOp::Bind, left, parser.expression(self.lbp() - 1)?),
                    self.position,
                )))
            }

            // Order by expression
            TokenKind::Caret => {
                let mut terms = Vec::new();

                parser.expect(TokenKind::LeftParen, false)?;
                loop {
                    let mut descending = false;
                    if parser.token().kind == TokenKind::LeftCaret {
                        parser.expect(TokenKind::LeftCaret, false)?;
                    } else if parser.token().kind == TokenKind::RightCaret {
                        parser.expect(TokenKind::RightCaret, false)?;
                        descending = true;
                    }

                    terms.push(Box::new(Node::new(
                        NodeKind::SortTerm(parser.expression(0)?, descending),
                        self.position,
                    )));

                    if parser.token().kind != TokenKind::Comma {
                        break;
                    }
                    parser.expect(TokenKind::Comma, false)?;
                }
                parser.expect(TokenKind::RightParen, false)?;

                Ok(Box::new(Node::new(
                    NodeKind::SortOp(left, terms),
                    self.position,
                )))
            }

            // Context variable bind
            TokenKind::At => {
                let rhs = parser.expression(self.lbp())?;

                if !matches!(rhs.kind, NodeKind::Var(..)) {
                    return Err(Box::new(S0214 {
                        position: rhs.position,
                        op: '@'.to_string(),
                    }));
                }

                Ok(Box::new(Node::new(
                    NodeKind::Binary(BinaryOp::ContextBind, left, rhs),
                    self.position,
                )))
            }

            // Positional variable bind
            TokenKind::Hash => {
                let rhs = parser.expression(self.lbp())?;

                if !matches!(rhs.kind, NodeKind::Var(..)) {
                    return Err(Box::new(S0214 {
                        position: rhs.position,
                        op: '#'.to_string(),
                    }));
                }

                Ok(Box::new(Node::new(
                    NodeKind::Binary(BinaryOp::PositionalBind, left, rhs),
                    self.position,
                )))
            }

            // Ternary conditional
            TokenKind::Question => {
                let truthy = parser.expression(0)?;

                let falsy = if parser.token().kind == TokenKind::Colon {
                    parser.expect(TokenKind::Colon, false)?;
                    Some(parser.expression(0)?)
                } else {
                    None
                };

                Ok(Box::new(Node::new(
                    NodeKind::Ternary {
                        cond: left,
                        truthy,
                        falsy,
                    },
                    self.position,
                )))
            }

            // Object group by
            TokenKind::LeftBrace => Ok(Box::new(Node::new(
                NodeKind::GroupBy(left, parse_object(parser)?),
                self.position,
            ))),

            // Array predicate or index
            TokenKind::LeftBracket => {
                if parser.token().kind == TokenKind::RightBracket {
                    // Empty predicate means maintain singleton arrays in the output

                    let mut step = &mut left;

                    // Walk back through left hand sides to find something that's not an array
                    // predicate
                    while let NodeKind::Binary(BinaryOp::Predicate, ref mut left, ..) = step.kind {
                        step = left
                    }

                    step.keep_array = true;

                    parser.expect(TokenKind::RightBracket, false)?;

                    Ok(left)
                } else {
                    let rhs = parser.expression(0)?;
                    parser.expect(TokenKind::RightBracket, true)?;
                    Ok(Box::new(Node::new(
                        NodeKind::Binary(BinaryOp::Predicate, left, rhs),
                        self.position,
                    )))
                }
            }

            _ => Err(Box::new(S0201 {
                position: self.position,
                value: self.to_string(),
            })),
        }
    }
}

/// Parses an object definition.
fn parse_object(parser: &mut Parser) -> Result<Object> {
    let mut object: Object = Vec::new();
    if parser.token().kind != TokenKind::RightBrace {
        loop {
            let key = parser.expression(0)?;
            parser.expect(TokenKind::Colon, false)?;
            let value = parser.expression(0)?;
            object.push((key, value));
            if parser.token().kind != TokenKind::Comma {
                break;
            }
            parser.expect(TokenKind::Comma, false)?;
        }
    }
    parser.expect(TokenKind::RightBrace, true)?;
    Ok(object)
}
