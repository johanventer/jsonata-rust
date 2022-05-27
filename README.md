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

## Differences from reference JSONata

### Function signatures are not supported

Function signatures have their problems as described [here](docs/function-signatures.md), and are not supported by this implementation.

Most of the JSONata functions, however, support being passed the context as the first argument as dictated by their signature, e.g:

```
["Hello", "world"].$substring(1, 2)

/* Output: ["el", "or"] */
```

This is implemented in each built-in function itself. For example, if `$string` sees that it is called with no arguments, it will use the current context.

In addition, for all the built-in functions, type checking of arguments is also implemented directly in the functions themselves so that you get eqivalent runtime errors for passing the wrong things to these functions as you would in reference JSONata.

## Status

This is my first real Rust project, so I'm learning as I go. There's plenty of non-idiomatic code, and currently there's a bunch of core JSONata features that still need to be implemented. There's a TODO section below with a high-level list.

Currently, the implementation passes over 400 of the tests from the JSONata test suite.

## Goals

This crate implements JSONata in Rust, and as such can take JSON input, parse it, evaluate it against a JSONata expression. There's a few other ideas in here that are in semi-baked state or non-existent:

- A command line utility and REPL (semi-baked)
- WASM bindings to run directly in the browser (semi-baked)
- JSONata-compatible JSON output for the AST, as it's often useful to feed the AST of one expression back into another, particularly for tooling like [jsonata-visual-editor](https://github.com/jsonata-ui/jsonata-visual-editor) and being compatible here would help (non-existent)

It would be cool if we could transform the AST to bytecode which can be compiled to WASM or perhaps LLVM IR, so that specific JSONata expressions could be run as native code outside of the evaluator to provide high-performance and scale.

## TODO

There's still a lot left to do.

### Features

There are a number of JSONata features which are not yet implemented:

- Parent operator
- Context and index bind variables
- Regular expressions
- Lots of functions remain unimplemented
- Object transforms
- Sorting
- Partial function application

### Code issues

There's a bunch of issues with the code - I'm learning Rust as I go, so as I learn more, the code improves. However, here's some issues I know about:

- I've tried to implement structural sharing of the input and the output values, with the minimal number of heap allocations. This was a lot of effort working out the lifetimes, but I'm not actually sure it was worth it.
- Currently using the same JsonAta for performing multiple evaluations will be additive in terms of memory - the original result and input are tied to the lifetime of JsonAta, so reusing it just keeps using memory in the arena.
- Code is too spaghetti in some places, needs to be more Rust-idiomatic
- There's a lot of code that's not very efficient, lots of opportunities for optimization
- Function signature code is not very good, both the parsing and the macro - I had never written a proc-macro before and just ended up emitting a lot of strings of code, which is probably bad form.

### Tests

There's a couple of missing things in the testsuite tests which run the JSONata test suite, namely the time limit and depth test case options.

That being said, it passes over 400 of the JSONata tests, you can run them like this:

```bash
cargo test testsuite
```

In `tests/testsuite/groups` are the tests groups that are passing, while `tests/testsuite/skip` contains the groups that still require feature implementation.

### Benchmarks

I would really like to implement some benchmarks for tracking overall performance as the code changes.
In particular, I would like to make use of [criterion](https://docs.rs/criterion/latest/criterion/).

It would also be good to benchmark against Javascript JSONata, but I fear this version will never
compete in the browser environment because of the JSON parsing/stringification on the way in and out.
However, it might be possible to compare the evaluation time directly in Node, if we make sure to give Node some JIT warmup to make it fair.

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
