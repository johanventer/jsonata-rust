# The problem with function signatures

Reference JSONata includes functionality for specifying the type signature of functions ([see the documentation here](http://docs.jsonata.org/programming#function-signatures)). The feature is implemented by creating regular expressions for validating function arguments against the signature.

While there was at one point initial support for function signatures in this implementation, there are a number of issues with them that led to that support being removed.

## Regular expressions everywhere

In the reference implementation, they are implemented by constructing and running regular expressions every time a function with a signature is called. This does not lead to the best performance. In Rust, we could do better by having the function signature specified in a proc macro attribute and generating the argument type checking code at compile time, but other problems with them make this difficult.

## Context argument

One of the function signature specifiers is `-`, which indicates that in the absence of an argument, the context should be passed instead. This makes the following possible:

```
[1, 2, 3].$string()

/* Output: ['1', '2', '3'] */
```

If we look at the signature of `$string`, it is actually `<x-b?:s>`, which says:

- `x-`: The first argument is of any type, but can be replaced with the context if it's missing
- `b?`: The second argument is an optional boolean (in this case it represents whether to pretty print or not)
- `:s`: The return type is string, but this is ignored

Now what if you wanted to pretty print the context value? You might think you could do something like this, which does not work:

```
[1, 2, 3].$string(true)

/* Output: [true, true, true] */
```

What about using [partial function application](http://docs.jsonata.org/programming#partial-function-application)?

```
(
  $s := $string(?, true);
  [1, 2, 3].$s()
)
```

Nope, this doesn't actually parse (`S0208: Parameter 2 of function definition must be a variable name (start with $)`).

Let's try defining our own function to wrap `$string`:

```
  $s := function ($x) { $string($x, true) };
  [1, 2, 3].$s()

  /* Output: undefined */
```

The context is not passed in! To make this work we have to include a function signature indicating that context can be substituted for the first parameter:

```
  $s := function($x)<x-> { $string($x, true) };
  [1, 2, 3].$s()

  /* Output: ['1', '2', '3'] */
```

In truth, we could also just as easily have passed the context `$` directly:

```
[1, 2, 3].$string($, true)

/* Output: ['1', '2', '3'] */
```

So what did the function signature give us? Certainly not type safety, as the first argument is an any, and the return value remains unchecked. It also didn't provide any functionality that we couldn't already do by passing the context `$`.

## Optionals

Most languages that have function signatures with optional arguments will not let you place the optional arguments before non-optional arguments. In JSONata, you can do this and the results are expected, but it's not exactly clear how this is supported as there's no way to pass `undefined` or some other empty value in the place of the optional argument.

## Grammar

Function signatures are specified after the closing parenthesis when declaring a function, and enclosed in `<` and `>` carets.

This overloads the grammar for `<`, making it less [context free](https://en.wikipedia.org/wiki/Context-free_grammar) and more [context sensitive](https://en.wikipedia.org/wiki/Context-sensitive_grammar).

## Conclusion

Overall, it feels like function signatures were somewhat of a bolt-on feature and certainly do not take the place of a proper runtime type system, so we will not support them (unless somebody wants to do the work in such a way that reduces the runtime overhead).

A cleaner idea might be to introduce a mode for type checking function arguments, specified in a more TypeScript-esque way:

```
  $x := function($a: number, $b: string, $c?: boolean) { }
```

This would be backwards-incompatible with reference JSONata, of course, so would have to be opt-in.
