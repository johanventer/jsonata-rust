use crate::ast::*;
use crate::error::*;
use crate::parser::Parser;
use crate::tokenizer::{Token, TokenKind};

/// Represents a symbol, which is essentially an enhanced token that performs its own parsing and
/// creates syntax trees.
pub trait Symbol {
   /// Returns the left binding power for the symbol.
   fn lbp(&self) -> u32;

   /// Returns the `null denotation` for the symbol, essentially this is the AST created when this
   /// symbol is treated as a prefix (and doesn't care about tokens to the left of it).
   fn nud(&self, parser: &mut Parser) -> Box<Node>;

   /// Returns the `left denotation` for the symbol, essentially this is the AST created when this
   /// symbol is treated as infix (and cares about tokens to the left of it).
   fn led(&self, parser: &mut Parser, left: Box<Node>) -> Box<Node>;
}

/// Parses an object definition.
///
/// An object is defined by a vector of tuples consisting of (name, value), both of which are AST
/// nodes themselves.
fn object_parse(parser: &mut Parser) -> Vec<(Box<Node>, Box<Node>)> {
   let mut props = Vec::new();
   if parser.token().kind != TokenKind::RightBrace {
      loop {
         let name = parser.expression(0);
         parser.expect(TokenKind::Colon, false);
         let value = parser.expression(0);
         props.push((name, value));
         if parser.token().kind != TokenKind::Comma {
            break;
         }
         parser.expect(TokenKind::Comma, false);
      }
   }
   parser.expect(TokenKind::RightBrace, true);
   props
}

/// This is the majority of the parsing logic, constructed as an implementation of `Symbol` for all
/// types of `Token`.
impl Symbol for Token {
   fn lbp(&self) -> u32 {
      use TokenKind::*;
      match &self.kind {
         End | Range | Colon | Comma | SemiColon | RightParen | RightBracket | RightBrace
         | Pipe | Not | Tilde | Null | Boolean(..) | Str(..) | Number(..) | Name(..)
         | Variable(..) => 0,
         Assignment => 10,
         Question => 20,
         Or => 25,
         And => 30,
         NotEqual | GreaterEqual | LessEqual | ChainFunction | In | Equal | RightCaret
         | LeftCaret | Caret => 40,
         Ampersand | Plus | Minus => 50,
         Wildcard | Descendent | ForwardSlash | Percent => 60,
         LeftBrace => 70,
         Period => 75,
         LeftBracket | LeftParen => 80,
         At | Hash => 80,
      }
   }

