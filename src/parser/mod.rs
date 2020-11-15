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

pub mod ast;
mod parser;
mod postprocess;
mod symbol;
mod tokenizer;

pub use parser::parse;
