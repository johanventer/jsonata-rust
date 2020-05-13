use crate::ast::*;
use crate::error::Error;
use crate::parser::Parser;
use crate::tokenizer::{Token, TokenKind};

pub trait Symbol {
    fn lbp(&self) -> u32;
    fn nud(&self, parser: &mut Parser) -> Box<dyn Node>;
    fn led(&self, parser: &mut Parser, left: Box<dyn Node>) -> Box<dyn Node>;
}

impl Symbol for Token {
    /// The left binding power of the token
    fn lbp(&self) -> u32 {
        match &self.kind {
            // Double character operators
            TokenKind::Range => 20,
            TokenKind::Assignment => 10,
            TokenKind::NotEqual => 40,
            TokenKind::GreaterEqual => 40,
            TokenKind::LessEqual => 40,
            TokenKind::DescendantWildcard => 60,
            TokenKind::ChainFunction => 40,
            // Named operators
            TokenKind::And => 30,
            TokenKind::Or => 25,
            TokenKind::In => 40,
            // Single character operators
            TokenKind::Period => 75,
            TokenKind::LeftBracket => 80,
            TokenKind::RightBracket => 0,
            TokenKind::LeftBrace => 70,
            TokenKind::RightBrace => 0,
            TokenKind::LeftParen => 80,
            TokenKind::RightParen => 0,
            TokenKind::Comma => 0,
            TokenKind::At => 80,
            TokenKind::Hash => 80,
            TokenKind::SemiColon => 80,
            TokenKind::Colon => 80,
            TokenKind::Question => 20,
            TokenKind::Add => 50,
            TokenKind::Sub => 50,
            TokenKind::Mul => 60,
            TokenKind::Div => 60,
            TokenKind::Mod => 60,
            TokenKind::Pipe => 20,
            TokenKind::Equ => 40,
            TokenKind::RightCaret => 40,
            TokenKind::LeftCaret => 40,
            TokenKind::Pow => 40,
            TokenKind::Ampersand => 50,
            TokenKind::Not => 0,
            TokenKind::Tilde => 0,
            // Literal values
            TokenKind::Null => 0,
            TokenKind::Boolean(..) => 0,
            TokenKind::String(..) => 0,
            TokenKind::Number(..) => 0,
            // Identifiers
            TokenKind::Name(..) => 0,
            TokenKind::Variable(..) => 0,
        }
    }

    /// Produce the null denotation for the token
    fn nud(&self, parser: &mut Parser) -> Box<dyn Node> {
        match &self.kind {
            TokenKind::Null => Box::new(NullNode {
                position: self.position,
            }),
            TokenKind::Boolean(value) => Box::new(BooleanNode {
                position: self.position,
                value: value.clone(),
            }),
            TokenKind::String(value) => Box::new(StringNode {
                position: self.position,
                value: value.clone(),
            }),
            TokenKind::Number(value) => Box::new(NumberNode {
                position: self.position,
                value: value.clone(),
            }),
            TokenKind::Name(value) => Box::new(NameNode {
                position: self.position,
                value: value.clone(),
            }),
            TokenKind::Variable(value) => Box::new(VariableNode {
                position: self.position,
                value: value.clone(),
            }),
            TokenKind::And => Box::new(NameNode {
                position: self.position,
                value: "and".to_string(),
            }),
            TokenKind::Or => Box::new(NameNode {
                position: self.position,
                value: "or".to_string(),
            }),
            TokenKind::In => Box::new(NameNode {
                position: self.position,
                value: "in".to_string(),
            }),
            TokenKind::Sub => Box::new(UnaryMinusNode {
                position: self.position,
                expression: parser.expression(70),
            }),
            TokenKind::Mul => Box::new(WildcardNode {
                position: self.position,
            }),
            TokenKind::DescendantWildcard => Box::new(DescendantWildcardNode {
                position: self.position,
            }),
            TokenKind::Mod => Box::new(ParentNode {
                position: self.position,
            }),
            _ => panic!(format!(
                "{:#?}",
                Error {
                    code: "S0211",
                    position: self.position,
                    message: format!("The symbol {} cannot be used as a unary operator", self)
                }
            )),
        }
    }

    /// Produce the left denotation for the token
    fn led(&self, parser: &mut Parser, left: Box<dyn Node>) -> Box<dyn Node> {
        match &self.kind {
            TokenKind::Period => Box::new(MapNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            }),
            TokenKind::Add => Box::new(AddNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            }),
            TokenKind::Sub => Box::new(SubtractNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            }),
            TokenKind::Mul => Box::new(MultiplyNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            }),
            TokenKind::Div => Box::new(DivideNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            }),
            TokenKind::Mod => Box::new(ModulusNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            }),
            TokenKind::Equ => Box::new(EqualNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            }),
            TokenKind::LeftCaret => Box::new(LessThanNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            }),
            TokenKind::RightCaret => Box::new(GreaterThanNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            }),
            TokenKind::NotEqual => Box::new(NotEqualNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            }),
            TokenKind::LessEqual => Box::new(LessEqualNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            }),
            TokenKind::GreaterEqual => Box::new(GreaterEqualNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            }),
            TokenKind::Ampersand => Box::new(ConcatNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            }),
            TokenKind::And => Box::new(AndNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            }),
            TokenKind::Or => Box::new(OrNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            }),
            TokenKind::In => Box::new(InNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            }),
            TokenKind::ChainFunction => Box::new(ChainFunctionNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            }),
            //                        TokenKind::LeftParen => {
            //                            let mut arguments = Vec::new();
            //                            let mut is_partial = false;
            //
            //                            if parser.token().kind != TokenKind::RightParen {
            //                                loop {
            //                                    match parser.token().kind {
            //                                        TokenKind::Question => {
            //                                            is_partial = true;
            //                                            arguments.push(PartialArgNode {
            //                                                position: parser.token().position
            //                                            });
            //                                        }
            //
            //
            //                                    }
            //
            //                                    parser.next();
            //                                }
            //                            }
            //
            //                            parser.expect(TokenKind::LeftParen);
            //
            //                            Box::new(FunctionNode {
            //                            position: self.position,
            //
            //
            //                        })
            //                        },
            _ => unimplemented!("led not implemented for token"),
        }
    }
}
