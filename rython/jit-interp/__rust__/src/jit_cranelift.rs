use crate::arena::Arena;
use crate::bytecode::{Instruction, OpCode};
use crate::value::Value;
use cranelift::prelude::*;
use cranelift::codegen::ir::{types, AbiParam, InstBuilder};
use cranelift::codegen::binemit::NullTrapSink;
use cranelift::codegen::isa::CallConv;
use cranelift::frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{Linkage, Module, FuncId};
use cranelift_simplejit::{SimpleJITBuilder, SimpleJITModule};
use std::collections::{BTreeSet, HashMap};
use std::mem;

pub type NativeFn = unsafe extern "C" fn(*mut Arena, *const Value, usize) -> Value;

pub struct CraneliftJIT {
    module: SimpleJITModule,
    func_ptrs: HashMap<usize, NativeFn>,
    signature: Signature,
    externs: ExternalSymbols,
}

struct ExternalSymbols {
    load_string: FuncId,
    print: FuncId,
    call_function: FuncId,
}

impl CraneliftJIT {
    pub fn new() -> Self {
        let mut builder = SimpleJITBuilder::new(cranelift_module::default_libcall_names());
        builder.symbol("ry_load_string", ry_load_string as *const u8);
        builder.symbol("ry_print", ry_print as *const u8);
        builder.symbol("ry_call_function", ry_call_function as *const u8);

        let mut module = SimpleJITModule::new(builder);

        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(types::I64));
        sig.params.push(AbiParam::new(types::I64));
        sig.params.push(AbiParam::new(types::I64));
        sig.returns.push(AbiParam::new(types::I64));

        let mut load_sig = Signature::new(CallConv::SystemV);
        load_sig.params.push(AbiParam::new(types::I64));
        load_sig.params.push(AbiParam::new(types::I64));
        load_sig.returns.push(AbiParam::new(types::I64));
        let load_string = module
            .declare_function("ry_load_string", Linkage::Import, &load_sig)
            .unwrap();

        let mut print_sig = Signature::new(CallConv::SystemV);
        print_sig.params.push(AbiParam::new(types::I64));
        print_sig.returns.push(AbiParam::new(types::I32));
        let print = module
            .declare_function("ry_print", Linkage::Import, &print_sig)
            .unwrap();

        let mut call_sig = Signature::new(CallConv::SystemV);
        call_sig.params.push(AbiParam::new(types::I64));
        call_sig.params.push(AbiParam::new(types::I64));
        call_sig.params.push(AbiParam::new(types::I64));
        call_sig.params.push(AbiParam::new(types::I64));
        call_sig.returns.push(AbiParam::new(types::I64));
        let call_function = module
            .declare_function("ry_call_function", Linkage::Import, &call_sig)
            .unwrap();

