//! Implements a JSONata parser, which takes a stream of tokens and produces a hierarchy of AST
//! nodes.
//!
//! From the reference JSONata code:
//! > This parser implements the 'Top down operator precedence' algorithm developed by Vaughan R Pratt; <http://dl.acm.org/citation.cfm?id=512931>.
//! > and builds on the Javascript framework described by Douglas Crockford at <http://javascript.crockford.com/tdop/tdop.html>
//! > and in 'Beautiful Code', edited by Andy Oram and Greg Wilson, Copyright 2007 O'Reilly Media, Inc. 798-0-596-51004-6
//!
//! The formulation of a Top Down Operator Precendence parser (Pratt's Parser) is little more
//! complicated (and a lot more verbose) in a non-dynamic language.
//!
//! More resources:
//! - <http://effbot.org/zone/simple-top-down-parsing.htm>
//! - <http://journal.stuffwithstuff.com/2011/03/19/pratt-parsers-expression-parsing-made-easy/>
//!
//! Some definitions for some of the obscure abbreviations used in this parsing method:
//! - `rbp` & `lbp`: Left/right binding power, this is how the algorithm evaluates operator precedence
//! - `nud`: Null denotation, a nud symbol *does not* care about tokens to the left of it
//! - `led`: Left denotation, a led symbol *does* cares about tokens to the left of it
//!
//! Basic algorithm:
//! 1. Lexer generates tokens
//! 2. If the token appears at the beginning of an expression, call the nud method. If it appears
//!    infix, call the led method with the current left hand side as an argument.
//! 3. Expression parsing ends when the token's precedence is less than the expression's
//!    precedence.
//! 4. Productions are returned, which point to other productions forming the AST.
mod ast;
mod symbol;
mod tokenizer;

use crate::error::*;

use ast::*;
use symbol::Symbol;
use tokenizer::*;

pub use ast::Node;
pub use tokenizer::{Token, TokenKind};

/// An instance of a parser.
pub struct Parser<'a> {
    /// The tokenizer which will produce the tokens for parsing.
    tokenizer: Tokenizer<'a>,
    /// The last token obtained from the tokenizer.
    token: Token,
    /// TODO: remove
    depth: usize,
}

impl<'a> Parser<'a> {
    /// Returns the parsed AST for a given source string
    fn parse(source: &'a str) -> Box<Node> {
        let mut parser = Self::new(source);
        let ast = parser.expression(0);
        ast
        // parser.process_ast(&ast)
    }

    /// Create a new parser from a source string slice.
    fn new(source: &'a str) -> Self {
        let mut tokenizer = Tokenizer::new(source);
        Self {
            token: tokenizer.next(false),
            tokenizer,
            depth: 0,
        }
    }

    /// Obtain a reference to the current token.
    fn token(&self) -> &Token {
        &self.token
    }

    /// Advance the tokenizer.
    fn next(&mut self, infix: bool) {
        self.token = self.tokenizer.next(infix);
    }

    /// Ensure that the current token is an expected type, and then advance the tokenzier.
    fn expect(&mut self, expected: TokenKind, infix: bool) {
        if self.token.kind == TokenKind::End {
            error!(s0203, self.token.position, &expected)
        }

        if self.token.kind != expected {
            error!(s0202, self.token.position, &expected, &self.token)
        }

        self.next(infix);
    }

    /// Parse an expression, with a specified right binding power.
    fn expression(&mut self, rbp: u32) -> Box<Node> {
        self.depth += 1;
        // println!(
        //     "Enter {} ###################################################",
        //     self.depth
        // );
        let mut last = self.token.clone();
        self.next(true);
        // println!("{}: nud: {}", self.depth, last);
        let mut left = last.nud(self);

        while rbp < self.token.lbp() {
            // println!(
            //     "{}: rbp: {}, current.lbp: {}, current: {}",
            //     self.depth,
            //     rbp,
            //     self.token.lbp(),
            //     self.token
            // );
            last = self.token.clone();
            self.next(false);
            // println!("{}: led: {}", self.depth, last);
            left = last.led(self, left)
        }

        // use crate::ast::ToJson;
        // use json::stringify_pretty;
        // println!(
        //     "RESULT {}: {}",
        //     self.depth,
        //     stringify_pretty(left.to_json(), 4)
        // );
        // println!(
        //     "Exit {} ###################################################",
        //     self.depth
        // );
        self.depth -= 1;

        left
    }

