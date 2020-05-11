use crate::ast::*;
use crate::parser::Parser;

#[derive(Debug, Clone)]
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

impl Token {
    /// The left binding power of the token
    pub fn lbp(&self) -> u32 {
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
    pub fn nud(&self, parser: &mut Parser) -> Box<dyn Node> {
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
            _ => unimplemented!("nud not implemented for token"),
        }
    }

    /// Produce the left denotation for the token
    pub fn led(&self, parser: &mut Parser, left: Box<dyn Node>) -> Box<dyn Node> {
        match &self.kind {
            //            TokenKind::Period => Box::new(MapNode {
            //               lhs: left,
            //              rhs: parser.expression(self.lbp()),
            //         }),
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
            _ => unimplemented!("led not implemented for token"),
        }
    }
}
