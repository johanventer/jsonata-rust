use crate::ast::*;
use crate::error::*;
use crate::parser::Parser;
use crate::tokenizer::{Token, TokenKind};
use crate::JsonAtaResult;

/// Represents a symbol, which is essentially an enhanced token that performs its own parsing and
/// creates syntax trees.
pub trait Symbol {
    /// Returns the left binding power for the symbol.
    fn lbp(&self) -> u32;

    /// Returns the `null denotation` for the symbol, essentially this is the AST created when this
    /// symbol is treated as a prefix (and doesn't care about tokens to the left of it).
    fn nud(&self, parser: &mut Parser) -> JsonAtaResult<Node>;

    /// Returns the `left denotation` for the symbol, essentially this is the AST created when this
    /// symbol is treated as infix (and cares about tokens to the left of it).
    fn led(&self, parser: &mut Parser, left: Node) -> JsonAtaResult<Node>;
}

/// Parses an object definition.
fn object_parse(parser: &mut Parser, vec: &mut Vec<Node>) -> JsonAtaResult<()> {
    if parser.token().kind != TokenKind::RightBrace {
        loop {
            let name = parser.expression(0)?;
            parser.expect(TokenKind::Colon, false)?;
            let value = parser.expression(0)?;
            vec.push(name);
            vec.push(value);
            if parser.token().kind != TokenKind::Comma {
                break;
            }
            parser.expect(TokenKind::Comma, false)?;
        }
    }
    parser.expect(TokenKind::RightBrace, true)?;
    Ok(())
}

