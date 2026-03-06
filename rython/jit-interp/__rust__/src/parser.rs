use pest_derive::Parser;
use pest_consume::{Error, Parser as PestConsumer};
use std::result;

#[derive(Parser)]
#[grammar = "rython.pest"]
pub struct RythonParser;

type Result<T> = result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

#[pest_consume::parser]
impl RythonParser {
    fn EOI(_input: Node) -> Result<()> { Ok(()) }

    fn identifier(node: Node) -> Result<String> { Ok(node.as_str().to_string()) }

    fn number(node: Node) -> Result<i64> {
        node.as_str().parse::<i64>().map_err(|e| node.error(e))
    }

    fn float(node: Node) -> Result<f64> {
        node.as_str().parse::<f64>().map_err(|e| node.error(e))
    }

    fn string(node: Node) -> Result<String> {
        let s = node.as_str();
        Ok(s[1..s.len()-1].to_string())
    }

    fn r#type(node: Node) -> Result<String> { Ok(node.as_str().to_string()) }

    fn op(node: Node) -> Result<Op> {
        match node.as_str() {
            "+" => Ok(Op::Add),
            "-" => Ok(Op::Sub),
            "*" => Ok(Op::Mul),
            "/" => Ok(Op::Div),
            "<" => Ok(Op::Lt),
            "<=" => Ok(Op::Le),
            other => panic!("Invalid operator: {}", other),
        }
    }

    fn param(node: Node) -> Result<(String, String)> {
        let mut children = node.into_children();
        let name_node = children.next().unwrap();
        let type_node = children.next().unwrap();
        let name = Self::identifier(name_node)?;
        let ty = Self::r#type(type_node)?;
        Ok((name, ty))
    }

    fn var_ref(node: Node) -> Result<Expr> { Ok(Expr::Var(node.as_str().to_string())) }

    fn call(node: Node) -> Result<Expr> {
        let mut children = node.into_children();
        let name_node = children.next().unwrap();
        let name = Self::identifier(name_node)?;
        let mut args = Vec::new();
        for child in children {
            if child.as_rule() == Rule::expression {
                args.push(Self::expression(child)?);
            }
        }
        Ok(Expr::Call { func: name, args })
    }

    fn term(node: Node) -> Result<Expr> {
        let child = node.into_children().next().unwrap();
        match child.as_rule() {
            Rule::number => Ok(Expr::Number(Self::number(child)?)),
            Rule::float => Ok(Expr::Float(Self::float(child)?)),
            Rule::string => Ok(Expr::String(Self::string(child)?)),
            Rule::call => Self::call(child),
            Rule::var_ref => Self::var_ref(child),
            Rule::expression => Self::expression(child),
            _ => unreachable!(),
        }
    }

    fn expression(node: Node) -> Result<Expr> {
        let mut children = node.into_children();
        let mut res = Self::term(children.next().unwrap())?;
        while let Some(op_node) = children.next() {
            let op = Self::op(op_node)?;
            let right = Self::term(children.next().unwrap())?;
            res = Expr::BinOp {
                left: Box::new(res),
                op,
                right: Box::new(right),
            };
        }
        Ok(res)
    }

    fn block(node: Node) -> Result<Vec<Statement>> {
        let mut stmts = Vec::new();
        for child in node.into_children() {
            if child.as_rule() == Rule::statement {
                stmts.push(Self::statement(child)?);
            }
        }
        Ok(stmts)
    }

    fn while_statement(node: Node) -> Result<Statement> {
        let mut children = node.into_children();
        let cond_node = children.next().unwrap();
        let cond = Self::expression(cond_node)?;
        let block_node = children.next().unwrap();
        let body = Self::block(block_node)?;
        Ok(Statement::While(WhileStmt { condition: cond, body }))
    }

