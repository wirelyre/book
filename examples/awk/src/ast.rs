use pest::Parser;
use regex::Regex;

#[derive(Parser)]
#[grammar = "awk.pest"]
struct AwkParser;

pub fn parse(program_text: &str) -> Result<Vec<(Pattern, Action)>, pest::error::Error<Rule>> {
    let pairs = AwkParser::parse(Rule::file, program_text)?;
    let mut program = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::EOI => (),
            Rule::pattern_action => {
                let mut pairs = pair.into_inner();
                let pattern = parse_pattern(pairs.next().unwrap().into_inner());
                let action = parse_action(pairs.next().unwrap().into_inner());
                program.push((pattern, action));
            }
            Rule::pattern_only => {
                let mut pairs = pair.into_inner();
                let pattern = parse_pattern(pairs.next().unwrap().into_inner());
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
                let action = parse_action(pairs.next().unwrap().into_inner());
                program.push((Pattern::Always, action));
            }
            _ => unreachable!(),
        }
    }

    Ok(program)
}

fn parse_pattern(mut pairs: pest::iterators::Pairs<Rule>) -> Pattern {
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
            Pattern::Regex(Regex::new(&s).expect("invalid regex"))
        }
        Rule::expr => Pattern::Expr(parse_expr(pat.into_inner())),
        _ => unreachable!(),
    };

    if pairs.next().is_some() {
        unimplemented!();
    }

    parsed
}
fn parse_action(pairs: pest::iterators::Pairs<Rule>) -> Action {
    fn parse_lvalue(mut pairs: pest::iterators::Pairs<Rule>) -> Lvalue {
        let pair = pairs.next().unwrap();
        match pair.as_rule() {
            Rule::variable => Lvalue::Variable(pair.as_str().to_string()),
            Rule::field_ref => Lvalue::Field(parse_expr(pair.into_inner())),
            _ => unreachable!(),
        }
    }

    Action {
        stmts: pairs
            .map(|pair| match pair.as_rule() {
                Rule::print_statement => Statement::Print(parse_expr(pair.into_inner())),
                Rule::assignment_statement => {
                    let mut pairs = pair.into_inner();
                    let lvalue = parse_lvalue(pairs.next().unwrap().into_inner());
                    let expr = parse_expr(pairs.next().unwrap().into_inner());
                    Statement::Assignment(lvalue, expr)
                }
                _ => unreachable!(),
            })
            .collect(),
    }
}

fn parse_expr(_pairs: pest::iterators::Pairs<Rule>) -> Expr {
    unimplemented!()
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

    Unary(UnaryOp, Box<Expr>),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
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
