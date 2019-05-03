use std::collections::HashMap;
use std::fmt;
use std::io::BufRead;

use crate::ast::{self, Expr, Pattern};

#[derive(Clone)]
pub enum Value {
    Number(f64),
    String(String),
}

impl Value {
    fn truthy(&self) -> bool {
        match self {
            Value::Number(n) if *n == 0. => false,
            Value::String(s) if s == "" => false,
            _ => true,
        }
    }
    fn as_num(&self) -> f64 {
        match self {
            Value::Number(n) => *n,
            Value::String(s) => s.parse().unwrap_or(0.),
        }
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Value {
        Value::Number(if b { 1. } else { 0. })
    }
}
impl From<f64> for Value {
    fn from(f: f64) -> Value {
        Value::Number(f)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(n) => f.write_fmt(format_args!("{}", n)),
            Value::String(s) => f.write_str(s),
        }
    }
}

#[derive(Default)]
pub struct Environment {
    line: Line,
    variables: HashMap<String, Value>,
}
pub type Program = Vec<(ast::Pattern, ast::Action)>;

pub struct Line {
    // invariant: at least one of `fields` and `string` is Some
    //   assert!(fields.is_some() || string.is_some())
    fields: Option<Vec<Value>>,
    string: Option<String>,
}

impl Line {
    pub fn new(s: &str) -> Line {
        Line {
            fields: None,
            string: Some(s.to_string()),
        }
    }

    fn get_field_ref(&mut self, i: usize) -> &mut Value {
        if self.fields.is_none() {
            self.fields = Some(
                self.string
                    .as_ref()
                    .unwrap()
                    .split_whitespace()
                    .map(|field| Value::String(field.to_string()))
                    .collect(),
            );
        }

        let fields = self.fields.as_mut().unwrap();

        if fields.len() < i + 1 {
            fields.resize(i + 1, Value::String(String::new()));
        }

        &mut fields[i]
    }

    fn get_string_ref(&mut self) -> &mut String {
        if self.string.is_none() {
            self.string = Some(itertools::join(self.fields.as_ref().unwrap(), " "));
        }

        self.string.as_mut().unwrap()
    }

    pub fn get_field(&mut self, field: usize) -> &Value {
        self.get_field_ref(field)
    }
    pub fn set_field(&mut self, field: usize, value: Value) {
        *self.get_field_ref(field) = value;
        self.string = None;
    }
    pub fn get_string(&mut self) -> &str {
        self.get_string_ref()
    }
    pub fn set_string(&mut self, to: String) {
        *self.get_string_ref() = to;
        self.fields = None;
    }
}

impl Default for Line {
    fn default() -> Line {
        Line {
            fields: None,
            string: Some(String::new()),
        }
    }
}

impl Environment {
    pub fn run_begin(&mut self, p: &Program) {
        for (pattern, action) in p {
            match pattern {
                Pattern::Begin => action.run(self),
                _ => (),
            }
        }
    }
    pub fn run_end(&mut self, p: &Program) {
        for (pattern, action) in p {
            match pattern {
                Pattern::End => action.run(self),
                _ => (),
            }
        }
    }
    pub fn run_file<B: BufRead>(&mut self, p: &Program, b: &mut B) {
        let mut record = String::new();

        while b.read_line(&mut record).unwrap() > 0 {
            self.line = Line::new(&record);

            for (pattern, action) in p {
                match pattern {
                    Pattern::Always => action.run(self),
                    Pattern::Expr(e) => {
                        if e.eval(self).truthy() {
                            action.run(self);
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}

impl ast::Action {
    fn run(&self, _env: &mut Environment) {
        unimplemented!()
    }
}

impl ast::Statement {
    fn eval(&self, env: &mut Environment) {
        use ast::Statement;

        match self {
            Statement::Assignment(lv, expr) => {
                let val = expr.eval(env);
                lv.assign(env, val)
            }
            Statement::Print(expr) => print!("{}", expr.eval(env)),
        }
    }
}

impl ast::Expr {
    fn eval(&self, env: &mut Environment) -> Value {
        use ast::{BinaryOp, UnaryOp};

        match self {
            Expr::Num(n) => Value::Number(*n),
            Expr::String(s) => Value::String(s.clone()),

            Expr::Unary(UnaryOp::Field, n) => {
                let n = n.eval(env).as_num() as usize;

                if n == 0 {
                    Value::String(env.line.get_string().to_string())
                } else {
                    env.line.get_field(n).clone()
                }
            }
            Expr::Binary(op, lhs, rhs) => {
                use BinaryOp::*;
                let lhs = lhs.eval(env);
                let rhs = rhs.eval(env);

                match op {
                    Add => (lhs.as_num() + rhs.as_num()).into(),
                    Subtract => (lhs.as_num() - rhs.as_num()).into(),
                    Multiply => (lhs.as_num() * rhs.as_num()).into(),
                    Divide => (lhs.as_num() / rhs.as_num()).into(),
                    Modulus => {
                        let lhs = lhs.as_num() as i64;
                        let rhs = rhs.as_num() as i64;
                        lhs.checked_rem(rhs)
                            .map(|i| i as f64)
                            .unwrap_or(std::f64::INFINITY)
                            .into()
                    }
                    Exponent => (lhs.as_num().powf(rhs.as_num())).into(),

                    Concat => {
                        let mut s = lhs.to_string();
                        s.push_str(&rhs.to_string());
                        Value::String(s)
                    }

                    Less => (lhs.as_num() < rhs.as_num()).into(),
                    LessOrEqual => (lhs.as_num() <= rhs.as_num()).into(),
                    NotEqual => (lhs.as_num() != rhs.as_num()).into(),
                    Equal => (lhs.as_num() == rhs.as_num()).into(),
                    Greater => (lhs.as_num() > rhs.as_num()).into(),
                    GreaterOrEqual => (lhs.as_num() >= rhs.as_num()).into(),

                    // TODO: short-circuiting
                    And => (lhs.truthy() && rhs.truthy()).into(),
                    Or => (lhs.truthy() || rhs.truthy()).into(),
                }
            }
        }
    }
}

impl ast::Lvalue {
    fn assign(&self, env: &mut Environment, val: Value) {
        use ast::Lvalue::*;

        match self {
            Field(n) => {
                let n = n.eval(env).as_num() as usize;
                if n == 0 {
                    env.line.set_string(format!("{}", val));
                } else {
                    env.line.set_field(n, val);
                }
            }
            Variable(name) => {
                env.variables.insert(name.to_string(), val);
            }
        }
    }
}
