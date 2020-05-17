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
use crate::ast::Node;
use crate::error::*;
use crate::symbol::Symbol;
use crate::tokenizer::{Token, TokenKind, Tokenizer};

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
    /// Create a new parser from a source string slice.
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
            error!(s0203, self.token.position, &expected)
        }

        if self.token.kind != expected {
            error!(s0202, self.token.position, &expected, &self.token)
        }

        self.next(infix);
    }

    pub fn expression(&mut self, rbp: u32) -> Box<Node> {
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
}

pub fn parse(source: &str) -> Box<Node> {
    let mut parser = Parser::new(source);
    parser.expression(0)
}

#[cfg(test)]
mod tests {
    //! Parsing tests, mostly just to ensure that the parser doesn't fail on valid JSONata. Most
    //! of these examples are taken from the JSONata docs. These are not meant to be tests of the
    //! produced AST, which is proved correct by the integration tests.
    use super::*;
    use test_case::test_case;

    #[test_case("Address1.City")]
    #[test_case("Other.`Over 18 ?`")]
    #[test_case("Phone1[0]")]
    #[test_case("Phone2[-1]")]
    #[test_case("Phone3[0].Number")]
    #[test_case("Phone4[[0..1]]")]
    #[test_case("$[0]")]
    #[test_case("$[0].ref")]
    #[test_case("$[0].ref[0]")]
    #[test_case("$.ref")]
    #[test_case("Phone5[type='mobile']")]
    #[test_case("Phone6[type='mobile'].number")]
    #[test_case("Address2.*")]
    #[test_case("*.Postcode1")]
    #[test_case("**.Postcode2")]
    #[test_case("FirstName & ' ' & Surname")]
    #[test_case("Address3.(Street & ', ' & City)")]
    #[test_case("5&0&true")]
    #[test_case("Numbers1[0] + Numbers[1]")]
    #[test_case("Numbers2[0] - Numbers[1]")]
    #[test_case("Numbers3[0] * Numbers[1]")]
    #[test_case("Numbers4[0] / Numbers[1]")]
    #[test_case("Numbers5[0] % Numbers[1]")]
    #[test_case("Numbers6[0] = Numbers[5]")]
    #[test_case("Numbers7[0] != Numbers[5]")]
    #[test_case("Numbers8[0] < Numbers[5]")]
    #[test_case("Numbers9[0] <= Numbers[5]")]
    #[test_case("Numbers10[0] > Numbers[5]")]
    #[test_case("Numbers11[0] >= Numbers[5]")]
    #[test_case("\"01962 001234\" in Phone.number")]
    #[test_case("(Numbers12[2] != 0) and (Numbers[5] != Numbers[1])")]
    #[test_case("(Numbers13[2] != 0) or (Numbers[5] = Numbers[1])")]
    #[test_case("Email1.[address]")]
    #[test_case("[Address4, Other.`Alternative.Address`].City")]
    #[test_case("Phone7.{type: number}")]
    #[test_case("Phone8{type: number}")]
    #[test_case("Phone9{type: number[]}")]
    #[test_case("(5 + 3) * 4")]
    #[test_case("Product.(Price * Quantity)")]
    #[test_case("(expr1; expr2; expr3)")]
    #[test_case("Account1.Order.Product{`Product Name`: Price}")]
    #[test_case(
        r#"
        Account2.Order.Product {
            `Product Name`: {"Price": Price, "Qty": Quantity}
        }
    "#
    )]
    #[test_case(
        r#"
        Account3.Order.Product {
          `Product Name`: $.{"Price": Price, "Qty": Quantity}
        }
    "#
    )]
    #[test_case("Account4.Order.Product{`Product Name`: $.(Price*Quantity)}")]
    #[test_case("Account5.Order.Product{`Product Name`: $sum($.(Price*Quantity))}")]
    #[test_case("$sum1(Account.Order.Product.Price)")]
    #[test_case("$sum2(Account.Order.Product.(Price*Quantity))")]
    #[test_case(
        r#"
        Invoice.(
          $p := Product.Price;
          $q := Product.Quantity;
          $p * $q
        )
    "#
    )]
    #[test_case(
        r#"
        (
          $volume := function($l, $w, $h){ $l * $w * $h };
          $volume(10, 10, 5);
        )
    "#
    )]
    #[test_case(
        r#"
        (
          $factorial:= function($x){ $x <= 1 ? 1 : $x * $factorial($x-1) };
          $factorial(4)
        )
    "#
    )]
    #[test_case(
        r#"
        (
          $factorial := function($x){(
            $iter := function($x, $acc) {
              $x <= 1 ? $acc : $iter($x - 1, $x * $acc)
            };
            $iter($x, 1)
          )};
          $factorial(170)
        )
    "#
    )]
    #[test_case(
        r#"
        (
          $twice := function($f) { function($x){ $f($f($x)) } };
          $add3 := function($y){ $y + 3 };
          $add6 := $twice($add3);
          $add6(7)
        )
    "#
    )]
    #[test_case(
        r#"
        Account.(
          $AccName := function() { $.'Account Name' };

          Order[OrderID = 'order104'].Product.{
            'Account': $AccName(),
            'SKU-' & $string(ProductID): $.'Product Name'
          }
        )
    "#
    )]
    #[test_case(
        r#"
        (
          $firstN := $substring(?, 0, ?);
          $first5 := $firstN(?, 5);
          $first5("Hello, World")
        )
    "#
    )]
    #[test_case(
        "Customer.Email ~> $substringAfter(\"@\") ~> $substringBefore(\".\") ~> $uppercase()"
    )]
    #[test_case(
        r#"
        Account.Order.Product.{
          'Product': `Product Name`,
          'Order': %.OrderID,
          'Account': %.%.`Account Name`
        }
    "#
    )]
    #[test_case(
        r#"
        library.books#$i['Kernighan' in authors].{
          'title': title,
          'index': $i
        }
    "#
    )]
    #[test_case(
        r#"
        library.loans@$l.books@$b[$l.isbn=$b.isbn].{
          'title': $b.title,
          'customer': $l.customer
        }
    "#
    )]
    #[test_case(
        r#"
        (library.loans)@$l.(catalog.books)@$b[$l.isbn=$b.isbn].{
          'title': $b.title,
          'customer': $l.customer
        }
    "#
    )]
    #[test_case("payload ~> |Account.Order.Product|{'Price': Price * 1.2}|")]
    #[test_case("$ ~> |Account.Order.Product|{'Total': Price * Quantity}, ['Price', 'Quantity']|")]
    #[test_case(
        r#"
        /* Long-winded expressions might need some explanation */
        (
          $pi := 3.1415926535897932384626;
          /* JSONata is not known for its graphics support! */
          $plot := function($x) {(
            $floor := $string ~> $substringBefore(?, '.') ~> $number;
            $index := $floor(($x + 1) * 20 + 0.5);
            $join([0..$index].('.')) & 'O' & $join([$index..40].('.'))
          )};

          /* Factorial is the product of the integers 1..n */
          $product := function($a, $b) { $a * $b };
          $factorial := function($n) { $n = 0 ? 1 : $reduce([1..$n], $product) };

          $sin := function($x){ /* define sine in terms of cosine */
            $cos($x - $pi/2)
          };
          $cos := function($x){ /* Derive cosine by expanding Maclaurin series */
            $x > $pi ? $cos($x - 2 * $pi) : $x < -$pi ? $cos($x + 2 * $pi) :
              $sum([0..12].($power(-1, $) * $power($x, 2*$) / $factorial(2*$)))
          };

          [0..24].$sin($*$pi/12).$plot($)
        )
    "#
    )]
    fn parser_tests(source: &str) {
        parse(source);
    }
}