    // fn process_ast(&self, ast: &Box<Node>) -> Box<Node> {
    //     use Node::*;
    //     match &**ast {
    //         PathSeparator(node) => {
    //             let mut result: Box<Node>;
    //             let lhs = self.process_ast(&node.lhs);

    //             if let Path(_) = lhs.as_ref() {
    //                 // Left hand side is a Path, so let's start with that
    //                 result = lhs;
    //             } else if let Parent(node) = lhs.as_ref() {
    //                 // Let hand side is a parent, so we will be looking for a parent
    //                 result = Box::new(Path(PathNode {
    //                     steps: vec![],
    //                     seeking_parent: vec![node.slot.clone()],
    //                     keep_singleton_array: false
    //                 }));
    //             } else {
    //                 // Otherwise we are creating a new path, where the left hand side will be the
    //                 // first step
    //                 result = Box::new(Path(PathNode {
    //                     steps: vec![lhs],
    //                     seeking_parent: vec![],
    //                     keep_singleton_array: false
    //                 }));
    //             }

    //             let mut rhs = self.process_ast(&node.rhs);
    //             /*
    //              TODO: This needs implementing
    //                         if (rest.type === 'function' &&
    //                             rest.procedure.type === 'path' &&
    //                             rest.procedure.steps.length === 1 &&
    //                             rest.procedure.steps[0].type === 'name' &&
    //                             result.steps[result.steps.length - 1].type === 'function') {
    //                             // next function in chain of functions - will override a thenable
    //                             result.steps[result.steps.length - 1].nextFunction = rest.procedure.steps[0].value;
    //                         }
    //             */
    //             if let Path(result) = result.as_mut() {
    //                 if let Path(node) = rhs.as_mut() {
    //                     // Right hand side is a path, so it must be merged with our result
    //                         result.steps.append(&mut node.steps);
    //                 } else {
    //                     /*
    //                     TODO: Figure out what predicate and stages are valid for
    //                     if(typeof rest.predicate !== 'undefined') {
    //                         rest.stages = rest.predicate;
    //                         delete rest.predicate;
    //                     }
    //                     */
    //                     result.steps.push(rhs);
    //                 }

    //                 for step in &mut result.steps {
    //                     let mut replace = false;
    //                     match step.as_ref() {
    //                         // Don't allow steps to be numbers, null, or boolean values
    //                         Number(node) => error!(s0213, node.get_position(), &node.get_value()),
    //                         Null(node)  => error!(s0213, node.get_position(), &node.get_value()),
    //                         Boolean(node) => error!(s0213, node.get_position(), &node.get_value()),

    //                         // Any steps within a path that are string literals should be changed to names
    //                         Str(node) => replace = true,

    //                         _ => ()
    //                     }
    //                     if replace {
    //                         *step = Box::new(Name(LiteralNode::new(step.get_position(), step.get_value())));
    //                     }
    //                 }
    //                 // Any step that signal keeping a singleton array, should be flagged on the path
    //                 if result.steps.iter().any(|step|
    //                     match step.as_ref() {
    //                         Name(node) => node.keep_array,
    //                         _ => false
    //                 }) {
    //                     result.keep_singleton_array = true;
    //                 }

    //                 // If first step is a path constructor, flag it for special handling
    //                 if let Some(Array(node)) = result.steps.first_mut().map(|b| b.as_mut()) {
    //                     node.consarray = true;
    //                 }
    //                 // If last step is a path constructor, flag it for special handling
    //                 if let Some(Array(node)) = result.steps.last_mut().map(|b| b.as_mut()) {
    //                      node.consarray = true;
    //                 }

    //                 self.resolve_ancestry(result);
    //             } else {
    //                 // We know that result is a path as we constructed it above. TODO: What's
    //                 // the idiomatic way in Rust to assert we know what we're doing here?
    //                 unreachable!("`node` should definitely be a path here")
    //             }

