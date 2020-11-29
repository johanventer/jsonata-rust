use crate::error::*;
use crate::Result;

use super::ast::*;
use super::process::process_ast;
use super::symbol::Symbol;
use super::tokenizer::*;

pub(crate) struct Parser {
    tokenizer: Tokenizer,
    token: Token,
}

impl Parser {
    fn new(source: &str) -> Result<Self> {
        let mut tokenizer = Tokenizer::new(source);
        Ok(Self {
            token: tokenizer.next(false)?,
            tokenizer,
        })
    }

    pub(crate) fn token(&self) -> &Token {
        &self.token
    }

    pub(crate) fn next(&mut self, infix: bool) -> Result<()> {
        self.token = self.tokenizer.next(infix)?;
        Ok(())
    }

    pub(crate) fn expect(&mut self, expected: TokenKind, infix: bool) -> Result<()> {
        if self.token.kind == TokenKind::End {
            return Err(Box::new(S0203 {
                position: self.token.position,
                expected: expected.to_string(),
            }));
        }

        if self.token.kind != expected {
            return Err(Box::new(S0202 {
                position: self.token.position,
                expected: expected.to_string(),
                actual: self.token.kind.to_string(),
            }));
        }

        self.next(infix)?;

        Ok(())
    }

    pub(crate) fn expression(&mut self, bp: u32) -> Result<Box<Node>> {
        let mut last = self.token.clone();
        self.next(true)?;
        let mut left = last.nud(self)?;

        while bp < self.token.lbp() {
            last = self.token.clone();
            self.next(false)?;
            left = last.led(self, left)?;
        }

        Ok(left)
    }
}

pub(crate) fn parse(source: &str) -> Result<Box<Node>> {
    let mut parser = Parser::new(source)?;
    let ast = parser.expression(0)?;
    if !matches!(parser.token().kind, TokenKind::End) {
        return Err(box S0201 {
            position: parser.token().position,
            value: parser.token().to_string(),
        });
    }
    Ok(process_ast(ast)?)
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
    #[test_case("Account2.Order.Product^(Price)")]
    #[test_case("Account3.Order.Product^(>Price)")]
    #[test_case("Account4.Order.Product^(>Price, <Quantity)")]
    #[test_case("Account5.Order.Product^(Price * Quantity)")]
    #[test_case("student[type='fulltime']^(DoB).name")]
    #[test_case(
        r#"
        Account6.Order.Product.{
          'Product': `Product Name`,
          'Order': %.OrderID,
          'Account': %.%.`Account Name`
        }
    "#
    )]
    #[test_case(
        r#"
        Account7.Order.Product {
            `Product Name`: {"Price": Price, "Qty": Quantity}
        }
    "#
    )]
    #[test_case(
        r#"
        Account8.Order.Product {
          `Product Name`: $.{"Price": Price, "Qty": Quantity}
        }
    "#
    )]
    #[test_case(
        r#"
        library1.books#$i['Kernighan' in authors].{
          'title': title,
          'index': $i
        }
    "#
    )]
    #[test_case(
        r#"
        library2.loans@$l.books@$b[$l.isbn=$b.isbn].{
          'title': $b.title,
          'customer': $l.customer
        }
    "#
    )]
    #[test_case(
        r#"
        (library3.loans)@$l.(catalog.books)@$b[$l.isbn=$b.isbn].{
          'title': $b.title,
          'customer': $l.customer
        }
    "#
    )]
    #[test_case("Account9.Order.Product{`Product Name`: $.(Price*Quantity)}")]
    #[test_case("Account10.Order.Product{`Product Name`: $sum($.(Price*Quantity))}")]
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
    fn parser_tests(source: &str) -> Result<Box<Node>> {
        parse(source)
    }
}
