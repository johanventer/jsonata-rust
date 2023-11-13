# Status

_Notes from the original creator of [the forked repo](https://github.com/johanventer/jsonata-rust)_

There's a number of issues to be resolved:

- I've tried to implement structural sharing of the input and the output values, with the minimal number of heap allocations. This was a lot of effort working out the lifetimes, but I'm not actually sure it was worth it
- Code is too spaghetti in some places and could be more Rust-idiomatic
- There's a lot of code that's not very efficient - lots of opportunities for optimization
- There's obviously still a bunch of missing features - I'm really aiming for feature-parity with the reference implementation (within reason)
- The API has not had any real thought put into it yet

# Goals

- Feature-parity with the reference implementation (within reason)
- Clean API and idiomatic code (make the easy things easy, and the complex possible)
- Well documented for users, and easy to onboard for contributors
- Efficient and optimised, at least no low hanging fruit.

There's a few other ideas that are semi-baked or non-existent:

- A command line utility and REPL (semi-baked)
- JSONata-compatible JSON output for the AST, as it's often useful to feed the AST of one expression back into another, particularly for tooling like [jsonata-visual-editor](https://github.com/jsonata-ui/jsonata-visual-editor) and being compatible here would help (non-existent)
- Benchmarks, both to track improvements within the Rust implementation and to compare against [jsonata-js](https://github.com/jsonata-js/jsonata) (non-existent).

Long term stretch goals:

- It would be cool if we could transform the AST to bytecode which can be compiled to WASM or perhaps LLVM IR, so that specific JSONata expressions could be run as native code outside of the evaluator to provide high-performance and scale.