   fn nud(&self, parser: &mut Parser) -> Box<Node> {
      use TokenKind::*;
      match &self.kind {
         Null => Box::new(Node::Null(LiteralNode::new(self.position, NullValue {}))),
         Boolean(value) => Box::new(Node::Boolean(LiteralNode::new(self.position, *value))),
         Str(value) => Box::new(Node::Str(LiteralNode::new(self.position, value.clone()))),
         Number(value) => Box::new(Node::Number(LiteralNode::new(self.position, *value))),
         Name(value) => Box::new(Node::Name(LiteralNode::new(self.position, value.clone()))),
         Variable(value) => Box::new(Node::Variable(LiteralNode::new(
            self.position,
            value.clone(),
         ))),
         And => Box::new(Node::Name(LiteralNode::new(
            self.position,
            "and".to_string(),
         ))),
         Or => Box::new(Node::Name(LiteralNode::new(
            self.position,
            "or".to_string(),
         ))),
         In => Box::new(Node::Name(LiteralNode::new(
            self.position,
            "in".to_string(),
         ))),
         Minus => Box::new(Node::UnaryMinus(UnaryNode {
            position: self.position,
            expression: parser.expression(70),
         })),
         Wildcard => Box::new(Node::Wildcard(MarkerNode {
            position: self.position,
         })),
         Descendent => Box::new(Node::DescendantWildcard(MarkerNode {
            position: self.position,
         })),
         Percent => Box::new(Node::ParentOp(MarkerNode {
            position: self.position,
         })),
         // Parenthesis - block expression
         LeftParen => {
            let mut expressions = Vec::new();

            while parser.token().kind != TokenKind::RightParen {
               expressions.push(parser.expression(0));
               if parser.token().kind != TokenKind::SemiColon {
                  break;
               }
               parser.expect(TokenKind::SemiColon, false);
            }
            parser.expect(TokenKind::RightParen, true);

            Box::new(Node::Block(ExpressionsNode::new(
               self.position,
               expressions,
            )))
         }
         // Array constructor
         LeftBracket => {
            let mut expressions = Vec::new();
            if parser.token().kind != TokenKind::RightBracket {
               loop {
                  let mut item = parser.expression(0);
                  if parser.token().kind == TokenKind::Range {
                     let position = parser.token().position;
                     parser.expect(TokenKind::Range, false);
                     item = Box::new(Node::Range(BinaryNode {
                        position,
                        lhs: item,
                        rhs: parser.expression(0),
                     }));
                  }
                  expressions.push(item);
                  if parser.token().kind != TokenKind::Comma {
                     break;
                  }
                  parser.expect(TokenKind::Comma, false);
               }
            }
            parser.expect(TokenKind::RightBracket, true);
            Box::new(Node::Array(ExpressionsNode::new(
               self.position,
               expressions,
            )))
         }
         // Object - unary prefix form
         LeftBrace => {
            let object = object_parse(parser);
            Box::new(Node::Object(ObjectNode {
               position: self.position,
               lhs: object,
            }))
         }
         // Object transformer
         Pipe => {
            let pattern = parser.expression(0);
            parser.expect(Pipe, false);
            let update = parser.expression(0);
            let delete = if parser.token().kind == Comma {
               parser.expect(Comma, false);
               Some(parser.expression(0))
            } else {
               None
            };
            parser.expect(Pipe, false);
            Box::new(Node::Transform(TransformNode {
               position: self.position,
               pattern,
               update,
               delete,
            }))
         }
         _ => error!(s0211, self.position, self),
      }
   }

