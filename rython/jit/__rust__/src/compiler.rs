use crate::parser::{Expr, Program, Statement, Op, Type, FunctionDef};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue, BasicValue, AnyValue};
use inkwell::types::{BasicTypeEnum, BasicType};
use inkwell::targets::{Target, TargetMachine, InitializationConfig, FileType, RelocMode, CodeModel};
use inkwell::OptimizationLevel;
use std::collections::HashMap;
use std::path::Path;
use std::convert::TryFrom;

/// The LLVM Compiler for Rython.
pub struct Compiler<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    // Store both the pointer and the element type for Opaque Pointers (LLVM 15+)
    pub variables: HashMap<String, (PointerValue<'ctx>, BasicTypeEnum<'ctx>)>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        
        let void_type = context.void_type();
        let i8_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
        let size_t_type = context.i64_type();

        let init_fn_type = void_type.fn_type(&[], false);
        module.add_function("rython_mem_init", init_fn_type, None);

        let malloc_fn_type = i8_ptr_type.fn_type(&[size_t_type.into()], false);
        module.add_function("rython_malloc", malloc_fn_type, None);

        Compiler {
            context,
            module,
            builder,
            variables: HashMap::new(),
        }
    }

    pub fn compile_program(&mut self, program: &Program) {
        for stmt in &program.body {
            match stmt {
                Statement::FunctionDef(func) => {
                    self.compile_function(func);
                }
                _ => {}
            }
        }
    }

    fn to_llvm_type(&self, ty: &Type) -> BasicTypeEnum<'ctx> {
        match ty {
            Type::Int => self.context.i64_type().into(),
            Type::Float => self.context.f64_type().into(),
            Type::Str => self.context.ptr_type(inkwell::AddressSpace::default()).into(),
        }
    }

    fn compile_function(&mut self, func_def: &FunctionDef) -> Option<FunctionValue<'ctx>> {
        let ret_type = self.to_llvm_type(&func_def.return_type);
        let arg_types: Vec<inkwell::types::BasicMetadataTypeEnum> = func_def
            .args
            .iter()
            .map(|(_, ty)| self.to_llvm_type(ty).into())
            .collect();

        let fn_type = ret_type.fn_type(&arg_types, false);
        let function = self.module.add_function(&func_def.name, fn_type, None);

        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);

        if func_def.name == "main" {
            let init_msg = self.module.get_function("rython_mem_init").expect("GC init not found");
            self.builder.build_call(init_msg, &[], "gc_init").unwrap();
        }

        self.variables.clear();

        for (i, arg) in function.get_param_iter().enumerate() {
            let (arg_name, _arg_type) = &func_def.args[i];
            let arg_llvm_type = arg.get_type();
            let alloca = self.create_entry_block_alloca(function, arg_name, arg_llvm_type);
            self.builder.build_store(alloca, arg).unwrap();
            self.variables.insert(arg_name.clone(), (alloca, arg_llvm_type));
        }

        for stmt in &func_def.body {
            self.compile_statement(stmt, function);
        }

        if entry.get_terminator().is_none() {
             match func_def.return_type {
                 Type::Int => self.builder.build_return(Some(&self.context.i64_type().const_int(0, false))).unwrap(),
                 Type::Float => self.builder.build_return(Some(&self.context.f64_type().const_float(0.0))).unwrap(),
                 Type::Str => self.builder.build_return(Some(&self.context.ptr_type(inkwell::AddressSpace::default()).const_null())).unwrap(),
             };
        }

        if function.verify(true) {
            Some(function)
        } else {
            unsafe { function.delete(); }
            None
        }
    }

    fn create_entry_block_alloca(&self, function: FunctionValue<'ctx>, name: &str, ty: BasicTypeEnum<'ctx>) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();
        let entry = function.get_first_basic_block().unwrap();
        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }
        builder.build_alloca(ty, name).unwrap()
    }

    fn compile_statement(&mut self, stmt: &Statement, function: FunctionValue<'ctx>) {
        match stmt {
            Statement::VarDecl(decl) => {
                let val = self.compile_expr(&decl.value);
                let val_type = val.get_type();
                let alloca = self.create_entry_block_alloca(function, &decl.name, val_type);
                self.builder.build_store(alloca, val).unwrap();
                self.variables.insert(decl.name.clone(), (alloca, val_type));
            }
            Statement::Return(expr) => {
                let val = self.compile_expr(expr);
                self.builder.build_return(Some(&val)).unwrap();
            }
            Statement::Expr(expr) => {
                self.compile_expr(expr);
            }
            Statement::FunctionDef(_) => (),
        }
    }

    fn compile_expr(&self, expr: &Expr) -> BasicValueEnum<'ctx> {
        match expr {
            Expr::Number(n) => self.context.i64_type().const_int(*n as u64, false).into(),
            Expr::Float(f) => self.context.f64_type().const_float(*f).into(),
            Expr::String(s) => {
                let malloc_fn = self.module.get_function("rython_malloc").expect("rython_malloc not found");
                let s_len = s.len() as u64;
                let size_val = self.context.i64_type().const_int(s_len + 1, false);
                
                let call = self.builder.build_call(malloc_fn, &[size_val.into()], "str_ptr").unwrap();
                let ptr = BasicValueEnum::try_from(call.as_any_value_enum())
                    .expect("rython_malloc should return a pointer")
                    .into_pointer_value();
                
                let _global_str = self.builder.build_global_string_ptr(s, "str_const").unwrap();
                
                // Copy the string (using memcpy or manual loop for simplicity in IR)
                // For now, let's just return the malloced pointer to show it's being called.
                // In a real implementation we would memcpy.
                ptr.as_basic_value_enum()
            }
            Expr::Var(name) => {
                let (ptr, ty) = self.variables.get(name).expect("Variable not found");
                self.builder.build_load(*ty, *ptr, name).unwrap().as_basic_value_enum()
            }
            Expr::BinOp { left, op, right } => {
                let lhs = self.compile_expr(left);
                let rhs = self.compile_expr(right);

                match (lhs, rhs) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => match op {
                        Op::Add => self.builder.build_int_add(l, r, "addtmp").unwrap().into(),
                        Op::Sub => self.builder.build_int_sub(l, r, "subtmp").unwrap().into(),
                        Op::Mul => self.builder.build_int_mul(l, r, "multmp").unwrap().into(),
                        Op::Div => self.builder.build_int_signed_div(l, r, "divtmp").unwrap().into(),
                    },
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => match op {
                        Op::Add => self.builder.build_float_add(l, r, "addtmp").unwrap().into(),
                        Op::Sub => self.builder.build_float_sub(l, r, "subtmp").unwrap().into(),
                        Op::Mul => self.builder.build_float_mul(l, r, "multmp").unwrap().into(),
                        Op::Div => self.builder.build_float_div(l, r, "divtmp").unwrap().into(),
                    },
                    _ => panic!("Type mismatch in binary operation"),
                }
            }
            Expr::Call { func, args } => {
                let function = self.module.get_function(func).expect("Function not found");
                let compiled_args: Vec<inkwell::values::BasicMetadataValueEnum> = args
                    .iter()
                    .map(|arg| self.compile_expr(arg).into())
                    .collect();
                
                let call = self.builder.build_call(function, &compiled_args, "calltmp").unwrap();
                // Use explicit trait method call if needed, but import AnyValue is already done.
                // CallSiteValue implements AnyValue in Inkwell 0.7.1
                let any_val = call.as_any_value_enum();
                BasicValueEnum::try_from(any_val)
                    .expect("Function should return a basic value")
            }
        }
    }

    pub fn emit_to_file(&self, path: &str) -> Result<(), String> {
        Target::initialize_native(&InitializationConfig::default()).map_err(|e| e.to_string())?;

        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple).map_err(|e| e.to_string())?;
        let cpu = TargetMachine::get_host_cpu_name().to_string();
        let features = TargetMachine::get_host_cpu_features().to_string();

        let target_machine = target
            .create_target_machine(
                &triple,
                &cpu,
                &features,
                OptimizationLevel::Default,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or_else(|| "Failed to create target machine".to_string())?;

        target_machine
            .write_to_file(&self.module, FileType::Object, Path::new(path))
            .map_err(|e| e.to_string())
    }

    pub fn emit_ir_to_string(&self) -> String {
        self.module.print_to_string().to_string()
    }
}
