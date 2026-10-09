# rbc

A small resistor calculator ("resistor bc") for the command line. It combines
resistors in **series** and **parallel**, understands common SI suffixes, and
works with **complex values** so you can throw impedances into a network too.

Results are printed with the ohm symbol (`Ω`).

## Install

```sh
cargo install --path .
```

## Usage

Pass an expression as arguments for a one-shot calculation:

```sh
$ rbc "4.7k // 3k"
1831.1688311688315 Ω
```

Run it with no arguments to get an interactive REPL:

```sh
$ rbc
> 4.7k + 3k
7700 Ω
> (4 // 3) // 2
0.9230769230769231 Ω
```

The REPL keeps command history (stored in the per-user cache directory) and
shows a live preview of the answer as you type.

## Syntax

| Input        | Meaning                                            |
| ------------ | -------------------------------------------------- |
| `+`          | series: `a + b` = `a + b`                          |
| `//`         | parallel: `a // b` = `1 / (1/a + 1/b)`             |
| `(` `)`      | grouping                                           |
| `k`, `K`     | ×1000 (e.g. `4.7k` → 4700)                         |
| `M`          | ×1,000,000 (e.g. `1M` → 1000000)                   |
| `m`          | ×0.001 (e.g. `4.7m` → 0.0047)                      |
| `i`, `j`     | imaginary unit (e.g. `3j`, `4.7kj`, or bare `i`)   |

Operators share the same precedence and evaluate **left to right**, so
`4 // 3 + 5` means `(4 // 3) + 5` and `4 + 3 // 2` means `(4 + 3) // 2`.
Use parentheses when you want a different order.

Values may be complex: a numeric literal followed by `i` or `j` is purely
imaginary, and the two spellings are interchangeable. This makes it possible to
combine impedances:

```sh
$ rbc "4 // 3j"
1.44+1.92i Ω
```

A parallel branch containing `0` yields `0` (an open branch shorts the pair).

## Development

```sh
cargo test
```

The test suite covers tokenizing, suffix and imaginary parsing, operator
precedence, parentheses, error cases, and complex arithmetic.

## License

No license has been declared for this project yet.