    //             result
    //         },
    //         Name(node) => {
    //             Box::new(Path(PathNode {
    //                 steps: vec![Box::new(*ast.clone())],
    //                 seeking_parent: vec![],
    //                 keep_singleton_array: node.keep_array
    //             }))
    //         },
    //         // Predicated step:
    //         //  Left hand side is a step or a predicated step
    //         //  Right hand side is the predicate expression
    //         // ArrayPredicate(node) =>  {
    //         //     let mut result = self.process_ast(node.lhs);
    //         //     let mut step = &result;
    //         //     let mut is_stages = false;

    // /*
    //                          // predicated step
    //                         // LHS is a step or a predicated step
    //                         // RHS is the predicate expr
    //                         result = processAST(expr.lhs);
    //                         var step = result;
    //                         var type = 'predicate';
    //                         if (result.type === 'path') {
    //                             step = result.steps[result.steps.length - 1];
    //                             type = 'stages';
    //                         }
    //                         if (typeof step.group !== 'undefined') {
    //                             throw {
    //                                 code: "S0209",
    //                                 stack: (new Error()).stack,
    //                                 position: expr.position
    //                             };
    //                         }
    //                         if (typeof step[type] === 'undefined') {
    //                             step[type] = [];
    //                         }
    //                         var predicate = processAST(expr.rhs);
    //                         if(typeof predicate.seekingParent !== 'undefined') {
    //                             predicate.seekingParent.forEach(slot => {
    //                                 if(slot.level === 1) {
    //                                     seekParent(step, slot);
    //                                 } else {
    //                                     slot.level--;
    //                                 }
    //                             });
    //                             pushAncestry(step, predicate);
    //                         }
    //                         step[type].push({type: 'filter', expr: predicate, position: expr.position});
    //                         break;
    // // */
    //         // },
    //         _ => Box::new(*ast.clone())

    //         // },
    //         // // TODO: Group-by
    //         // OrderBy(node) => {

    //         // },
    //         // Assignment(node) => {

    //         // },
    //         // FocusVariableBind(node) => {

    //         // },
    //         // IndexVariableBind(node) => {

    //         // },
    //         // Chain(node) => {

    //         // },
    //         // Add(node) => {

    //         // },
    //         // Subtract(node) => {

    //         // },
    //         // Multiply(node) => {

    //         // },
    //         // Divide(node) => {

    //         // },
    //         // Modulus(node) => {

    //         // },
    //         // Equal(node) => {

    //         // },
    //         // LessThan(node) => {

    //         // },
    //         // GreaterThan(node) => {

    //         // },
    //         // NotEqual(node) => {

    //         // },
    //         // LessThanEqual(node) => {

    //         // },
    //         // GreaterThanEqual(node) => {

    //         // },
    //         // Concat(node) => {

    //         // },
    //         // And(node) => {

    //         // },
    //         // Or(node) => {

    //         // },
    //         // In(node) => {

    //         // },
    //         // Range(node) => {

    //         // },
    //         // // Unary nodes
    //         // Array(node) => {

    //         // },
    //         // ObjectPrefix(node) => {

    //         // },
    //         // UnaryMinus(node) => {

    //         // },
    //         // // Functions
    //         // FunctionCall(node) | PartialFunctionCall(node) => {

    //         // },
    //         // LambdaFunction(node) => {

    //         // },
    //         // // Objects
    //         // Transform(node) => {

    //         // },
    //         // // Paths
    //         // Name(node) => {

    //         // },
    //         // Parent(node) => {

    //         // },
    //         // Wildcard(node) => {

    //         // },
    //         // DescendantWildcard(node) => {

    //         // },
    //         // // Literals
    //         // Null(node) => {

    //         // },
    //         // Boolean(node) => {

    //         // },
    //         // String(node) => {

    //         // },
    //         // Number(node) => {

    //         // },
    //         // Variable => {

    //         // },
    //         // // Other operators
    //         // Ternary(node) => {

    //         // },
    //         // Block(node) => {

    //         // },

