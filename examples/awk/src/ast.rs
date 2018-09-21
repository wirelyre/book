use pest::Parser;

#[derive(Parser)]
#[grammar = "awk.pest"]
struct AwkParser;

pub fn parse(_program_text: &str) -> Vec<(Pattern, Action)> {
    unimplemented!()
}

pub struct Action {
}

pub enum Pattern {
    Always,
    Begin,
    End,
    Expr(Expr),
    // Range(Expr, Expr),
}

pub enum Lvalue {
    Field(Expr),
    Variable(String),
    /*
    PostDecrement(Box<Lvalue>),
    PostIncrement(Box<Lvalue>),
    PreDecrement(Box<Lvalue>),
    PreIncrement(Box<Lvalue>),
    */
}

pub enum Expr {
    Num(f64),
    String(String),
    Lvalue(Box<Lvalue>),

    Unary(UnaryOp, Box<Expr>),
    Binary(BinaryOp, Box<Expr>),
    // Assignment(AssignmentOp, Box<Lvalue>, Box<Expr>),
    Assignment(Box<Lvalue>, Box<Expr>),

    Conditional(Box<Expr>, Box<Expr>, Box<Expr>),
}

pub enum UnaryOp {
    Not,
    Plus,
    Minus,
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
    Match,
    NonMatch,
    And,
    Or,
}
/*
pub enum AssignmentOp {
    Exponent,
    Modulus,
    Multiply,
    Divide,
    Add,
    Subtract,
    Normal,
}
*/
