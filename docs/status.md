# Status

This is my first real Rust project, so I'm learning as I go. There's plenty of non-idiomatic code, and currently there's a bunch of core JSONata features that still need to be implemented.

There's a number of issues to be resolved:

- I've tried to implement structural sharing of the input and the output values, with the minimal number of heap allocations. This was a lot of effort working out the lifetimes, but I'm not actually sure it was worth it
- Code is too spaghetti in some places and could be more Rust-idiomatic
- There's a lot of code that's not very efficient - lots of opportunities for optimization
- There's obviously still a bunch of missing features - I'm really aiming for feature-parity with the reference implementation (within reason)
- The API has not had any real thought put into it yet

# Goals

Things that I would like to achieve:

- Feature-parity with the reference implementation (within reason)
- Clean API and idiomatic code (make the easy things easy, and the complex possible)
- Well documented for users, and easy to onboard for contributors
- Efficient and optimised, at least no low hanging fruit

There's a few other ideas that are semi-baked or non-existent:

- A command line utility and REPL (semi-baked)
- JSONata-compatible JSON output for the AST, as it's often useful to feed the AST of one expression back into another, particularly for tooling like [jsonata-visual-editor](https://github.com/jsonata-ui/jsonata-visual-editor) and being compatible here would help (non-existent)

Long term stretch goals:

- It would be cool if we could transform the AST to bytecode which can be compiled to WASM or perhaps LLVM IR, so that specific JSONata expressions could be run as native code outside of the evaluator to provide high-performance and scale.

## Benchmarks

I would really like to implement some benchmarks for tracking overall performance as the code changes at some point.

In particular, I would like to make use of [criterion](https://docs.rs/criterion/latest/criterion/).

It would also be good to benchmark against Javascript JSONata, but I fear this version will never
compete in the browser environment because of the JSON parsing/stringification on the way in and out across the WASM boundary.
However, it might be possible to compare the evaluation time directly in Node, if we make sure to give Node some JIT warmup to make it fair.
