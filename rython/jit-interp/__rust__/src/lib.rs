use pyo3::prelude::*;
use std::collections::HashMap;
use std::rc::Rc;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use std::fs;

#[path = "interp/arena.rs"] mod arena;
#[path = "interp/bytecode.rs"] mod bytecode;
#[path = "interp/frontend.rs"] mod frontend;
#[path = "interp/ir.rs"] mod ir;
#[path = "interp/lower.rs"] mod lower;
#[path = "interp/value.rs"] mod value;
#[path = "interp/vm.rs"] mod vm;

mod parser;

use frontend::{CodeGen, Func as FrontendFunc, Stmt as FrontendStmt, Expr as FrontendExpr, BinOp as FrontendBinOp};
use lower::LoweringContext;
use vm::VM;

static mut JIT_THRESHOLD: usize = 1000000; // Not used, kept for compatibility

static COMPILATION_CACHE: Lazy<Mutex<HashMap<String, (Vec<(Vec<bytecode::Instruction>, usize, usize)>, Vec<String>)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn convert_expr(expr: parser::Expr) -> FrontendExpr {
    match expr {
        parser::Expr::Number(n) => FrontendExpr::Number(n),
        parser::Expr::Float(f) => FrontendExpr::Number(f as i64),
        parser::Expr::String(s) => FrontendExpr::String(s),
        parser::Expr::Var(name) => FrontendExpr::Variable(name),
        parser::Expr::BinOp { left, op, right } => {
            let left = convert_expr(*left);
            let right = convert_expr(*right);
            let binop = match op {
                parser::Op::Add => FrontendBinOp::Add,
                parser::Op::Sub => FrontendBinOp::Sub,
                parser::Op::Mul => FrontendBinOp::Mul,
                parser::Op::Div => FrontendBinOp::Div,
                parser::Op::Lt => FrontendBinOp::Lt,
                parser::Op::Le => FrontendBinOp::Le,
            };
            FrontendExpr::Binary(binop, Box::new(left), Box::new(right))
        }
        parser::Expr::Call { func, args } => {
            let args = args.into_iter().map(convert_expr).collect();
            FrontendExpr::Call(func, args)
        }
    }
}

fn convert_statement(stmt: parser::Statement) -> FrontendStmt {
    match stmt {
        parser::Statement::VarDecl(decl) => FrontendStmt::Let(decl.name, convert_expr(decl.value)),
        parser::Statement::Return(expr) => FrontendStmt::Return(convert_expr(expr)),
        parser::Statement::Expr(expr) => {
            match expr {
                parser::Expr::Call { func, args } if func == "print" => {
                    if args.len() == 1 {
                        FrontendStmt::Print(convert_expr(args.into_iter().next().unwrap()))
                    } else {
                        panic!("print expects exactly one argument");
                    }
                }
                _ => FrontendStmt::Expr(convert_expr(expr)),
            }
        }
        parser::Statement::While(while_stmt) => {
            let cond = convert_expr(while_stmt.condition);
            let body = while_stmt.body.into_iter().map(convert_statement).collect();
            FrontendStmt::While(cond, Box::new(FrontendStmt::Block(body)))
        }
        parser::Statement::If(if_stmt) => {
            let cond = convert_expr(if_stmt.condition);
            let then_block = if_stmt.then_block.into_iter().map(convert_statement).collect();
            let then_stmt = Box::new(FrontendStmt::Block(then_block));
            let else_stmt = if_stmt.else_block.map(|block| {
                let else_block = block.into_iter().map(convert_statement).collect();
                Box::new(FrontendStmt::Block(else_block))
            });
            FrontendStmt::If(cond, then_stmt, else_stmt)
        }
        parser::Statement::Assign(name, expr) => {
            FrontendStmt::Expr(FrontendExpr::Assign(name, Box::new(convert_expr(expr))))
        }
        parser::Statement::FunctionDef(_) => panic!("Function def not expected here"),
        parser::Statement::Import(_) => panic!("Import should have been resolved earlier"),
    }
}

fn convert_function_def(f: parser::FunctionDef) -> FrontendFunc {
    let params = f.args.into_iter().map(|(name, _ty)| name).collect();
    let body = f.body.into_iter().map(convert_statement).collect();
    FrontendFunc { name: f.name, params, body }
}

fn collect_functions_from_file(
    filename: &str,
    already_loaded: &mut HashSet<String>,
) -> Result<Vec<FrontendFunc>, String> {
    let canonical = Path::new(filename)
        .canonicalize()
        .map_err(|e| format!("Cannot canonicalize {}: {}", filename, e))?;
    let canonical_str = canonical.to_string_lossy().to_string();
    if already_loaded.contains(&canonical_str) {
        return Ok(Vec::new());
    }
    already_loaded.insert(canonical_str);

    let code = fs::read_to_string(filename)
        .map_err(|e| format!("Cannot read {}: {}", filename, e))?;
    let program = parser::parse_rython_code(&code)
        .map_err(|e| format!("Parse error in {}: {}", filename, e))?;

    let dir = Path::new(filename).parent().unwrap_or(Path::new(".")).to_path_buf();
    let mut functions = Vec::new();
    let mut imports = Vec::new();

    for stmt in program.body {
        match stmt {
            parser::Statement::FunctionDef(f) => {
                functions.push(convert_function_def(f));
            }
            parser::Statement::Import(import_path) => {
                let full_path = dir.join(&import_path);
                imports.push(full_path.to_string_lossy().to_string());
            }
            _ => {}
        }
    }

    for import_path in imports {
        let imported_funcs = collect_functions_from_file(&import_path, already_loaded)?;
        functions.extend(imported_funcs);
    }

    Ok(functions)
}

