# Operator precedence (WIP)

This chapter will discuss two methods of dealing with operator precedence:
directly in the PEG grammar, and using a `PrecClimber`. It will probably also
include an explanation of how precedence climbing works.

* * *

How do you correctly parse expressions with infix operators, like `2+3*4` and `(5-3)^2`?

## Directly in the grammar

PEGs are powerful enough that these expressions can be parsed directly.
