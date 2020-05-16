/// From the reference JavaScript JSONAta code:
///   This parser implements the 'Top down operator precedence' algorithm developed by Vaughan R Pratt; http://dl.acm.org/citation.cfm?id=512931.
///   and builds on the Javascript framework described by Douglas Crockford at http://javascript.crockford.com/tdop/tdop.html
///   and in 'Beautiful Code', edited by Andy Oram and Greg Wilson, Copyright 2007 O'Reilly Media, Inc. 798-0-596-51004-6
///
/// The formulation of a Top Down Operator Precendence parser (Pratt's Parser) is little more
/// complicated (and a lot more verbose) in a non-dynamic language.
///
/// More resources:
///  - http://effbot.org/zone/simple-top-down-parsing.htm
///  - http://journal.stuffwithstuff.com/2011/03/19/pratt-parsers-expression-parsing-made-easy/
///
/// Some definitions for some of the obscure abbreviations used in this parsing method:
///  rbp & lbp: Left/right binding power, this is how the algorithm evaluates operator precedence
///  nud: Null denotation, a nud symbol DOES NOT care about tokens to the left of it
///  led: Left denotation, a led symbol DOES cares about tokens to the left of it
///
/// Basic algorithm:
///  1. Lexer generates tokens
///  2. If the token appears at the beginning of an expression, call the nud method. If it appears
///     infix, call the led method with the current left hand side as an argument.
///  3. Expression parsing ends when the token's precendence is less than the expression's
///     precendence.
///  4. Productions are returned, which point to other productions forming the AST.
use crate::ast::{Node, ToJson};
use crate::symbol::Symbol;
use crate::tokenizer::{Token, TokenKind, Tokenizer};

pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
    token: Token,
    depth: usize,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut tokenizer = Tokenizer::new(source);
        Self {
            token: tokenizer.next(false),
            tokenizer,
            depth: 0,
        }
    }

    pub fn token(&self) -> &Token {
        &self.token
    }

    pub fn next(&mut self, infix: bool) {
        self.token = self.tokenizer.next(infix);
    }

    pub fn expect(&mut self, expected: TokenKind, infix: bool) {
        if self.token.kind == TokenKind::End {
            error!(S0203, self.token.position, expected)
        }

        if self.token.kind != expected {
            error!(S0202, self.token.position, expected, self.token)
        }

        self.next(infix);
    }

    pub fn expression(&mut self, rbp: u32) -> Box<Node> {
        self.depth += 1;
        let mut last = self.token.clone();
        //println!("{}: last: {:#?}", self.depth, last);
        self.next(true);
        //println!("{}: current: {:#?}", self.depth, self.token);
        //println!("{}: nud: {:#?}", self.depth, last);
        let mut left = last.nud(self);

        while rbp < self.token.lbp() {
            //println!(
            //    "{}: rbp: {}, current.lbp: {}",
            //    self.depth,
            //    rbp,
            //    self.token.lbp()
            //);
            last = self.token.clone();
            //println!("{}: last: {:#?}", self.depth, last);
            self.next(false);
            //println!("{}: current: {:#?}", self.depth, self.token);
            //println!("{}: led: {:#?}", self.depth, last);
            left = last.led(self, left)
        }

        //use json::stringify_pretty;
        //println!("{}: {}", self.depth, stringify_pretty(left.to_json(), 4));

        self.depth -= 1;

        left
    }
}

pub fn parse(source: &str) -> Box<Node> {
    let mut parser = Parser::new(source);
    parser.expression(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use json::stringify_pretty;

    #[test]
    fn basic() {
        let ast = parse("1 + 2 * 3");
        let json = ast.to_json();
        let expected = r#"{
    "type": "binary",
    "value": "+",
    "position": 3,
    "lhs": {
        "type": "number",
        "position": 1,
        "value": 1
    },
    "rhs": {
        "type": "binary",
        "value": "*",
        "position": 7,
        "lhs": {
            "type": "number",
            "position": 5,
            "value": 2
        },
        "rhs": {
            "type": "number",
            "position": 9,
            "value": 3
        }
    }
}"#;
        assert_eq!(expected, stringify_pretty(json, 4));
    }

    #[test]
    fn function() {
        let ast = parse(
            r#"
            $plot := function($x) {(
                $floor := $string ~> $substringBefore(?, '.') ~> $number;
                $index := $floor(($x + 1) * 20 + 0.5);
                /*$join([0..$index].('.')) & 'O' & $join([$index..40].('.'))*/
            )}
            "#,
        );
        let json = ast.to_json();
        println!("{}", stringify_pretty(json, 4));
    }
}
