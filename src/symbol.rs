use crate::ast::*;
use crate::parser::Parser;
use crate::tokenizer::{Token, TokenKind};

pub trait Symbol {
    fn lbp(&self) -> u32;
    fn nud(&self, parser: &mut Parser) -> Box<Node>;
    fn led(&self, parser: &mut Parser, left: Box<Node>) -> Box<Node>;
}

impl Symbol for Token {
    /// The left binding power of the token
    fn lbp(&self) -> u32 {
        use TokenKind::*;
        match &self.kind {
            End => 0,
            // Double character operators
            Range => 20,
            Assignment => 10,
            NotEqual => 40,
            GreaterEqual => 40,
            LessEqual => 40,
            DescendantWildcard => 60,
            ChainFunction => 40,
            // Named operators
            And => 30,
            Or => 25,
            In => 40,
            // Single character operators
            Period => 75,
            LeftBracket => 80,
            RightBracket => 0,
            LeftBrace => 70,
            RightBrace => 0,
            LeftParen => 80,
            RightParen => 0,
            Comma => 0,
            At => 80,
            Hash => 80,
            SemiColon => 0,
            Colon => 80,
            Question => 20,
            Add => 50,
            Sub => 50,
            Mul => 60,
            Div => 60,
            Mod => 60,
            Pipe => 20,
            Equ => 40,
            RightCaret => 40,
            LeftCaret => 40,
            Pow => 40,
            Ampersand => 50,
            Not => 0,
            Tilde => 0,
            // Literal values
            Null => 0,
            Boolean(..) => 0,
            Str(..) => 0,
            Number(..) => 0,
            // Identifiers
            Name(..) => 0,
            Variable(..) => 0,
        }
    }

    /// Produce the null denotation for the token
    fn nud(&self, parser: &mut Parser) -> Box<Node> {
        use TokenKind::*;
        match &self.kind {
            Null => Box::new(Node::Null(LiteralNode {
                position: self.position,
                value: NullValue {},
            })),
            Boolean(value) => Box::new(Node::Boolean(LiteralNode {
                position: self.position,
                value: *value,
            })),
            Str(value) => Box::new(Node::String(LiteralNode {
                position: self.position,
                value: value.clone(),
            })),
            Number(value) => Box::new(Node::Number(LiteralNode {
                position: self.position,
                value: *value,
            })),
            Name(value) => Box::new(Node::Name(LiteralNode {
                position: self.position,
                value: value.clone(),
            })),
            Variable(value) => Box::new(Node::Variable(LiteralNode {
                position: self.position,
                value: value.clone(),
            })),
            And => Box::new(Node::Name(LiteralNode {
                position: self.position,
                value: "and".to_string(),
            })),
            Or => Box::new(Node::Name(LiteralNode {
                position: self.position,
                value: "or".to_string(),
            })),
            In => Box::new(Node::Name(LiteralNode {
                position: self.position,
                value: "in".to_string(),
            })),
            Sub => Box::new(Node::UnaryMinus(UnaryNode {
                position: self.position,
                expression: parser.expression(70),
            })),
            Mul => Box::new(Node::Wildcard(BasicNode {
                position: self.position,
            })),
            DescendantWildcard => Box::new(Node::DescendantWildcard(BasicNode {
                position: self.position,
            })),
            Mod => Box::new(Node::Parent(BasicNode {
                position: self.position,
            })),
            // Parenthesis - block expression
            LeftParen => {
                let mut expressions = Vec::new();

                while parser.token().kind != TokenKind::RightParen {
                    expressions.push(*parser.expression(0));
                    if parser.token().kind != TokenKind::SemiColon {
                        break;
                    }
                    parser.expect(TokenKind::SemiColon, false);
                }
                parser.expect(TokenKind::RightParen, true);

                Box::new(Node::Block(ExpressionsNode {
                    position: self.position,
                    expressions,
                }))
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
                        expressions.push(*item);
                        if parser.token().kind != TokenKind::Comma {
                            break;
                        }
                        parser.expect(TokenKind::Comma, false);
                    }
                }
                parser.expect(TokenKind::LeftBracket, true);
                Box::new(Node::Array(ExpressionsNode {
                    position: self.position,
                    expressions,
                }))
            }
            _ => error!(S0211, self.position, self),
        }
    }

    /// Produce the left denotation for the token
    fn led(&self, parser: &mut Parser, left: Box<Node>) -> Box<Node> {
        use TokenKind::*;
        match &self.kind {
            Period => Box::new(Node::PathSeparator(BinaryNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            })),
            Add => Box::new(Node::Add(BinaryNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            })),
            Sub => Box::new(Node::Subtract(BinaryNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            })),
            Mul => Box::new(Node::Multiply(BinaryNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            })),
            Div => Box::new(Node::Divide(BinaryNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            })),
            Mod => Box::new(Node::Modulus(BinaryNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            })),
            Equ => Box::new(Node::Equal(BinaryNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            })),
            LeftCaret => Box::new(Node::LessThan(BinaryNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            })),
            RightCaret => Box::new(Node::GreaterThan(BinaryNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            })),
            NotEqual => Box::new(Node::NotEqual(BinaryNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            })),
            LessEqual => Box::new(Node::LessThanEqual(BinaryNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            })),
            GreaterEqual => Box::new(Node::GreaterThanEqual(BinaryNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            })),
            Ampersand => Box::new(Node::Concat(BinaryNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            })),
            And => Box::new(Node::And(BinaryNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            })),
            Or => Box::new(Node::Or(BinaryNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            })),
            In => Box::new(Node::In(BinaryNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            })),
            ChainFunction => Box::new(Node::Chain(BinaryNode {
                position: self.position,
                lhs: left,
                rhs: parser.expression(self.lbp()),
            })),
            LeftParen => {
                let mut arguments = Vec::new();
                let mut is_partial = false;
                let mut is_function_def = false;

                if parser.token().kind != TokenKind::RightParen {
                    loop {
                        match parser.token().kind {
                            TokenKind::Question => {
                                is_partial = true;
                                arguments.push(Node::PartialFunctionArg(BasicNode {
                                    position: parser.token().position,
                                }));
                                parser.expect(TokenKind::Question, false);
                            }
                            _ => {
                                arguments.push(*parser.expression(0));
                            }
                        }
                        if parser.token().kind != TokenKind::Comma {
                            break;
                        }
                        parser.expect(TokenKind::Comma, false);
                    }
                }
                parser.expect(TokenKind::RightParen, true);

                // If the name of the function is 'function' or λ, then this is function definition (lambda function)
                if let Node::Name(literal) = left.as_ref() {
                    if literal.value == "function" || literal.value == "\x03BB" {
                        is_function_def = true;

                        // All of the args must be Variable nodes
                        for arg in &arguments {
                            match &arg {
                                Node::Variable(_) => (),
                                // TODO: Better error handling
                                Node::Name(literal) => {
                                    error!(S0208, literal.position, literal.value)
                                }
                                _ => error!(S0208, arg.get_position(), "TODO"),
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
                        procedure: left,
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
                    _ => error!(S0212, left.get_position()),
                }

                Box::new(Node::Assignment(BinaryNode {
                    position: self.position,
                    lhs: left,
                    rhs: parser.expression(self.lbp() - 1),
                }))
            }
            _ => unimplemented!("led not implemented for token {:#?}", self),
        }
    }
}

