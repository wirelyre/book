# Example: Calculator

Let's write a simple command-line calculator that can evaluate arithmetic
expressions using **operator precedence** &mdash; that is, following the order
of operations.

Here is an example session using this calculator:

```
1 + 2
 = 3
1 + 2 ^ 3
 = 9
(1 + 2) ^ 3
 = 27
```

## Writing the grammar

Start by [initializing a new project] using Cargo. In addition to `pest` and
`pest_derive`, add the dependency `lazy_static = "1.3"`.

We'll be parsing numbers into [`f64`] using Rust's standard library. We'll
accept a fairly flexible definition of real numbers, including an optional
decimal point and an optional exponent. Since the exponent is also an integer,
we can break that into a separate rule `int`. In `src/grammar.pest`:

```pest
num = @{ int ~ ("." ~ ASCII_DIGIT*)? ~ (^"e" ~ int)? }
    int = { ("+" | "-")? ~ ASCII_DIGIT+ }
```

Note that `num` is [*atomic*], since whitespace is not allowed within a number.
The notation `^"e"` accepts [case-insensitively] either `e` or `E`.

We need to parse mathematical operators. Since we'll be using [`PrecClimber`],
we need to have a separate rule for each operator. For convenience, we'll also
wrap them into a single `operation` rule:

```pest
operation = _{ add | subtract | multiply | divide | power }
    add      = { "+" }
    subtract = { "-" }
    multiply = { "*" }
    divide   = { "/" }
    power    = { "^" }
```

Now let's think about arithmetic expressions. Ignoring operator precedence for
a moment, we can think of an expression as a list of "things" separated by
operators. These "things", often called *terms*, can be either numbers, or else
entire expressions contained in parentheses.

Here's the trick: We'll avoid dealing with operator precedence in the grammar
file entirely! Instead, we'll delay until we consume the parse result. It is
possible to deal with precedence [in the grammar], but in this case it is much
simpler not to.

```pest
expr = { term ~ (operation ~ term)* }
term = _{ num | "(" ~ expr ~ ")" }
```

[TODO: rewrite this paragraph]

This is more complicated than it looks. Pay special attention to `term`:
Although it refers to itself recursively, the left parenthesis `(` ansures that
parsing a `term` always advances forward in the input. In the end, a parsed
`expr` consists of a list of either `num` or another parenthesized `expr`,
separated by `add`, `subtract`, *etc*.

Finally, we need a rule that wraps `expr` and makes sure it matches the whole
input, and [implicit whitespace].

```pest
calculation = _{ SOI ~ expr ~ EOI }

WHITESPACE = _{ " " | "\t" }
```

## Parsing

[initializing a new project]: csv.md#setup
[`f64`]: https://doc.rust-lang.org/std/primitive.f64.html
[*atomic*]: ../grammars/syntax.md#atomic
[case-insensitively]: ../grammars/syntax.html#terminals
[`PrecClimber`]: https://docs.rs/pest/2.1/pest/prec_climber/struct.PrecClimber.html
[in the grammar]: ../precedence.html#directly-in-the-grammar
[implicit whitespace]: ../grammars/syntax.html#implicit-whitespace
