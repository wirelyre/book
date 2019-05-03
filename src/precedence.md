# Operator precedence

# TODO:
* reduce length
* style
* check code

How do you correctly parse expressions with infix operators, like `2+3*4` and
`(5-3)^2`? Such expressions can be tricky to parse because some operators bind
more tightly than others. For instance, `2+3*4` should be exactly the same as
`2 + (3*4)`.

Consider a calculator language with numbers and these operators: `+ - * / ^`.
Addition and subtraction should have the lowest operator precedence;
multiplication and division should have higher precedence, exponentiation
should have even higher precedence, and parentheses should have the highest
precedence of all.

```pest
number = { ASCII_DIGIT+ }
op_add = { "+" }
op_sub = { "-" }
op_mul = { "*" }
op_div = { "/" }
op_exp = { "^" }
WHITESPACE = _{ " " }
```

## Directly in the grammar

This is the traditional method of parsing expressions, used commonly in BNF
parsers. PEGs are powerful enough that these expressions can be parsed
directly.

First make a rule for "basic" terms. These include atoms like numbers, but also
parenthesized expressions which will be parsed recursively.

```pest
term = { number | "("~expr~")" }
```

Next make a rule for each level of precedence.

```pest
expr  = _{ prec1 }
prec1 =  { prec2 ~ ((op_add | op_sub) ~ prec2)* }
prec2 =  { prec3 ~ ((op_mul | op_div) ~ prec3)* }
prec3 =  { term ~ (op_exp ~ term)* }
```

When this grammar parses an `expr`, it descends into rules of increasingly high
precedence. Eventually it reaches `prec3`, which eagerly consumes expressions
with exponentiation. `prec2` can now use "exponent expressions" as a primitive
and parse multiplication and division. Likewise, `prec1` can use "product
expressions" as a primitive and parse addition and subtraction.

Each rule thus focuses on just a few operators. Operators (operations?) with
higher strength are delegated to another rule. At the very top of the
precedence hierarchy is a basic term: either a number, or parentheses which
drop parsing back to the lowest precedence. At any particular level, you can be
sure that all operators of higher precedence have already been consumed.

* * *

Although this method is usable for languages with only a few operators, it can
become unwieldy for larger languages. It requires a separate rule for each
level of precedence, which makes the grammar file quite large and introduces a
lot of Rust boilerplate.

It is also very challenging to name each level of precedence in a descriptive
way. Many real-world BNF grammars are hard to read because `sum`, `product`,
and `power` are hardly descriptive, and we're only dealing with five operators
here.

It also does not correctly express associativity. Right-associative operators,
like exponentiation typically is, require Rust code to turn the "stream of
tokens" into a right-associative expression. (Associativity *can* be written
directly in the grammar, at the cost of O(n<sup>2</sup>) parse time.)

## More easily, using `PrecClimber`

Instead, use the [`pest::prec_climber`] module. This module contains a few
tools which massively simplify grammars with operators of various precedence.

In the grammar, simply combine all operators into a single *silent* rule:

[TODO: definitely combine these code blocks into one]

```pest
operator = _{ op_add | op_sub | op_mul | op_div | op_exp }
```

Then make a single `expr` rule which produces a long stream of alternating
`term` and `operator`, without any regard for precedence:

```pest
expr = { term ~ (operator ~ term)* }
```

As before, we still need a rule to distinguish between basic atoms (numbers)
and parenthesized expressions:

```pest
term = { number | "("~expr~")" }
```

That's it! On the Rust side, to use this grammar, we need to construct a
[`PrecClimber`] which handles the details. Specifically, it needs to know which
operators are of which precedence, and whether they are left or right
associative.

```rust
use pest::prec_climber::{Assoc, Operator, PrecClimber};

let climber = PrecClimber::new(vec![
    Operator::new(Rule::op_add, Assoc::Left) | Operator::new(Rule::op_sub, Assoc::Left),
    Operator::new(Rule::op_mul, Assoc::Left) | Operator::new(Rule::op_div, Assoc::Left),
    Operator::new(Rule::op_exp, Assoc::Right)
]);
```

[TODO: mention `lazy_static`]

Operators of lower precedence come first.

To actually "climb" an `expr` rule, (*i.e.* to parse an expression), pass
functions or closures into the `PrecClimber::climb(&self)` method:

```rust
use pest::iterators::Pair;

enum Value {
    Number(f64),
    Add(Box<Value>, Box<Value>), Sub(Box<Value>, Box<Value>),
    Mul(Box<Value>, Box<Value>), Div(Box<Value>, Box<Value>),
    Exp(Box<Value>, Box<Value>),
}

fn parse(expr: Pairs<Rule>) -> Value {
    let primary = |pair| match pair.as_rule() {
        Rule::number => Value::Number(pair.parse().unwrap()),
        Rule::expr => parse(pair.into_inner()),
    };

    let infix = |lhs: Value, op: Pair<Rule>, rhs: Value| {
        let lhs = Box::new(lhs);
        let rhs = Box::new(rhs);

        match op.as_rule() {
            Rule::op_add => Value::Add(lhs, rhs),
            Rule::op_sub => Value::Sub(lhs, rhs),
            Rule::op_mul => Value::Mul(lhs, rhs),
            Rule::op_div => Value::Div(lhs, rhs),
            Rule::op_exp => Value::Exp(lhs, rhs),
        }
    };

    climber.climb(expr, primary, infix)
}
```

### How precedence climbing works

Precedence climbing is basically a form of the "directly in the grammar"
parser, implemented in a single recursive function.

At each call, the function is only concerned about operators of a particular
precedence. For instance, at the start, only addition/subtraction matter.
[TODO: is this a valid simplification of precedence 0 vs 1?] If the climber
encounters an addition or subtraction, it handles that by itself and continues.

But if it encounters an operator of higher precedence, it recursively deals
with expressions of that precedence or higher. If we're parsing `2 + 5 * …`,
once we reach the `*`, we have to parse the upcoming multiplications before
returning to dealing with additions.

When parsing a sub-expression of relatively high precedence (like `5 * 6 + …`),
if it encounters an operator of *lower* precedence, it returns back to the call
site.

The method is called precedence climbing because it is called recursively with
increasing ("climbing") precedence. Each invocation deals only with expressions
of *exactly this* precedence. If a higher-precedence operator is encountered,
it is called recursively with that precedence. If a lower-precedence operator
is encountered, it returns immediately.

[`pest::prec_climber`]: https://docs.rs/pest/2.1/pest/prec_climber/index.html
[`PrecClimber`]: https://docs.rs/pest/2.1/pest/prec_climber/struct.PrecClimber.html
