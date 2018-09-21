use std::collections::HashMap;
use std::io::BufRead;

use ast::{self, Expr, Pattern};

#[derive(Clone)]
enum Value {
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
    fn to_string(self) -> String {
        match self {
            Value::Number(n) => format!("{}", n),
            Value::String(s) => s,
        }
    }
}

#[derive(Default)]
pub struct Environment {
    fields: Vec<Value>,
    variables: HashMap<String, Value>,
}
pub type Program = Vec<(ast::Pattern, ast::Action)>;

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
            self.fields = record
                .split_whitespace()
                .map(|s| Value::String(s.to_string()))
                .collect();

            for (pattern, action) in p {
                match pattern {
                    Pattern::Always => action.run(self),
                    Pattern::Expr(e) => if e.eval(self).truthy() {
                        action.run(self);
                    },
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
impl ast::Expr {
    fn eval(&self, env: &mut Environment) -> Value {
        use ast::{BinaryOp, UnaryOp};

        match self {
            Expr::Num(n) => Value::Number(*n),
            Expr::String(s) => Value::String(s.clone()),
            Expr::Lvalue(lv) => lv.eval(env),

            Expr::Unary(..) => unimplemented!(),
            Expr::Binary(..) => unimplemented!(),
            Expr::Assignment(lhs, rhs) => {
                let value = rhs.eval(env);
                lhs.assign(env, value.clone());
                value
            }

            Expr::Conditional(cond, if_true, if_false) => if cond.eval(env).truthy() {
                if_true.eval(env)
            } else {
                if_false.eval(env)
            },
        }
    }
}

impl ast::Lvalue {
    fn eval(&self, env: &mut Environment) -> Value {
        use ast::Lvalue::*;

        match self {
            Field(n) => {
                let n = n.eval(env).as_num() as usize;
                if n == 0 {
                    let string_fields: Vec<String> =
                        env.fields.iter().cloned().map(Value::to_string).collect();
                    Value::String(string_fields.join(" "))
                } else {
                    match env.fields.get(n - 1) {
                        Some(val) => val.clone(),
                        None => Value::String(String::new()),
                    }
                }
            }
            Variable(name) => match env.variables.get(name) {
                Some(val) => val.clone(),
                None => Value::String(String::new()),
            },
        }
    }
    fn assign(&self, env: &mut Environment, val: Value) {
        use ast::Lvalue::*;

        match self {
            Field(n) => {
                let n = n.eval(env).as_num() as usize;
                if n == 0 {
                    env.fields = val
                        .to_string()
                        .split_whitespace()
                        .map(|s| Value::String(s.to_string()))
                        .collect();
                } else {
                    unimplemented!()
                }
            }
            Variable(name) => {
                env.variables.insert(name.to_string(), val);
            }
        }
    }
}