   /// Produce the left denotation for the token
   fn led(&self, parser: &mut Parser, mut left: Box<Node>) -> Box<Node> {
      macro_rules! binary {
         ($n:tt) => {
            Box::new(Node::$n(BinaryNode {
               position: self.position,
               lhs: left,
               rhs: parser.expression(self.lbp()),
            }))
         };
      }

      use TokenKind::*;
      match &self.kind {
         Period => binary!(PathSeparator),
         Plus => binary!(Add),
         Minus => binary!(Subtract),
         Wildcard => binary!(Multiply),
         ForwardSlash => binary!(Divide),
         Percent => binary!(Modulus),
         Equal => binary!(Equal),
         LeftCaret => binary!(LessThan),
         RightCaret => binary!(GreaterThan),
         NotEqual => binary!(NotEqual),
         LessEqual => binary!(LessThanEqual),
         GreaterEqual => binary!(GreaterThanEqual),
         Ampersand => binary!(Concat),
         And => binary!(And),
         Or => binary!(Or),
         In => binary!(In),
         ChainFunction => binary!(Chain),
         LeftParen => {
            let mut arguments = Vec::new();
            let mut is_partial = false;
            let mut is_function_def = false;

            if parser.token().kind != TokenKind::RightParen {
               loop {
                  match parser.token().kind {
                     TokenKind::Question => {
                        is_partial = true;
                        arguments.push(Box::new(Node::PartialFunctionArg(MarkerNode {
                           position: parser.token().position,
                        })));
                        parser.expect(TokenKind::Question, false);
                     }
                     _ => {
                        arguments.push(parser.expression(0));
                     }
                  }
                  if parser.token().kind != TokenKind::Comma {
                     break;
                  }
                  parser.expect(TokenKind::Comma, false);
               }
            }
            parser.expect(TokenKind::RightParen, true);

            // If the name of the function is 'function' or λ, then this is a function definition (lambda function)
            if let Node::Name(literal) = left.as_ref() {
               if literal.value == "function" || literal.value == "\x03BB" {
                  is_function_def = true;

                  // All of the args must be Variable nodes
                  for arg in &arguments {
                     match arg.as_ref() {
                        Node::Variable(_) => (),
                        _ => error!(s0208, arg.get_position(), &arg.get_value()),
                     }
                  }

                  // TODO: Parse function signatures
               }
            }

            let func: Box<Node>;

            if is_function_def {
               parser.expect(TokenKind::LeftBrace, false);
               func = Box::new(Node::LambdaFunction(LambdaNode {
                  position: self.position,
                  arguments,
                  body: parser.expression(0),
               }));
               parser.expect(TokenKind::RightBrace, false);
            } else if is_partial {
               func = Box::new(Node::PartialFunctionCall(FunctionCallNode {
                  position: self.position,
                  procedure: left,
                  arguments,
               }))
            } else {
               func = Box::new(Node::FunctionCall(FunctionCallNode {
                  position: self.position,
                  procedure: left,
                  arguments,
               }))
            }

            func
         }
         Assignment => {
            match left.as_ref() {
               Node::Variable(_) => (),
               _ => error!(s0212, left.get_position()),
            }

            Box::new(Node::Assignment(BinaryNode {
               position: self.position,
               lhs: left,
               rhs: parser.expression(self.lbp() - 1),
            }))
         }
         // Order by
         Caret => {
            let mut terms = Vec::new();

            parser.expect(TokenKind::LeftParen, false);
            loop {
               let mut descending = false;
               if parser.token().kind == LeftCaret {
                  parser.expect(LeftCaret, false);
               } else if parser.token().kind == RightCaret {
                  parser.expect(RightCaret, false);
                  descending = true;
               } else {
                  // Unspecified, default to ascending
               }
               terms.push(SortTermNode {
                  position: self.position,
                  descending,
                  expression: parser.expression(0),
               });
               if parser.token().kind != Comma {
                  break;
               }
               parser.expect(Comma, false);
            }
            parser.expect(RightParen, false);

            Box::new(Node::OrderBy(OrderByNode {
               position: self.position,
               lhs: left,
               rhs: terms,
            }))
         }
         // Focus variable bind
         At => {
            let rhs = parser.expression(self.lbp());
            match rhs.as_ref() {
               Node::Variable(_) => (),
               _ => error!(s0214, rhs.get_position(), "@"),
            }
            Box::new(Node::FocusVariableBind(BinaryNode {
               position: self.position,
               lhs: left,
               rhs,
            }))
         }
         // Index (position) variable bind
         Hash => {
            let rhs = parser.expression(self.lbp());
            match rhs.as_ref() {
               Node::Variable(_) => (),
               _ => error!(s0214, rhs.get_position(), "#"),
            }
            Box::new(Node::IndexVariableBind(BinaryNode {
               position: self.position,
               lhs: left,
               rhs,
            }))
         }
         // Ternary operator ?:
         Question => {
            let then = parser.expression(0);
            let els = if parser.token().kind == Colon {
               parser.expect(Colon, false);
               Some(parser.expression(0))
            } else {
               None
            };
            Box::new(Node::Ternary(TernaryNode {
               position: self.position,
               condition: left,
               then,
               els,
            }))
         }
         // Object group by
         LeftBrace => {
            let object = object_parse(parser);
            Box::new(Node::ObjectGroup(ObjectGroupNode {
               position: self.position,
               lhs: left,
               rhs: object,
            }))
         }
         // Filter - predicate or array index
         LeftBracket => {
            if parser.token().kind == RightBracket {
               // Empty predicate means maintain singleton arrays in the output
               let mut step = left.as_mut();
               while let Node::ArrayPredicate(pred) = step {
                  step = pred.lhs.as_mut();
               }

               match step {
                  Node::Name(literal) => {
                     literal.keep_array = true;
                  },
                  _ => unreachable!("TODO: You cannot specify a singleton predicate on anything other than Node::Name")
               }

               parser.expect(RightBracket, false);
               left
            } else {
               let rhs = parser.expression(0);
               parser.expect(RightBracket, true);
               Box::new(Node::ArrayPredicate(BinaryNode {
                  position: self.position,
                  lhs: left,
                  rhs,
               }))
            }
         }
         _ => unimplemented!("led not implemented for token {:#?}", self),
      }
   }
}
