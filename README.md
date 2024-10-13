# cranelift-example

My example JIT compiler using cranelift.

I wrote this code to study cranelift and test the binary size of the generated JIT compiler.

## Toy Language Spec

- Every value is `i32`.
- A number, infix operator (+, -, *, /, ==), parentheses, and function `print`, `rand` are supported.
  - See `src/grammar.pest` for syntax.
	- `print` prints an argument to stdout.
	- `rand` returns a random number from 0 to the argument.
  - Note that operator precedence is not supported, and every operator is right-associative.
- The program takes four integer inputs, which are represented as `$0`, `$1`, `$2`, and `$3`.
- The program is a single expression.

For example,

```
2 * print ($0 * $0 + $1 * $1)
```

The above program calculates the square or norm `($0, $1)` and prints it.
Also, the result of the program is `2 * (the square of the norm)`.

## Reference

- [https://rodrigodd.github.io/2022/11/26/bf_compiler-part3.html]