    //     }
    // }

    fn resolve_ancestry(&self, path: &mut PathNode) {
        // TODO
    }
}

pub fn parse(source: &str) -> Box<Node> {
    Parser::parse(source)
}

#[cfg(test)]
mod tests {
    //! Parsing tests, mostly just to ensure that the parser doesn't fail on valid JSONata. Most
    //! of these examples are taken from the JSONata docs. These are not meant to be tests of the
    //! produced AST, which is proved correct by the integration tests.
    use super::*;
    use test_case::test_case;

    // #[test_case("Address1.City")]
    // #[test_case("Other.`Over 18 ?`")]
    // #[test_case("Phone1[0]")]
    // #[test_case("Phone2[-1]")]
    // #[test_case("Phone3[0].Number")]
    // #[test_case("Phone4[[0..1]]")]
    // #[test_case("$[0]")]
    // #[test_case("$[0].ref")]
    // #[test_case("$[0].ref[0]")]
    // #[test_case("$.ref")]
    // #[test_case("Phone5[type='mobile']")]
    // #[test_case("Phone6[type='mobile'].number")]
    // #[test_case("Address2.*")]
    // #[test_case("*.Postcode1")]
    // #[test_case("**.Postcode2")]
    // #[test_case("FirstName & ' ' & Surname")]
    // #[test_case("Address3.(Street & ', ' & City)")]
    // #[test_case("5&0&true")]
    // #[test_case("Numbers1[0] + Numbers[1]")]
    // #[test_case("Numbers2[0] - Numbers[1]")]
    // #[test_case("Numbers3[0] * Numbers[1]")]
    // #[test_case("Numbers4[0] / Numbers[1]")]
    // #[test_case("Numbers5[0] % Numbers[1]")]
    // #[test_case("Numbers6[0] = Numbers[5]")]
    // #[test_case("Numbers7[0] != Numbers[5]")]
    // #[test_case("Numbers8[0] < Numbers[5]")]
    // #[test_case("Numbers9[0] <= Numbers[5]")]
    // #[test_case("Numbers10[0] > Numbers[5]")]
    // #[test_case("Numbers11[0] >= Numbers[5]")]
    // #[test_case("\"01962 001234\" in Phone.number")]
    // #[test_case("(Numbers12[2] != 0) and (Numbers[5] != Numbers[1])")]
    // #[test_case("(Numbers13[2] != 0) or (Numbers[5] = Numbers[1])")]
    // #[test_case("Email1.[address]")]
    #[test_case("[Address4, Other.`Alternative.Address`].City")]
    // #[test_case("Phone7.{type: number}")]
    // #[test_case("Phone8{type: number}")]
    // #[test_case("Phone9{type: number[]}")]
    // #[test_case("(5 + 3) * 4")]
    // #[test_case("Product.(Price * Quantity)")]
    // #[test_case("(expr1; expr2; expr3)")]
    // #[test_case("Account1.Order.Product{`Product Name`: Price}")]
    // #[test_case(
    //     r#"
    //     Account2.Order.Product {
    //         `Product Name`: {"Price": Price, "Qty": Quantity}
    //     }
    // "#
    // )]
    // #[test_case(
    //     r#"
    //     Account3.Order.Product {
    //       `Product Name`: $.{"Price": Price, "Qty": Quantity}
    //     }
    // "#
    // )]
    // #[test_case("Account4.Order.Product{`Product Name`: $.(Price*Quantity)}")]
    // #[test_case("Account5.Order.Product{`Product Name`: $sum($.(Price*Quantity))}")]
    // #[test_case("$sum1(Account.Order.Product.Price)")]
    // #[test_case("$sum2(Account.Order.Product.(Price*Quantity))")]
    // #[test_case(
    //     r#"
    //     Invoice.(
    //       $p := Product.Price;
    //       $q := Product.Quantity;
    //       $p * $q
    //     )
    // "#
    // )]
    // #[test_case(
    //     r#"
    //     (
    //       $volume := function($l, $w, $h){ $l * $w * $h };
    //       $volume(10, 10, 5);
    //     )
    // "#
    // )]
    // #[test_case(
    //     r#"
    //     (
    //       $factorial:= function($x){ $x <= 1 ? 1 : $x * $factorial($x-1) };
    //       $factorial(4)
    //     )
    // "#
    // )]
    // #[test_case(
    //     r#"
    //     (
    //       $factorial := function($x){(
    //         $iter := function($x, $acc) {
    //           $x <= 1 ? $acc : $iter($x - 1, $x * $acc)
    //         };
    //         $iter($x, 1)
    //       )};
    //       $factorial(170)
    //     )
    // "#
    // )]
    // #[test_case(
    //     r#"
    //     (
    //       $twice := function($f) { function($x){ $f($f($x)) } };
    //       $add3 := function($y){ $y + 3 };
    //       $add6 := $twice($add3);
    //       $add6(7)
    //     )
    // "#
    // )]
    // #[test_case(
    //     r#"
    //     Account.(
    //       $AccName := function() { $.'Account Name' };