        CraneliftJIT {
            module,
            func_ptrs: HashMap::new(),
            signature: sig,
            externs: ExternalSymbols { load_string, print, call_function },
        }
    }

    pub fn compile(&mut self, func_idx: usize, bytecode: &[Instruction], num_params: usize, num_regs: usize) -> NativeFn {
        if let Some(&ptr) = self.func_ptrs.get(&func_idx) {
            return ptr;
        }

        let mut ctx = self.module.make_context();
        ctx.func.signature = self.signature.clone();

        let mut func_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);

        let entry_block = builder.create_block();
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        let arena_ptr = builder.append_block_param(entry_block, types::I64);
        let args_ptr = builder.append_block_param(entry_block, types::I64);
        let _num_args = builder.append_block_param(entry_block, types::I64);

        let mut reg_vars = Vec::with_capacity(num_regs);
        for i in 0..num_regs {
            let var = Variable::new(i);
            builder.declare_var(var, types::I64);
            reg_vars.push(var);
        }

        for i in 0..num_params {
            let offset = (i * mem::size_of::<Value>()) as i32;
            let arg_val = builder.ins().load(types::I64, MemFlags::new(), args_ptr, offset);
            builder.def_var(reg_vars[i], arg_val);
        }

        // Basic block discovery
        let mut block_starts = BTreeSet::new();
        block_starts.insert(0);
        for insn in bytecode {
            match insn.opcode() {
                OpCode::Jump | OpCode::Branch => {
                    block_starts.insert(insn.imm() as usize);
                }
                _ => {}
            }
        }
        let mut next_is_start = false;
        for (idx, insn) in bytecode.iter().enumerate() {
            if next_is_start { block_starts.insert(idx); }
            match insn.opcode() {
                OpCode::Jump | OpCode::Branch | OpCode::Return => next_is_start = true,
                _ => next_is_start = false,
            }
        }

        let mut block_map = HashMap::new();
        for &start in &block_starts {
            let block = builder.create_block();
            block_map.insert(start, block);
        }

        let mut current_block = entry_block;
        builder.switch_to_block(current_block);

        let mut i = 0;
        while i < bytecode.len() {
            if let Some(&block) = block_map.get(&i) {
                if block != current_block {
                    current_block = block;
                    builder.switch_to_block(current_block);
                }
            }

            let insn = bytecode[i];
            match insn.opcode() {
                OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div | OpCode::Lt | OpCode::Le => {
                    let dst = insn.dst() as usize;
                    let src1 = insn.src1() as usize;
                    let src2 = insn.src2() as usize;
                    let a = builder.use_var(reg_vars[src1]);
                    let b = builder.use_var(reg_vars[src2]);
                    let result = match insn.opcode() {
                        OpCode::Add => builder.ins().iadd(a, b),
                        OpCode::Sub => builder.ins().isub(a, b),
                        OpCode::Mul => builder.ins().imul(a, b),
                        OpCode::Div => builder.ins().sdiv(a, b),
                        OpCode::Lt => {
                            let cmp = builder.ins().icmp(IntCC::SignedLessThan, a, b);
                            builder.ins().bint(types::I64, cmp)
                        }
                        OpCode::Le => {
                            let cmp = builder.ins().icmp(IntCC::SignedLessThanOrEqual, a, b);
                            builder.ins().bint(types::I64, cmp)
                        }
                        _ => unreachable!(),
                    };
                    builder.def_var(reg_vars[dst], result);
                }
                OpCode::LoadConst => {
                    let dst = insn.dst() as usize;
                    let imm = insn.imm() as i64;
                    let val = builder.ins().iconst(types::I64, imm);
                    builder.def_var(reg_vars[dst], val);
                }
                OpCode::LoadBool => {
                    let dst = insn.dst() as usize;
                    let b = insn.imm() != 0;
                    let val = builder.ins().iconst(types::I64, b as i64);
                    builder.def_var(reg_vars[dst], val);
                }
                OpCode::LoadNil => {
                    let dst = insn.dst() as usize;
                    let val = builder.ins().iconst(types::I64, 0);
                    builder.def_var(reg_vars[dst], val);
                }
                OpCode::LoadString => {
                    let dst = insn.dst() as usize;
                    let idx = insn.imm() as i64;
                    let idx_val = builder.ins().iconst(types::I64, idx);
                    let load_fn = self.module.declare_func_in_func(self.externs.load_string, &mut builder.func);
                    let call = builder.ins().call(load_fn, &[arena_ptr, idx_val]);
                    let result = builder.inst_results(call)[0];
                    builder.def_var(reg_vars[dst], result);
                }
                OpCode::Move => {
                    let dst = insn.dst() as usize;
                    let src = insn.src1() as usize;
                    let val = builder.use_var(reg_vars[src]);
                    builder.def_var(reg_vars[dst], val);
                }
                OpCode::Jump => {
                    let target = insn.imm() as usize;
                    let target_block = *block_map.get(&target).unwrap();
                    builder.ins().jump(target_block, &[]);
                    builder.seal_block(current_block);
                    i = target;
                    continue;
                }
                OpCode::Branch => {
                    let cond_reg = insn.dst() as usize;
                    let target = insn.imm() as usize;
                    let cond_val = builder.use_var(reg_vars[cond_reg]);
                    let zero = builder.ins().iconst(types::I64, 0);
                    let cmp = builder.ins().icmp(IntCC::NotEqual, cond_val, zero);
                    let target_block = *block_map.get(&target).unwrap();
                    let fallthrough_block = builder.create_block();
                    builder.ins().brnz(cmp, target_block, &[]);
                    builder.seal_block(current_block);
                    current_block = fallthrough_block;
                    builder.switch_to_block(current_block);
                }
                OpCode::Return => {
                    let val_reg = insn.dst() as usize;
                    let ret_val = builder.use_var(reg_vars[val_reg]);
                    builder.ins().return_(&[ret_val]);
                    builder.seal_block(current_block);
                }
                OpCode::Print => {
                    let val_reg = insn.dst() as usize;
                    let val = builder.use_var(reg_vars[val_reg]);
                    let print_fn = self.module.declare_func_in_func(self.externs.print, &mut builder.func);
                    builder.ins().call(print_fn, &[val]);
                }
                OpCode::Call => {
                    unimplemented!("JIT for Call not implemented");
                }
            }
            i += 1;
        }

        builder.seal_all_blocks();
        builder.finalize();

        let name = format!("func_{}", func_idx);
        let id = self.module
            .declare_function(&name, Linkage::Export, &ctx.func.signature)
            .unwrap();
        self.module
            .define_function(id, &mut ctx, &mut NullTrapSink {})
            .unwrap();
        self.module.clear_context(&mut ctx);
        self.module.finalize_definitions();

        let code_ptr = self.module.get_finalized_function(id);
        let func_ptr = unsafe { mem::transmute(code_ptr) };
        self.func_ptrs.insert(func_idx, func_ptr);
        func_ptr
    }

    pub fn get(&self, func_idx: usize) -> Option<NativeFn> {
        self.func_ptrs.get(&func_idx).copied()
    }
}

extern "C" {
    pub fn ry_load_string(arena: *mut Arena, idx: usize) -> Value;
    pub fn ry_print(val: Value);
    pub fn ry_call_function(arena: *mut Arena, args: *const Value, nargs: usize, func_idx: usize) -> Value;
}