    fn if_statement(node: Node) -> Result<Statement> {
        let mut children = node.into_children();
        let cond_node = children.next().unwrap();
        let cond = Self::expression(cond_node)?;
        let then_block_node = children.next().unwrap();
        let then_block = Self::block(then_block_node)?;
        let else_block = if let Some(else_node) = children.next() {
            match else_node.as_rule() {
                Rule::block => Some(Self::block(else_node)?),
                Rule::if_statement => Some(vec![Self::if_statement(else_node)?]),
                _ => unreachable!(),
            }
        } else {
            None
        };
        Ok(Statement::If(IfStmt { condition: cond, then_block, else_block }))
    }

    fn assign_statement(node: Node) -> Result<Statement> {
        let mut children = node.into_children();
        let name_node = children.next().unwrap();
        let val_node = children.next().unwrap();
        let name = Self::identifier(name_node)?;
        let val = Self::expression(val_node)?;
        Ok(Statement::Assign(name, val))
    }

    fn var_decl(node: Node) -> Result<Statement> {
        let mut children = node.into_children();
        let name_node = children.next().unwrap();
        let type_node = children.next().unwrap();
        let val_node = children.next().unwrap();
        let name = Self::identifier(name_node)?;
        let ty = Self::r#type(type_node)?;
        let val = Self::expression(val_node)?;
        Ok(Statement::VarDecl(VarDecl { name, type_def: ty, value: val }))
    }

    fn return_statement(node: Node) -> Result<Statement> {
        let mut children = node.into_children();
        let expr_node = children.next().unwrap();
        let expr = Self::expression(expr_node)?;
        Ok(Statement::Return(expr))
    }

    fn function_def(node: Node) -> Result<Statement> {
        let mut children = node.into_children();
        let name_node = children.next().unwrap();
        let name = Self::identifier(name_node)?;
        let mut args = Vec::new();
        let mut return_type = None;
        let mut block_node = None;
        for child in children {
            match child.as_rule() {
                Rule::param => args.push(Self::param(child)?),
                Rule::r#type => return_type = Some(Self::r#type(child)?),
                Rule::block => block_node = Some(child),
                _ => {}
            }
        }
        let return_type = return_type.expect("Missing return type");
        let block_node = block_node.expect("Missing function body");
        let body = Self::block(block_node)?;
        Ok(Statement::FunctionDef(FunctionDef { name, args, return_type, body }))
    }

    fn statement(node: Node) -> Result<Statement> {
        let child = node.into_children().next().unwrap();
        match child.as_rule() {
            Rule::var_decl => Self::var_decl(child),
            Rule::function_def => Self::function_def(child),
            Rule::return_statement => Self::return_statement(child),
            Rule::if_statement => Self::if_statement(child),
            Rule::while_statement => Self::while_statement(child),
            Rule::assign_statement => Self::assign_statement(child),
            Rule::expression => Ok(Statement::Expr(Self::expression(child)?)),
            _ => unreachable!(),
        }
    }

    fn program(node: Node) -> Result<Program> {
        let mut body = Vec::new();
        for child in node.into_children() {
            match child.as_rule() {
                Rule::statement => body.push(Self::statement(child)?),
                Rule::EOI => break,
                _ => {}
            }
        }
        Ok(Program { body })
    }
}

#[derive(Debug, Clone)]
pub enum Op {
    Add, Sub, Mul, Div, Lt, Le,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i64),
    Float(f64),
    String(String),
    Var(String),
    BinOp { left: Box<Expr>, op: Op, right: Box<Expr> },
    Call { func: String, args: Vec<Expr> },
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: String,
    pub type_def: String,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_block: Vec<Statement>,
    pub else_block: Option<Vec<Statement>>,
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub args: Vec<(String, String)>,
    pub return_type: String,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    VarDecl(VarDecl),
    FunctionDef(FunctionDef),
    Return(Expr),
    While(WhileStmt),
    If(IfStmt),
    Assign(String, Expr),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub struct Program {
    pub body: Vec<Statement>,
}

pub fn parse_rython_code(code: &str) -> Result<Program> {
    let nodes = RythonParser::parse(Rule::program, code)?;
    let program_node = nodes.single()?;
    RythonParser::program(program_node)
}