    //       Order[OrderID = 'order104'].Product.{
    //         'Account': $AccName(),
    //         'SKU-' & $string(ProductID): $.'Product Name'
    //       }
    //     )
    // "#
    // )]
    // #[test_case(
    //     r#"
    //     (
    //       $firstN := $substring(?, 0, ?);
    //       $first5 := $firstN(?, 5);
    //       $first5("Hello, World")
    //     )
    // "#
    // )]
    // #[test_case(
    //     "Customer.Email ~> $substringAfter(\"@\") ~> $substringBefore(\".\") ~> $uppercase()"
    // )]
    // #[test_case(
    //     r#"
    //     Account.Order.Product.{
    //       'Product': `Product Name`,
    //       'Order': %.OrderID,
    //       'Account': %.%.`Account Name`
    //     }
    // "#
    // )]
    // #[test_case(
    //     r#"
    //     library.books#$i['Kernighan' in authors].{
    //       'title': title,
    //       'index': $i
    //     }
    // "#
    // )]
    // #[test_case(
    //     r#"
    //     library.loans@$l.books@$b[$l.isbn=$b.isbn].{
    //       'title': $b.title,
    //       'customer': $l.customer
    //     }
    // "#
    // )]
    // #[test_case(
    //     r#"
    //     (library.loans)@$l.(catalog.books)@$b[$l.isbn=$b.isbn].{
    //       'title': $b.title,
    //       'customer': $l.customer
    //     }
    // "#
    // )]
    // #[test_case("payload ~> |Account.Order.Product|{'Price': Price * 1.2}|")]
    // #[test_case("$ ~> |Account.Order.Product|{'Total': Price * Quantity}, ['Price', 'Quantity']|")]
    // #[test_case(
    //     r#"
    //     /* Long-winded expressions might need some explanation */
    //     (
    //       $pi := 3.1415926535897932384626;
    //       /* JSONata is not known for its graphics support! */
    //       $plot := function($x) {(
    //         $floor := $string ~> $substringBefore(?, '.') ~> $number;
    //         $index := $floor(($x + 1) * 20 + 0.5);
    //         $join([0..$index].('.')) & 'O' & $join([$index..40].('.'))
    //       )};

    //       /* Factorial is the product of the integers 1..n */
    //       $product := function($a, $b) { $a * $b };
    //       $factorial := function($n) { $n = 0 ? 1 : $reduce([1..$n], $product) };

    //       $sin := function($x){ /* define sine in terms of cosine */
    //         $cos($x - $pi/2)
    //       };
    //       $cos := function($x){ /* Derive cosine by expanding Maclaurin series */
    //         $x > $pi ? $cos($x - 2 * $pi) : $x < -$pi ? $cos($x + 2 * $pi) :
    //           $sum([0..12].($power(-1, $) * $power($x, 2*$) / $factorial(2*$)))
    //       };

    //       [0..24].$sin($*$pi/12).$plot($)
    //     )
    // "#
    // )]
    fn parser_tests(source: &str) {
        let ast = Parser::parse(source);
        use json::stringify_pretty;
        println!("{}", stringify_pretty(ast.to_json(), 4));
    }
}
