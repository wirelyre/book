use pest::error::Error;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use regex::Regex;

#[derive(Parser)]
#[grammar = "awk.pest"]
struct AwkParser;

type Result<T> = std::result::Result<T, Error<Rule>>;

pub fn parse(program_text: &str) -> Result<Vec<(Pattern, Action)>> {
    let pairs = AwkParser::parse(Rule::file, program_text)?;
    let mut program = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::EOI => (),
            Rule::pattern_action => {
                let mut pairs = pair.into_inner();
                let pattern = parse_pattern(pairs.next().unwrap().into_inner())?;
                let action = parse_action(pairs.next().unwrap().into_inner())?;
                program.push((pattern, action));
            }
            Rule::pattern_only => {
                let mut pairs = pair.into_inner();
                let pattern = parse_pattern(pairs.next().unwrap().into_inner())?;
                program.push((
                    pattern,
                    Action {
                        stmts: vec![Statement::Print(Expr::Unary(
                            UnaryOp::Field,
                            Box::new(Expr::Num(0.)),
                        ))],
                    },
                ));
            }
            Rule::action_only => {
                let mut pairs = pair.into_inner();
                let action = parse_action(pairs.next().unwrap().into_inner())?;
                program.push((Pattern::Always, action));
            }
            _ => unreachable!(),
        }
    }

    Ok(program)
}

fn parse_pattern(mut pairs: Pairs<Rule>) -> Result<Pattern> {
    let pat = pairs.next().unwrap();
    let parsed = match pat.as_rule() {
        Rule::begin_pattern => Pattern::Begin,
        Rule::end_pattern => Pattern::End,
        Rule::regex => {
            let mut s = String::new();
            let mut chars = pat.as_str().chars().skip(1);
            while let Some(c) = chars.next() {
                match c {
                    '\\' => s.push(chars.next().unwrap()),
                    '/' => break,
                    _ => s.push(c),
                }
            }

            // TODO: error handling
            Pattern::Regex(Regex::new(&s).map_err(|e| {
                Error::new_from_span(
                    pest::error::ErrorVariant::CustomError {
                        message: format!("error in regular expression: {}", e),
                    },
                    pat.as_span(),
                )
            })?)
        }
        Rule::expr => Pattern::Expr(parse_expr(pat.into_inner())?),
        _ => unreachable!(),
    };

    if pairs.next().is_some() {
        unimplemented!();
    }

    Ok(parsed)
}
fn parse_action(pairs: Pairs<Rule>) -> Result<Action> {
    fn parse_lvalue(mut pairs: Pairs<Rule>) -> Result<Lvalue> {
        let pair = pairs.next().unwrap();
        Ok(match pair.as_rule() {
            Rule::variable => Lvalue::Variable(pair.as_str().to_string()),
            Rule::field_ref => Lvalue::Field(parse_expr(pair.into_inner())?),
            _ => unreachable!(),
        })
    }

    let stmts: Result<Vec<_>> = pairs
        .map(|pair| match pair.as_rule() {
            Rule::print_statement => Ok(Statement::Print(parse_expr(pair.into_inner())?)),
            Rule::assignment_statement => {
                let mut pairs = pair.into_inner();
                let lvalue = parse_lvalue(pairs.next().unwrap().into_inner())?;
                let expr = parse_expr(pairs.next().unwrap().into_inner())?;
                Ok(Statement::Assignment(lvalue, expr))
            }
            _ => unreachable!(),
        })
        .collect();

    Ok(Action { stmts: stmts? })
}

lazy_static::lazy_static! {
    static ref EXPR_CLIMBER: pest::prec_climber::PrecClimber<Rule> = {
        use Rule::*;
        use pest::prec_climber::{Assoc::{Left, Right}, Operator, PrecClimber};

        PrecClimber::new(vec![
            Operator::new(op_or, Left),
            Operator::new(op_and, Left),

            Operator::new(op_leq, Left) | Operator::new(op_geq, Left) |
            Operator::new(op_eq, Left)  | Operator::new(op_neq, Left) |
            Operator::new(op_lt, Left)  | Operator::new(op_gt, Left),

            Operator::new(op_concat, Left),

            Operator::new(op_add, Left) | Operator::new(op_sub, Left),
            Operator::new(op_mul, Left) | Operator::new(op_div, Left) |
                Operator::new(op_mod, Left),
            Operator::new(op_exp, Right),
        ])
    };
}