/* TODO

   // filter - predicate or array index
        infix("[", operators['['], function (left) {
            if (node.id === "]") {
                // empty predicate means maintain singleton arrays in the output
                var step = left;
                while (step && step.type === 'binary' && step.value === '[') {
                    step = step.lhs;
                }
                step.keepArray = true;
                advance("]");
                return left;
            } else {
                this.lhs = left;
                this.rhs = expression(operators[']']);
                this.type = 'binary';
                advance("]", true);
                return this;
            }
        });

        // order-by
        infix("^", operators['^'], function (left) {
            advance("(");
            var terms = [];
            for (; ;) {
                var term = {
                    descending: false
                };
                if (node.id === "<") {
                    // ascending sort
                    advance("<");
                } else if (node.id === ">") {
                    // descending sort
                    term.descending = true;
                    advance(">");
                } else {
                    //unspecified - default to ascending
                }
                term.expression = expression(0);
                terms.push(term);
                if (node.id !== ",") {
                    break;
                }
                advance(",");
            }
            advance(")");
            this.lhs = left;
            this.rhs = terms;
            this.type = 'binary';
            return this;
        });

        var objectParser = function (left) {
            var a = [];
            if (node.id !== "}") {
                for (; ;) {
                    var n = expression(0);
                    advance(":");
                    var v = expression(0);
                    a.push([n, v]); // holds an array of name/value expression pairs
                    if (node.id !== ",") {
                        break;
                    }
                    advance(",");
                }
            }
            advance("}", true);
            if (typeof left === 'undefined') {
                // NUD - unary prefix form
                this.lhs = a;
                this.type = "unary";
            } else {
                // LED - binary infix form
                this.lhs = left;
                this.rhs = a;
                this.type = 'binary';
            }
            return this;
        };

        // object constructor
        prefix("{", objectParser);

        // object grouping
        infix("{", operators['{'], objectParser);

  // focus variable bind
        infix("@", operators['@'], function (left) {
            this.lhs = left;
            this.rhs = expression(operators['@']);
            if(this.rhs.type !== 'variable') {
                return handleError({
                    code: "S0214",
                    stack: (new Error()).stack,
                    position: this.rhs.position,
                    token: "@"
                });
            }
            this.type = "binary";
            return this;
        });

        // index (position) variable bind
        infix("#", operators['#'], function (left) {
            this.lhs = left;
            this.rhs = expression(operators['#']);
            if(this.rhs.type !== 'variable') {
                return handleError({
                    code: "S0214",
                    stack: (new Error()).stack,
                    position: this.rhs.position,
                    token: "#"
                });
            }
            this.type = "binary";
            return this;
        });

        // if/then/else ternary operator ?:
        infix("?", operators['?'], function (left) {
            this.type = 'condition';
            this.condition = left;
            this.then = expression(0);
            if (node.id === ':') {
                // else condition
                advance(":");
                this.else = expression(0);
            }
            return this;
        });

        // object transformer
        prefix("|", function () {
            this.type = 'transform';
            this.pattern = expression(0);
            advance('|');
            this.update = expression(0);
            if (node.id === ',') {
                advance(',');
                this.delete = expression(0);
            }
            advance('|');
            return this;
        });
*/
