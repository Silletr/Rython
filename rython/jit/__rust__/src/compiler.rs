use crate::parser::{Expr, Program, Statement, Op};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Int(i64),
    Float(f64),
    Str(String),
}

impl RuntimeValue {
    pub fn to_u64(&self) -> u64 {
        match self {
            RuntimeValue::Int(i) => *i as u64,
            RuntimeValue::Float(f) => *f as u64,
            RuntimeValue::Str(_) => 0,
        }
    }
}

/// A simple Tree-Walk Interpreter for Rython.
/// This makes the language functional until we switch to LLVM JIT.
pub struct Interpreter {
    pub variables: HashMap<String, RuntimeValue>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            variables: HashMap::new(),
        }
    }

    pub fn run(&mut self, program: &Program) -> Option<RuntimeValue> {
        // Look for 'main' function first
        let main_func = program.body.iter().find_map(|stmt| {
            if let Statement::FunctionDef(f) = stmt {
                if f.name == "main" { return Some(f); }
            }
            None
        });

        if let Some(main) = main_func {
            for stmt in &main.body {
                if let Some(val) = self.eval_statement(stmt) {
                    return Some(val); // Return from main
                }
            }
        } else {
            // If no main, just execute top-level statements
            for stmt in &program.body {
                self.eval_statement(stmt);
            }
        }
        None
    }

    fn eval_statement(&mut self, stmt: &Statement) -> Option<RuntimeValue> {
        match stmt {
            Statement::VarDecl(decl) => {
                let val = self.eval_expr(&decl.value);
                self.variables.insert(decl.name.clone(), val);
                None
            }
            Statement::Return(expr) => Some(self.eval_expr(expr)),
            Statement::Expr(expr) => {
                self.eval_expr(expr);
                None
            }
            Statement::FunctionDef(_) => None, // Functions handled by lookup
        }
    }

    fn eval_expr(&self, expr: &Expr) -> RuntimeValue {
        match expr {
            Expr::Number(n) => RuntimeValue::Int(*n),
            Expr::Float(f) => RuntimeValue::Float(*f),
            Expr::String(s) => RuntimeValue::Str(s.clone()),
            Expr::Var(name) => self.variables.get(name).cloned().unwrap_or(RuntimeValue::Int(0)),
            Expr::BinOp { left, op, right } => {
                let l = self.eval_expr(left);
                let r = self.eval_expr(right);
                match (l, r) {
                    (RuntimeValue::Int(a), RuntimeValue::Int(b)) => match op {
                        Op::Add => RuntimeValue::Int(a + b),
                        Op::Sub => RuntimeValue::Int(a - b),
                        Op::Mul => RuntimeValue::Int(a * b),
                        Op::Div => RuntimeValue::Int(if b != 0 { a / b } else { 0 }),
                    },
                    (RuntimeValue::Float(a), RuntimeValue::Float(b)) => match op {
                        Op::Add => RuntimeValue::Float(a + b),
                        Op::Sub => RuntimeValue::Float(a - b),
                        Op::Mul => RuntimeValue::Float(a * b),
                        Op::Div => RuntimeValue::Float(a / b),
                    },
                    (RuntimeValue::Int(a), RuntimeValue::Float(b)) => match op {
                        Op::Add => RuntimeValue::Float(a as f64 + b),
                        Op::Sub => RuntimeValue::Float(a as f64 - b),
                        Op::Mul => RuntimeValue::Float(a as f64 * b),
                        Op::Div => RuntimeValue::Float(a as f64 / b),
                    },
                    (RuntimeValue::Float(a), RuntimeValue::Int(b)) => match op {
                        Op::Add => RuntimeValue::Float(a + b as f64),
                        Op::Sub => RuntimeValue::Float(a - b as f64),
                        Op::Mul => RuntimeValue::Float(a * b as f64),
                        Op::Div => RuntimeValue::Float(a / b as f64),
                    },
                    _ => RuntimeValue::Int(0),
                }
            }
            Expr::Call { func, args } => {
                // For now, very limited built-in or dummy calls
                println!("Calling function: {} with {} args", func, args.len());
                RuntimeValue::Int(0)
            }
        }
    }
}
