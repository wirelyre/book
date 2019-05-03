use pest::Parser;
use regex::Regex;

#[derive(Parser)]
#[grammar = "awk.pest"]
struct AwkParser;

pub fn parse(program_text: &str) -> Result<Vec<(Pattern, Action)>, pest::error::Error<Rule>> {
    let pairs = AwkParser::parse(Rule::file, program_text)?;

    Ok(unimplemented!())
}

pub struct Action {}

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
