# jsonata-rust

An (incomplete) implementation of [JSONata](https://jsonata.org) in Rust.

## What is JSONata?

From the JSONata website:

- Lightweight query and transformation language for JSON data
- Inspired by the location path semantics of XPath 3.1
- Sophisticated query expressions with minimal syntax
- Built in operators and functions for manipulating and combining data
- Create user-defined functions
- Format query results into any JSON output structure

Read the full documentation [here](https://docs.jsonata.org/overview.html), and give it a go in the exerciser environment [here](https://try.jsonata.org).

## Goals

This crate implements JSONata in Rust, and as such can take JSON input, parse it, evaluate it against a JSONata expression. There's a few more things I'm working towards:

- A command line utility and REPL
- WASM bindings to run directly in the browser
- Function signature declarative macro supporting JSONata's signature syntax
- JSONata-compatible JSON output for the AST, as it's often useful to feed the AST of one expression back into another, particularly for tooling like [jsonata-visual-editor](https://github.com/jsonata-ui/jsonata-visual-editor) and being compatible here would help.

Long term, I would like to try implementing a transformation from the AST to bytecode which can be compiled to WASM or perhaps LLVM IR, so that specific JSONata expressions could be run as native code
outside of the evaluator to provide high-performance and scale.

## Status

This is my first real Rust project, so I'm learning as I go. There's plenty of non-idiomatic code, and currently there's a bunch of core JSONata features that need to be implemented. There's a TODO section below with a high-level list.

Currently, the implementation passes over 400 of the tests from the JSONata test suite.

## TODO

There's still a lot left to do.

### Features

There are a number of JSONata features which are not yet implemented:

- [ ] Descendents, parents, wildcards - requires ancestory algorithm
- [ ] Context and index bind variables
- [ ] Regular expressions
- [ ] Lots of functions remain unimplemented
- [ ] Function signature validation
- [ ] Object transforms
- [ ] Sorting
- [ ] Partial function application

### Code issues

There's a bunch of issues with the code - I'm learning Rust as I go, so as I learn more, the code improves. However, here's some issues I know about:

- [ ] Bumpalo-based value arena leaks arrays
- [ ] Bumpalo arena should be per evaluation, and reset between evaluations
- [ ] Code is too spaghetti in some places, needs to be more Rust-idiomatic
- [ ] There's a lot of code that's not very efficient, lots of opportunities for optimization
- [ ] Function signature code is not very good, both the parsing and the macro

### Tests

There's a couple of missing things in the test suite tests which run the JSONata test suite:

- [ ] Implement time limit
- [ ] Implement depth

### Benchmarks

I would really like to implement some benchmarks for tracking overall performance as the code changes.
In particular, I would like to make use of [criterion](https://docs.rs/criterion/latest/criterion/).

It would also be good to benchmark against Javascript JSONata, but I fear this version will never
compete in the browser environment because of the JSON parsing/stringification on the way in and out.
However, it might be possible to compare the evaluation time directly without that.