fn parse_expr(pairs: Pairs<Rule>) -> Result<Expr> {
    let primary = |pair: Pair<Rule>| {
        Ok(match pair.as_rule() {
            Rule::expr => parse_expr(pair.into_inner())?,
            Rule::literal => {
                let pair = pair.into_inner().next().unwrap();
                match pair.as_rule() {
                    Rule::number => Expr::Num(pair.as_str().parse().unwrap()),
                    Rule::string => parse_string(pair.into_inner()),
                    _ => unreachable!(),
                }
            }
            Rule::function_call => parse_function_call(pair.into_inner())?,
            Rule::variable => Expr::Variable(pair.as_str().to_string()),
            Rule::field_ref => {
                let expr = parse_expr(pair.into_inner());
                Expr::Unary(UnaryOp::Field, Box::new(expr?))
            }
            _ => unreachable!(),
        })
    };

    let infix = |lhs: Result<Expr>, op: Pair<Rule>, rhs: Result<Expr>| {
        let op = match op.as_rule() {
            Rule::op_or => BinaryOp::Or,
            Rule::op_and => BinaryOp::And,
            Rule::op_leq => BinaryOp::LessOrEqual,
            Rule::op_geq => BinaryOp::GreaterOrEqual,
            Rule::op_eq => BinaryOp::Equal,
            Rule::op_neq => BinaryOp::NotEqual,
            Rule::op_lt => BinaryOp::Less,
            Rule::op_gt => BinaryOp::Greater,
            Rule::op_concat => BinaryOp::Concat,
            Rule::op_add => BinaryOp::Add,
            Rule::op_sub => BinaryOp::Subtract,
            Rule::op_mul => BinaryOp::Multiply,
            Rule::op_div => BinaryOp::Divide,
            Rule::op_mod => BinaryOp::Modulus,
            Rule::op_exp => BinaryOp::Exponent,
            _ => unreachable!(),
        };

        Ok(Expr::Binary(op, Box::new(lhs?), Box::new(rhs?)))
    };

    EXPR_CLIMBER.climb(pairs, primary, infix)
}

fn parse_string(pairs: Pairs<Rule>) -> Expr {
    let mut string = String::with_capacity(pairs.as_str().len());

    string.extend(pairs.map(|pair| match pair.as_rule() {
        Rule::regular_chars => pair.as_str(),
        Rule::escaped_char => match pair.as_str() {
            "\\r" => "\r",
            "\\n" => "\n",
            "\\t" => "\t",
            "\\\\" => "\\",
            "\\\"" => "\"",
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }));

    Expr::String(string)
}

fn parse_function_call(mut pairs: Pairs<Rule>) -> Result<Expr> {
    let func_name = pairs.next().unwrap();
    let args: Result<Vec<_>> = pairs.map(|pair| parse_expr(pair.into_inner())).collect();
    let mut args = args?;

    let expected_args = match func_name.as_str() {
        "rand" => 0,
        "exp" | "log" | "sqrt" | "int" => 1,
        _ => unreachable!(),
    };

    if args.len() != expected_args {
        return Err(Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: format!(
                    "expected {} arguments to function but found {}",
                    expected_args,
                    args.len()
                ),
            },
            func_name.as_span(),
        ));
    }

    Ok(match func_name.as_str() {
        "rand" => Expr::NullaryFunc(rand::random::<f64>),
        "exp" => Expr::UnaryFunc(f64::exp, Box::new(args.pop().unwrap())),
        "log" => Expr::UnaryFunc(f64::ln, Box::new(args.pop().unwrap())),
        "sqrt" => Expr::UnaryFunc(f64::sqrt, Box::new(args.pop().unwrap())),
        "int" => Expr::UnaryFunc(f64::trunc, Box::new(args.pop().unwrap())),
        _ => unreachable!(),
    })
}

pub struct Action {
    pub stmts: Vec<Statement>,
}

pub enum Pattern {
    Always,
    Begin,
    End,
    Expr(Expr),
    Regex(Regex),
    // Range(Expr, Expr),
}

pub enum Lvalue {
    Field(Expr),
    Variable(String),
}

pub enum Statement {
    Assignment(Lvalue, Expr),
    Print(Expr),
}

pub enum Expr {
    Num(f64),
    String(String),
    Variable(String),

    Unary(UnaryOp, Box<Expr>),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),

    NullaryFunc(fn() -> f64),
    UnaryFunc(fn(f64) -> f64, Box<Expr>),
}

pub enum UnaryOp {
    Field,
}
pub enum BinaryOp {
    Exponent,
    Multiply,
    Divide,
    Modulus,
    Add,
    Subtract,
    Concat,
    Less,
    LessOrEqual,
    NotEqual,
    Equal,
    Greater,
    GreaterOrEqual,
    And,
    Or,
}
