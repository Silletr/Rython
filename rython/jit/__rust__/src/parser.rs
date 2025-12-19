#![allow(dead_code)]

use pest_derive::Parser;
use pest_consume::{match_nodes, Error, Parser as PestConsumer};

#[derive(Debug, Clone)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Str,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i64),
    Float(f64),
    String(String),
    Var(String),
    BinOp {
        left: Box<Expr>,
        op: Op,
        right: Box<Expr>,
    },
    Call {
        func: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: String,
    pub type_def: Type,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub args: Vec<(String, Type)>,
    pub return_type: Type,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    VarDecl(VarDecl),
    FunctionDef(FunctionDef),
    Return(Expr),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub struct Program {
    pub body: Vec<Statement>,
}

#[derive(Parser)]
#[grammar = "rython.pest"]
pub struct RythonParser;

type Result<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

#[pest_consume::parser]
impl RythonParser {
    fn EOI(_node: Node) -> Result<()> {
        Ok(())
    }

    fn identifier(node: Node) -> Result<String> {
        Ok(node.as_str().to_string())
    }

    fn number(node: Node) -> Result<Expr> {
        Ok(Expr::Number(node.as_str().parse().map_err(|e| node.error(e))?))
    }

    fn float(node: Node) -> Result<Expr> {
        Ok(Expr::Float(node.as_str().parse().map_err(|e| node.error(e))?))
    }

    fn string(node: Node) -> Result<Expr> {
        let s = node.as_str();
        Ok(Expr::String(s[1..s.len()-1].to_string()))
    }

    fn r#type(node: Node) -> Result<Type> {
        match node.as_str() {
            "int" => Ok(Type::Int),
            "float" => Ok(Type::Float),
            "str" => Ok(Type::Str),
            _ => unreachable!(),
        }
    }

    fn var_ref(node: Node) -> Result<Expr> {
        Ok(Expr::Var(node.as_str().to_string()))
    }
    
    fn call(node: Node) -> Result<Expr> {
        match_nodes!(node.into_children();
            [identifier(func), expression(args)..] => {
                Ok(Expr::Call { func, args: args.collect() })
            },
        )
    }

    fn term(node: Node) -> Result<Expr> {
        match_nodes!(node.into_children();
            [number(n)] => Ok(n),
            [float(f)] => Ok(f),
            [string(s)] => Ok(s),
            [call(c)] => Ok(c),
            [var_ref(v)] => Ok(v),
            [expression(e)] => Ok(e),
        )
    }

    fn expression(node: Node) -> Result<Expr> {
        let span = node.as_span();
        let mut children = node.into_children();
        let first = children.next().ok_or_else(|| {
             Error::new_from_span(pest::error::ErrorVariant::CustomError { message: "Empty expression".to_string() }, span)
        })?;
        let mut res = Self::term(first)?;

        while let Some(op_node) = children.next() {
            let op_str = op_node.as_str();
            let op = match op_str {
                "+" => Op::Add,
                "-" => Op::Sub,
                "*" => Op::Mul,
                "/" => Op::Div,
                _ => unreachable!(),
            };
            let next_term = children.next().ok_or_else(|| {
                 Error::new_from_span(pest::error::ErrorVariant::CustomError { message: "Expected term after operator".to_string() }, span)
            })?;
            let right = Self::term(next_term)?;
            res = Expr::BinOp {
                left: Box::new(res),
                op,
                right: Box::new(right),
            };
        }
        Ok(res)
    }

    fn var_decl(node: Node) -> Result<Statement> {
        match_nodes!(node.into_children();
            [identifier(name), r#type(type_def), expression(value)] => {
                Ok(Statement::VarDecl(VarDecl {
                    name,
                    type_def,
                    value,
                }))
            },
        )
    }
    
    fn return_statement(node: Node) -> Result<Statement> {
        match_nodes!(node.into_children();
            [expression(expr)] => Ok(Statement::Return(expr)),
        )
    }
    
    fn param(node: Node) -> Result<(String, Type)> {
        match_nodes!(node.into_children();
            [identifier(name), r#type(type_def)] => Ok((name, type_def)),
        )
    }

    fn params(node: Node) -> Result<Vec<(String, Type)>> {
        match_nodes!(node.into_children();
            [param(p)..] => Ok(p.collect()),
        )
    }

    fn block(node: Node) -> Result<Vec<Statement>> {
        match_nodes!(node.into_children();
            [statement(s)..] => Ok(s.collect()),
        )
    }

    fn function_def(node: Node) -> Result<Statement> {
        match_nodes!(node.into_children();
            [
                identifier(name),
                params(args),
                r#type(return_type),
                block(body)
            ] => {
                Ok(Statement::FunctionDef(FunctionDef {
                    name,
                    args,
                    return_type,
                    body,
                }))
            },
        )
    }

    fn statement(node: Node) -> Result<Statement> {
        match_nodes!(node.into_children();
            [var_decl(decl)] => Ok(decl),
            [function_def(func)] => Ok(func),
            [return_statement(ret)] => Ok(ret),
            [expression(expr)] => Ok(Statement::Expr(expr)),
        )
    }

    fn program(node: Node) -> Result<Program> {
        match_nodes!(node.into_children();
            [statement(stmts).., EOI(_)] => Ok(Program { body: stmts.collect() }),
        )
    }
}

pub fn parse_rython_code(code: &str) -> Result<Program> {
    let nodes = <RythonParser as PestConsumer>::parse(Rule::program, code)
        .map_err(|e: pest::error::Error<Rule>| {
            Error::new_from_span(pest::error::ErrorVariant::CustomError { message: e.to_string() }, pest::Span::new(code, 0, code.len()).unwrap())
        })?;
    let program_node = nodes.single().map_err(|e: Error<Rule>| e)?;
    RythonParser::program(program_node)
}