/// This is the majority of the parsing logic, constructed as an implementation of `Symbol` for all
/// types of `Token`.
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

    fn nud(&self, parser: &mut Parser) -> JsonAtaResult<Node> {
        type T = TokenKind;
        type N = NodeKind;

        let p = self.position;

        match &self.kind {
            T::Null => Ok(Node::new(N::Null, p)),
            T::Bool(v) => Ok(Node::new(N::Bool(*v), p)),
            T::Str(v) => Ok(Node::new(N::Str(v.clone()), p)),
            T::Num(v) => Ok(Node::new(N::Num(*v), p)),
            T::Name(v) => Ok(Node::new(N::Name(v.clone()), p)),
            T::Var(v) => Ok(Node::new(N::Var(v.clone()), p)),
            T::And => Ok(Node::new(N::Name(String::from("and")), p)),
            T::Or => Ok(Node::new(N::Name(String::from("or")), p)),
            T::In => Ok(Node::new(N::Name(String::from("in")), p)),
            T::Minus => Ok(Node::new_with_child(
                N::Unary(UnaryOp::Minus),
                p,
                parser.expression(70)?,
            )),
            T::Wildcard => Ok(Node::new(N::Wildcard, p)),
            T::Descendent => Ok(Node::new(N::Descendent, p)),
            T::Percent => Ok(Node::new(N::Parent(None), p)),

            // Block of expressions
            T::LeftParen => {
                let mut expressions = Vec::new();

                while parser.token().kind != T::RightParen {
                    expressions.push(parser.expression(0)?);
                    if parser.token().kind != T::SemiColon {
                        break;
                    }
                    parser.expect(T::SemiColon, false)?;
                }
                parser.expect(T::RightParen, true)?;

                Ok(Node::new_with_children(N::Block, p, expressions))
            }

            // Array constructor
            T::LeftBracket => {
                let mut expressions = Vec::new();
                if parser.token().kind != T::RightBracket {
                    loop {
                        let mut item = parser.expression(0)?;
                        if parser.token().kind == T::Range {
                            let position = parser.token().position;
                            parser.expect(T::Range, false)?;
                            item = Node::new_with_children(
                                N::Binary(BinaryOp::Range),
                                p,
                                vec![item, parser.expression(0)?],
                            )
                        }
                        expressions.push(item);
                        if parser.token().kind != TokenKind::Comma {
                            break;
                        }
                        parser.expect(TokenKind::Comma, false)?;
                    }
                }
                parser.expect(T::RightBracket, true)?;

                Ok(Node::new_with_children(
                    N::Unary(UnaryOp::Array),
                    p,
                    expressions,
                ))
            }

            // Object constructor
            T::LeftBrace => {
                let mut children = Vec::new();
                object_parse(parser, &mut children)?;
                Ok(Node::new_with_children(
                    N::Unary(UnaryOp::Object),
                    p,
                    children,
                ))
            }

            // Object transformer
            T::Pipe => {
                // Pattern
                let mut children = vec![parser.expression(0)?];
                parser.expect(T::Pipe, false)?;

                // Update
                children.push(parser.expression(0)?);

                // Delete
                if parser.token().kind == T::Comma {
                    parser.expect(T::Comma, false)?;
                    children.push(parser.expression(0)?);
                }

                parser.expect(T::Pipe, false)?;

                Ok(Node::new_with_children(N::Transform, p, children))
            }

            _ => Err(Box::new(S0211 {
                position: p,
                symbol: self.to_string(),
            })),
        }
    }

    /// Produce the left denotation for the token
    fn led(&self, parser: &mut Parser, mut left: Node) -> JsonAtaResult<Node> {
        type T = TokenKind;
        type N = NodeKind;

        let p = self.position;

        macro_rules! binary {
            ($n:tt) => {
                Ok(Node::new_with_children(
                    N::Binary(BinaryOp::$n),
                    p,
                    vec![left, parser.expression(self.lbp())?],
                ))
            };
        }

        match &self.kind {
            T::Period => binary!(Path),
            T::Plus => binary!(Add),
            T::Minus => binary!(Subtract),
            T::Wildcard => binary!(Multiply),
            T::ForwardSlash => binary!(Divide),
            T::Percent => binary!(Modulus),
            T::Equal => binary!(Equal),
            T::LeftCaret => binary!(LessThan),
            T::RightCaret => binary!(GreaterThan),
            T::NotEqual => binary!(NotEqual),
            T::LessEqual => binary!(LessThanEqual),
            T::GreaterEqual => binary!(GreaterThanEqual),
            T::Ampersand => binary!(Concat),
            T::And => binary!(And),
            T::Or => binary!(Or),
            T::In => binary!(In),
            T::Apply => binary!(Apply),

            // Function calls or lambda definitions
            T::LeftParen => {
                let mut arguments = Vec::new();
                let mut is_partial = false;
                let mut is_function_def = false;

                if parser.token().kind != T::RightParen {
                    loop {
                        match parser.token().kind {
                            T::Question => {
                                is_partial = true;
                                arguments.push(Node::new(N::PartialArg, parser.token().position));
                                parser.expect(T::Question, false)?;
                            }
                            _ => {
                                arguments.push(parser.expression(0)?);
                            }
                        }
                        if parser.token().kind != T::Comma {
                            break;
                        }
                        parser.expect(T::Comma, false)?;
                    }
                }
                parser.expect(T::RightParen, true)?;

                // If the name of the function is 'function' or λ, then this is a function definition (lambda function)
                if let N::Name(ref name) = left.kind {
                    if name == "function" || name == "λ" {
                        is_function_def = true;

                        // All of the args must be Variable nodes
                        for arg in &arguments {
                            match arg.kind {
                                N::Var(..) => (),
                                _ => {
                                    return Err(Box::new(S0208 {
                                        position: arg.position,
                                        arg: arg.to_string(),
                                    }))
                                }
                            }
                        }

                        // TODO: Parse function signatures
                    }
                }

                let func: Node;

                if is_function_def {
                    parser.expect(T::LeftBrace, false)?;

                    // Body of the lambda function
                    arguments.push(parser.expression(0)?);

                    func = Node::new_with_children(N::Lambda, p, arguments);

                    parser.expect(T::RightBrace, false)?;
                } else {
                    arguments.push(left);
                    func = Node::new_with_children(N::Function(is_partial), p, arguments);
                }

                Ok(func)
            }

            // Variable assignment
            T::Bind => {
                match left.kind {
                    N::Var(..) => (),
                    _ => {
                        return Err(Box::new(S0212 {
                            position: left.position,
                        }))
                    }
                }

                Ok(Node::new_with_children(
                    N::Binary(BinaryOp::Bind),
                    p,
                    vec![left, parser.expression(self.lbp() - 1)?],
                ))
            }

            // Order by expression
            T::Caret => {
                let mut children = vec![left];

                parser.expect(T::LeftParen, false)?;
                loop {
                    let mut descending = false;
                    if parser.token().kind == T::LeftCaret {
                        parser.expect(T::LeftCaret, false)?;
                    } else if parser.token().kind == T::RightCaret {
                        parser.expect(T::RightCaret, false)?;
                        descending = true;
                    }

                    children.push(Node::new_with_child(
                        N::SortTerm(descending),
                        p,
                        parser.expression(0)?,
                    ));

                    if parser.token().kind != T::Comma {
                        break;
                    }
                    parser.expect(T::Comma, false)?;
                }
                parser.expect(T::RightParen, false)?;

                Ok(Node::new_with_children(N::Sort, p, children))
            }

            // Context variable bind
            T::At => {
                let rhs = parser.expression(self.lbp())?;
                match rhs.kind {
                    N::Var(..) => (),
                    _ => {
                        return Err(Box::new(S0214 {
                            position: rhs.position,
                            op: '@'.to_string(),
                        }))
                    }
                }

                Ok(Node::new_with_children(
                    N::Binary(BinaryOp::ContextBind),
                    p,
                    vec![left, rhs],
                ))
            }

            // Positional variable bind
            T::Hash => {
                let rhs = parser.expression(self.lbp())?;
                match rhs.kind {
                    N::Var(..) => (),
                    _ => {
                        return Err(Box::new(S0214 {
                            position: rhs.position,
                            op: '#'.to_string(),
                        }))
                    }
                }

                Ok(Node::new_with_children(
                    N::Binary(BinaryOp::PositionalBind),
                    p,
                    vec![left, rhs],
                ))
            }

            // Ternary conditional
            T::Question => {
                let mut children = vec![left, parser.expression(0)?];

                if parser.token().kind == T::Colon {
                    parser.expect(T::Colon, false)?;
                    children.push(parser.expression(0)?);
                }

                Ok(Node::new_with_children(N::Ternary, p, children))
            }

            // Object group by
            T::LeftBrace => {
                let mut children = vec![left];
                object_parse(parser, &mut children)?;
                Ok(Node::new_with_children(
                    N::Unary(UnaryOp::Object),
                    p,
                    children,
                ))
            }

            // Array predicate or index
            T::LeftBracket => {
                if parser.token().kind == T::RightBracket {
                    // Empty predicate means maintain singleton arrays in the output

                    let mut step = &mut left;

                    // Walk back through left hand sides to find something that's not an array
                    // predicate
                    while let N::Binary(BinaryOp::ArrayPredicate) = step.kind {
                        step = &mut step.children[0]
                    }

                    step.keep_array = true;
                    parser.expect(T::RightBracket, false)?;
                    Ok(left)
                } else {
                    let rhs = parser.expression(0)?;
                    parser.expect(T::RightBracket, true)?;
                    Ok(Node::new_with_children(
                        N::Binary(BinaryOp::ArrayPredicate),
                        p,
                        vec![left, rhs],
                    ))
                }
            }

            _ => unimplemented!("led not implemented for token {:#?}", self),
        }
    }
}