pub fn compile_file(filename: &str) -> Result<String, String> {
    let mut already_loaded = HashSet::new();
    let functions = collect_functions_from_file(filename, &mut already_loaded)?;

    let code = fs::read_to_string(filename)
        .map_err(|e| format!("Cannot read {}: {}", filename, e))?;
    let program = parser::parse_rython_code(&code)
        .map_err(|e| format!("Parse error in {}: {}", filename, e))?;

    let mut main_statements = Vec::new();
    for stmt in program.body {
        match stmt {
            parser::Statement::FunctionDef(_) | parser::Statement::Import(_) => {}
            other => main_statements.push(convert_statement(other)),
        }
    }

    let mut codegen = CodeGen::new();
    let ir_program = codegen.generate(functions, main_statements);
    let mut lower_ctx = LoweringContext::new();
    let (functions, string_pool) = lower_ctx.lower_program(&ir_program);

    // Use zero‑allocation interpreter with max call depth 1024
    let mut vm = VM::new(functions, string_pool, 1024 * 1024, unsafe { JIT_THRESHOLD }, 1024);
    match vm.run() {
        Ok(val) => Ok(format!("{:?}", val)),
        Err(e) => Err(e),
    }
}

#[pyfunction]
fn compile_to_native(py: Python, code: &str) -> PyResult<String> {
    py.allow_threads(|| {
        let program = parser::parse_rython_code(code)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Parse error: {}", e)))?;
        let mut funcs = Vec::new();
        let mut main_stmts = Vec::new();
        for stmt in program.body {
            match stmt {
                parser::Statement::FunctionDef(f) => funcs.push(convert_function_def(f)),
                parser::Statement::Import(_) => {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "import not supported in compile_to_native; use compile_file_py instead".to_string()
                    ));
                }
                other => main_stmts.push(convert_statement(other)),
            }
        }
        let mut codegen = CodeGen::new();
        let ir_program = codegen.generate(funcs, main_stmts);
        let mut lower_ctx = LoweringContext::new();
        let (functions, string_pool) = lower_ctx.lower_program(&ir_program);
        let mut vm = VM::new(functions, string_pool, 1024 * 1024, unsafe { JIT_THRESHOLD }, 1024);
        match vm.run() {
            Ok(val) => Ok(format!("{:?}", val)),
            Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e)),
        }
    })
}

#[pyfunction]
fn compile_file_py(py: Python, filename: &str) -> PyResult<String> {
    py.allow_threads(|| {
        compile_file(filename).map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
    })
}

#[pyfunction]
fn compile_many(py: Python, codes: Vec<String>) -> PyResult<Vec<String>> {
    let results: Vec<Result<String, String>> = py.allow_threads(|| {
        codes.into_par_iter().map(|code| {
            let program = parser::parse_rython_code(&code)
                .map_err(|e| format!("Parse error: {}", e))?;
            let mut funcs = Vec::new();
            let mut main_stmts = Vec::new();
            for stmt in program.body {
                match stmt {
                    parser::Statement::FunctionDef(f) => funcs.push(convert_function_def(f)),
                    parser::Statement::Import(_) => {
                        return Err("import not supported in compile_many".to_string());
                    }
                    other => main_stmts.push(convert_statement(other)),
                }
            }
            let mut codegen = CodeGen::new();
            let ir_program = codegen.generate(funcs, main_stmts);
            let mut lower_ctx = LoweringContext::new();
            let (functions, string_pool) = lower_ctx.lower_program(&ir_program);
            let mut vm = VM::new(functions, string_pool, 1024 * 1024, unsafe { JIT_THRESHOLD }, 1024);
            vm.run().map(|val| format!("{:?}", val))
        }).collect()
    });
    results.into_iter().collect::<Result<Vec<String>, String>>()
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e))
}

#[pyfunction]
fn set_jit_threshold(threshold: usize) {
    unsafe { JIT_THRESHOLD = threshold; }
}

#[pyfunction]
fn set_thread_count(n: usize) {
    rayon::ThreadPoolBuilder::new()
        .num_threads(n)
        .build_global()
        .unwrap();
}

#[pymodule]
fn rython(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compile_to_native, m)?)?;
    m.add_function(wrap_pyfunction!(compile_file_py, m)?)?;
    m.add_function(wrap_pyfunction!(compile_many, m)?)?;
    m.add_function(wrap_pyfunction!(set_jit_threshold, m)?)?;
    m.add_function(wrap_pyfunction!(set_thread_count, m)?)?;
    Ok(())
}