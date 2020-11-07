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

use crate::ast::*;
use crate::error::*;
use crate::symbol::*;
use crate::tokenizer::*;
use crate::JsonAtaResult;

/// An instance of a parser.
pub struct Parser {
    /// The tokenizer which will produce the tokens for parsing.
    tokenizer: Tokenizer,

    /// The last token obtained from the tokenizer.
    token: Token,
    // ancestor_label: u32,
    // ancestor_index: u32,
}

impl Parser {
    /// Create a new parser from a source string slice.
    fn new(source: &str) -> JsonAtaResult<Self> {
        let mut tokenizer = Tokenizer::new(source);
        Ok(Self {
            token: tokenizer.next(false)?,
            tokenizer,
            // ancestor_index: 0,
            // ancestor_label: 0,
        })
    }

    /// Obtain a reference to the current token.
    pub fn token(&self) -> &Token {
        &self.token
    }

    /// Advance the tokenizer.
    pub fn next(&mut self, infix: bool) -> JsonAtaResult<()> {
        self.token = self.tokenizer.next(infix)?;
        Ok(())
    }

    /// Ensure that the current token is an expected type, and then advance the tokenzier.
    pub fn expect(&mut self, expected: TokenKind, infix: bool) -> JsonAtaResult<()> {
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

    /// Parse an expression, with a specified minimum binding power.
    pub fn expression(&mut self, bp: u32) -> JsonAtaResult<Node> {
        let mut last = self.token.clone();
        self.next(true)?;
        let mut left = last.nud(self)?;

        while bp < self.token.lbp() {
            last = self.token.clone();
            self.next(false)?;
            left = last.led(self, left)?;
        }

        //println!("{:#?}", left);

        Ok(left)
    }
}

/// Returns the parsed AST for a given source string.
pub fn parse(source: &str) -> JsonAtaResult<Node> {
    let mut parser = Parser::new(source)?;
    let ast = parser.expression(0)?;
    Ok(process_ast(&ast)?)
}

type N = NodeKind;

fn process_ast(node: &Node) -> JsonAtaResult<Node> {
    let kind = node.kind.clone();

    match kind {
        N::Binary(ref op) => process_binary_node(node, op),
        N::Name(..) => Ok(Node::new_with_child(N::Path, node.position, node.clone())),
        _ => Ok(node.clone()),
    }
}

fn process_binary_node(node: &Node, op: &BinaryOp) -> JsonAtaResult<Node> {
    type N = NodeKind;

    match op {
        BinaryOp::Path => {
            let lhs = process_ast(&node.children[0])?;
            let mut rhs = process_ast(&node.children[1])?;

            let mut result = {
                // If lhs is a Path, start with that, otherwise create a new one
                if let N::Path = lhs.kind {
                    lhs
                } else {
                    Node::new_with_child(N::Path, lhs.position, lhs)
                }
            };

            // TODO: If the lhs is a Parent (parser.js:997)

            // TODO: If the rhs is a Function (parser.js:1001)

            // If rhs is a Path, merge the steps in
            if let N::Path = rhs.kind {
                result.children.append(&mut rhs.children);
            } else {
                // TODO: Predicate stuff (parser.js:1012)
                result.children.push(rhs);
            }

            // Path steps that are string literals should be changed to Name nodes, and should not
            // be number's or other literal values
            for step in &mut result.children {
                match step.kind {
                    N::Num(..) | N::Bool(..) | N::Null => {
                        return Err(box S0213 {
                            position: step.position,
                            value: step.kind.to_string(),
                        })
                    }
                    N::Str(ref v) => {
                        step.kind = N::Name(v.clone());
                    }
                    _ => (),
                }

                // TODO: Handle singleton array (parser.js:1034)
            }

            // TODO: Filter step types, handle first step is a path constructor, handle last step is an array constructor (parser.js:1040)

            Ok(result)
        }

        _ => {
            let lhs = process_ast(&node.children[0])?;
            let rhs = process_ast(&node.children[1])?;
            Ok(Node::new_with_children(
                node.kind.clone(),
                node.position,
                vec![lhs, rhs],
            ))
        }
    }
}

// fn process_ast(&mut self, node: Node) -> Node {
//     use NodeKind::*;
//     let mut node = node;

//     match &node.kind {
//         Parent(..) => {
//             node.kind = Parent(Some(Slot {
//                 label: format!("!{}", self.ancestor_label),
//                 level: 1,
//                 index: self.ancestor_index,
//             }));

//             self.ancestor_index += 1;
//             self.ancestor_label += 1;
//             node
//         }
//         // Wrap Name nodes in a Path node
//         Name(name) => {
//             let keep_array = node.keep_array;
//             let mut path = Node::new_with_child(Path, node.position, node);
//             path.keep_array = keep_array;
//             path
//             // TODO: seeking_parent
//         }
//         Unary(ref op) => match op {
//             // Array constructor - process each child
//             UnaryOp::Array => {
//                 // TODO: let consarray = node.consarray;
//                 node.children = node
//                     .children
//                     .into_iter()
//                     .map(|child| self.process_ast(child))
//                     .collect();
//                 node
//             }
//             UnaryOp::Minus => {
//                 let expression = &mut node.children[0];
//                 // Pre-process unary minus on numbers
//                 if let Num(ref mut num) = expression.kind {
//                     *num = -*num;
//                 } else {
//                     // pushAncestry
//                 }
//                 node
//             }
//         },
//         Transform | Object => {
//             node.children = node
//                 .children
//                 .into_iter()
//                 .map(|child| self.process_ast(child))
//                 .collect();
//             node
//         }
//         _ => node,
//     }

//    macro_rules! binary {
//        ($t:tt, $n:ident) => {{
//            let lhs = self.process_ast($n.lhs);
//            let rhs = self.process_ast($n.rhs);
//            // pushAncestory for both lhs and rhs
//            Box::new($t(BinaryNode {
//                position: $n.position,
//                lhs,
//                rhs,
//            }))
//        }};
//    }

//    /* Things to cover here:
//        [x] PathSeparator
//        [x] Name -> Gets wrapped in a path
//        [x] Chain -> Returns an Apply node
//        [x] ParentOp
//        [x] FunctionCall
//        [x] PartialFunctionCall
//        [x] LambdaFunction
//        [x] UnaryMinus
//        [x] Block
//        [x] Array
//        [x] Assignment -> Returns a Bind node
//        [x] OrderBy
//        [x] Ternary
//        [x] Transform
//        [x] Object
//        [ ] GroupBy
//        [ ] ArrayPredicate
//        [ ] FocusVariableBind
//        [ ] IndexVariableBind
//    */
//    match *ast {
//        PathSeparator(node) => {
//            let mut result: Box<Node>;
//            let lhs = self.process_ast(node.lhs);

//            if let Path(_) = *lhs {
//                // Left hand side is a Path, so let's start with that
//                result = lhs;
//            } else if let Parent(node) = *lhs {
//                // Let hand side is a parent, so we will be looking for a parent
//                result = Box::new(Path(PathNode {
//                    steps: vec![],
//                    seeking_parent: vec![node.slot],
//                    keep_singleton_array: false,
//                }));
//            } else {
//                // Otherwise we are creating a new path, where the left hand side will be the
//                // first step
//                result = Box::new(Path(PathNode {
//                    steps: vec![lhs],
//                    seeking_parent: vec![],
//                    keep_singleton_array: false,
//                }));
//            }

//            let mut rhs = self.process_ast(node.rhs);
//            /*
//             TODO: This needs implementing
//                        if (rest.type === 'function' &&
//                            rest.procedure.type === 'path' &&
//                            rest.procedure.steps.length === 1 &&
//                            rest.procedure.steps[0].type === 'name' &&
//                            result.steps[result.steps.length - 1].type === 'function') {
//                            // next function in chain of functions - will override a thenable
//                            result.steps[result.steps.length - 1].nextFunction = rest.procedure.steps[0].value;
//                        }
//            */
//            if let Path(result) = result.as_mut() {
//                if let Path(node) = rhs.as_mut() {
//                    // Right hand side is a path, so it must be merged with our result
//                    result.steps.append(&mut node.steps);
//                } else {
//                    /*
//                    TODO: Figure out what predicate and stages are valid for
//                    if(typeof rest.predicate !== 'undefined') {
//                        rest.stages = rest.predicate;
//                        delete rest.predicate;
//                    }
//                    */
//                    result.steps.push(rhs);
//                }

//                for step in &mut result.steps {
//                    let mut replace = false;
//                    match step.as_ref() {
//                        // Don't allow steps to be numbers, null, or boolean values
//                        Number(node) => error!(s0213, node.get_position(), &node.get_value()),
//                        Null(node) => error!(s0213, node.get_position(), &node.get_value()),
//                        Boolean(node) => error!(s0213, node.get_position(), &node.get_value()),

//                        // Any steps within a path that are string literals should be changed to names
//                        Str(node) => replace = true,

//                        _ => (),
//                    }
//                    if replace {
//                        *step = Box::new(Name(LiteralNode::new(
//                            step.get_position(),
//                            step.get_value(),
//                        )));
//                    }
//                }
//                // Any step that signal keeping a singleton array, should be flagged on the path
//                if result.steps.iter().any(|step| match step.as_ref() {
//                    Name(node) => node.keep_array,
//                    _ => false,
//                }) {
//                    result.keep_singleton_array = true;
//                }

//                // If first step is a path constructor, flag it for special handling
//                if let Some(Array(node)) = result.steps.first_mut().map(|b| b.as_mut()) {
//                    node.consarray = true;
//                }
//                // If last step is a path constructor, flag it for special handling
//                if let Some(Array(node)) = result.steps.last_mut().map(|b| b.as_mut()) {
//                    node.consarray = true;
//                }

//                // self.resolve_ancestry(result);
//            }

//            result
//        }
//        // Block (array of expressions) - process each node
//        Block(node) => {
//            let mut expressions = Vec::new();
//            let mut consarray = false;
//            let position = node.get_position();
//            for expr in node.expressions {
//                let expr = self.process_ast(expr);
//                match *expr {
//                    Array(ref node) => {
//                        if node.consarray {
//                            consarray = true;
//                        }
//                    }
//                    Path(ref node) => {
//                        if !node.steps.is_empty() {
//                            if let Array(ref node) = *node.steps[0] {
//                                if node.consarray {
//                                    consarray = true
//                                }
//                            }
//                        }
//                    }
//                    _ => (),
//                }
//                // pushAncestry(result, value)
//                expressions.push(expr);
//            }
//            Box::new(Block(ExpressionsNode {
//                position,
//                expressions,
//                consarray,
//            }))
//        }
//        // Ternary conditional
//        Ternary(node) => {
//            let position = node.get_position();
//            let condition = self.process_ast(node.condition);
//            // pushAncestry(result, result.condition)
//            let then = self.process_ast(node.then);
//            // pushAncestry(result, result.then)
//            let els = match node.els {
//                Some(node) => {
//                    let node = self.process_ast(node);
//                    // pushAncestry(result, node)
//                    Some(node)
//                }
//                None => None,
//            };
//            Box::new(Ternary(TernaryNode {
//                position,
//                condition,
//                then,
//                els,
//            }))
//        }
//        // Assignment
//        Assignment(node) => {
//            let lhs = self.process_ast(node.lhs);
//            let rhs = self.process_ast(node.rhs);
//            // pushAncestry(result, result.rhs)
//            Box::new(Bind(BindNode {
//                position: node.position,
//                lhs,
//                rhs,
//            }))
//        }
//        // Function application
//        Chain(node) => {
//            let lhs = self.process_ast(node.lhs);
//            let rhs = self.process_ast(node.rhs);
//            // pushAncestry(result, result.rhs)
//            Box::new(Apply(ApplyNode {
//                position: node.position,
//                lhs,
//                rhs,
//            }))
//        }
//        FunctionCall(node) => {
//            let mut arguments = Vec::new();
//            for arg in node.arguments {
//                let arg = self.process_ast(arg);
//                // pushAncestory
//                arguments.push(arg);
//            }
//            let procedure = self.process_ast(node.procedure);
//            Box::new(FunctionCall(FunctionCallNode {
//                position: node.position,
//                arguments,
//                procedure,
//            }))
//        }
//        PartialFunctionCall(node) => {
//            let mut arguments = Vec::new();
//            for arg in node.arguments {
//                let arg = self.process_ast(arg);
//                // pushAncestory
//                arguments.push(arg);
//            }
//            let procedure = self.process_ast(node.procedure);
//            Box::new(PartialFunctionCall(FunctionCallNode {
//                position: node.position,
//                arguments,
//                procedure,
//            }))
//        }
//        LambdaFunction(node) => {
//            let body = self.process_ast(node.body);
//            Box::new(LambdaFunction(LambdaNode {
//                position: node.position,
//                arguments: node.arguments,
//                body,
//            }))
//            // TODO: Tail call optimization
//        }
//        // Order by
//        //  LHS is the array to be ordered
//        //  RHS defines the terms
//        OrderBy(node) => {
//            let mut lhs = self.process_ast(node.lhs);
//            let mut terms = Vec::new();

//            for term in node.rhs {
//                let expression = self.process_ast(term.expression);
//                // pushAncestory
//                terms.push(SortTermNode {
//                    position: term.position,
//                    descending: term.descending,
//                    expression,
//                })
//            }

//            let sort = Box::new(Sort(SortNode {
//                position: node.position,
//                terms,
//            }));

//            if let Path(ref mut node) = lhs.as_mut() {
//                node.steps.push(sort);
//                lhs
//            } else {
//                Box::new(Path(PathNode {
//                    steps: vec![sort],
//                    seeking_parent: vec![],
//                    keep_singleton_array: false,
//                }))
//            }
//        }
//        // // Positional variable binding
//        // IndexVariableBind(node) => {

//        // },
//        // // Context variable binding
//        // FocusVariableBind(node) => {

//        // }
//        // Group by
//        //  LHS is a step or a predicated step
//        //  RHS is the object constructor expression
//        // GroupBy(node) => {
//        //     let mut result = self.process_ast(node.lhs);
//        //     result
//        // }
//        // Predicated step:
//        //  LHS is a step or a predicated step
//        //  RHS is the predicate expression
//        //ArrayPredicate(node) => {
//        //    let mut result = self.process_ast(node.lhs);
//        //    let mut step = &result;
//        //    let mut is_stages = false;

//        //    if let Path(node) = *result {
//        //        if node.steps.len() > 0 {
//        //            is_stages = true;
//        //            step = node.steps.last().unwrap();
//        //        }
//        //    }
//        //    //                         if (typeof step.group !== 'undefined') {
//        //    //                             throw {
//        //    //                                 code: "S0209",
//        //    //                                 stack: (new Error()).stack,
//        //    //                                 position: expr.position
//        //    //                             };
//        //    //                         }
//        //    //
//        //    //

//        //    let predicate = self.process_ast(node.rhs);

//        //    // /*
//        //    //                         var predicate = processAST(expr.rhs);
//        //    //                         if(typeof predicate.seekingParent !== 'undefined') {
//        //    //                             predicate.seekingParent.forEach(slot => {
//        //    //                                 if(slot.level === 1) {
//        //    //                                     seekParent(step, slot);
//        //    //                                 } else {
//        //    //                                     slot.level--;
//        //    //                                 }
//        //    //                             });
//        //    //                             pushAncestry(step, predicate);
//        //    //                         }
//        //    //                         step[type].push({type: 'filter', expr: predicate, position: expr.position});
//        //    //                         break;
//        //    // // */
//        //}
//        Add(node) => binary!(Add, node),
//        Subtract(node) => binary!(Subtract, node),
//        Multiply(node) => binary!(Multiply, node),
//        Divide(node) => binary!(Divide, node),
//        Modulus(node) => binary!(Modulus, node),
//        Equal(node) => binary!(Equal, node),
//        LessThan(node) => binary!(LessThan, node),
//        GreaterThan(node) => binary!(GreaterThan, node),
//        NotEqual(node) => binary!(NotEqual, node),
//        LessThanEqual(node) => binary!(LessThanEqual, node),
//        GreaterThanEqual(node) => binary!(GreaterThanEqual, node),
//        Concat(node) => binary!(Concat, node),
//        And(node) => binary!(And, node),
//        Or(node) => binary!(Or, node),
//        In(node) => binary!(In, node),
//        Range(node) => binary!(Range, node),
// _ => node,
// }
// }

//fn resolve_ancestry(&self, path: &mut PathNode) {
//    // TODO
//}

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
    fn parser_tests(source: &str) {
        let _ast = parse(source);
        // use json::stringify_pretty;
        // println!("{}", stringify_pretty(ast.to_json(), 4));
    }
}
