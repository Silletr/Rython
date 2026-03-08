use crate::ir::*;
use crate::bytecode::{Instruction, OpCode};
use std::collections::HashMap;
use std::rc::Rc;

pub struct LoweringContext {
    reg_map: Vec<u8>,
    next_reg: u8,
    bytecode: Vec<Instruction>,
    block_starts: HashMap<BasicBlockId, usize>,
    pending_jumps: Vec<(usize, BasicBlockId, bool)>,
    string_pool: Vec<String>,
    string_map: HashMap<String, u16>,
}

impl LoweringContext {
    pub fn new() -> Self {
        LoweringContext {
            reg_map: Vec::new(),
            next_reg: 0,
            bytecode: Vec::new(),
            block_starts: HashMap::new(),
            pending_jumps: Vec::new(),
            string_pool: Vec::new(),
            string_map: HashMap::new(),
        }
    }

    fn ensure_reg(&mut self, v: ValueId) -> u8 {
        while self.reg_map.len() <= v as usize {
            self.reg_map.push(self.next_reg);
            self.next_reg += 1;
        }
        self.reg_map[v as usize]
    }

    fn lower_function(&mut self, func: &Function) -> (Vec<Instruction>, usize) {
        self.reg_map.clear();
        self.next_reg = 0;
        self.bytecode.clear();
        self.block_starts.clear();
        self.pending_jumps.clear();

        for (idx, _) in func.params.iter().enumerate() {
            self.ensure_reg(idx as ValueId);
        }

        for block in &func.blocks {
            self.block_starts.insert(block.id, self.bytecode.len());
            for node in &block.instructions {
                match node {
                    IrNode::Add(dst, a, b) => {
                        let rd = self.ensure_reg(*dst);
                        let rs1 = self.ensure_reg(*a);
                        let rs2 = self.ensure_reg(*b);
                        self.bytecode.push(Instruction::encode_rrr(OpCode::Add, rd, rs1, rs2));
                    }
                    IrNode::Sub(dst, a, b) => {
                        let rd = self.ensure_reg(*dst);
                        let rs1 = self.ensure_reg(*a);
                        let rs2 = self.ensure_reg(*b);
                        self.bytecode.push(Instruction::encode_rrr(OpCode::Sub, rd, rs1, rs2));
                    }
                    IrNode::Mul(dst, a, b) => {
                        let rd = self.ensure_reg(*dst);
                        let rs1 = self.ensure_reg(*a);
                        let rs2 = self.ensure_reg(*b);
                        self.bytecode.push(Instruction::encode_rrr(OpCode::Mul, rd, rs1, rs2));
                    }
                    IrNode::Div(dst, a, b) => {
                        let rd = self.ensure_reg(*dst);
                        let rs1 = self.ensure_reg(*a);
                        let rs2 = self.ensure_reg(*b);
                        self.bytecode.push(Instruction::encode_rrr(OpCode::Div, rd, rs1, rs2));
                    }
                    IrNode::Lt(dst, a, b) => {
                        let rd = self.ensure_reg(*dst);
                        let rs1 = self.ensure_reg(*a);
                        let rs2 = self.ensure_reg(*b);
                        self.bytecode.push(Instruction::encode_rrr(OpCode::Lt, rd, rs1, rs2));
                    }
                    IrNode::Le(dst, a, b) => {
                        let rd = self.ensure_reg(*dst);
                        let rs1 = self.ensure_reg(*a);
                        let rs2 = self.ensure_reg(*b);
                        self.bytecode.push(Instruction::encode_rrr(OpCode::Le, rd, rs1, rs2));
                    }
                    IrNode::Const(dst, Constant::Int(i)) => {
                        let rd = self.ensure_reg(*dst);
                        self.bytecode.push(Instruction::encode_imm(OpCode::LoadConst, rd, *i as u16));
                    }
                    IrNode::Const(dst, Constant::Bool(b)) => {
                        let rd = self.ensure_reg(*dst);
                        let imm = if *b { 1 } else { 0 };
                        self.bytecode.push(Instruction::encode_imm(OpCode::LoadBool, rd, imm));
                    }
                    IrNode::Const(dst, Constant::Nil) => {
                        let rd = self.ensure_reg(*dst);
                        self.bytecode.push(Instruction::encode_imm(OpCode::LoadNil, rd, 0));
                    }
                    IrNode::Const(dst, Constant::String(s)) => {
                        let rd = self.ensure_reg(*dst);
                        let idx = if let Some(&i) = self.string_map.get(s) {
                            i
                        } else {
                            let i = self.string_pool.len() as u16;
                            self.string_pool.push(s.clone());
                            self.string_map.insert(s.clone(), i);
                            i
                        };
                        self.bytecode.push(Instruction::encode_imm(OpCode::LoadString, rd, idx));
                    }
                    IrNode::Param(dst, idx) => {
                        let rd = self.ensure_reg(*dst);
                        self.bytecode.push(Instruction::encode_rr(OpCode::Move, rd, *idx as u8));
                    }
                    IrNode::Jump(target) => {
                        let pos = self.bytecode.len();
                        self.bytecode.push(Instruction::encode_imm(OpCode::Jump, 0, 0));
                        self.pending_jumps.push((pos, *target, false));
                    }
                    IrNode::Branch(cond, t, _f) => {
                        let cond_reg = self.ensure_reg(*cond);
                        let pos = self.bytecode.len();
                        self.bytecode.push(Instruction::encode_imm(OpCode::Branch, cond_reg, 0));
                        self.pending_jumps.push((pos, *t, true));
                    }
                    IrNode::Return(val) => {
                        let rval = self.ensure_reg(*val);
                        self.bytecode.push(Instruction::encode_imm(OpCode::Return, rval, 0));
                    }
                    IrNode::Print(val) => {
                        let rval = self.ensure_reg(*val);
                        self.bytecode.push(Instruction::encode_imm(OpCode::Print, rval, 0));
                    }
                    IrNode::Call(dst, func_idx, args) => {
                        for (i, arg_reg) in args.iter().enumerate() {
                            let src = self.ensure_reg(*arg_reg);
                            self.bytecode.push(Instruction::encode_rr(OpCode::Move, i as u8, src));
                        }
                        self.bytecode.push(Instruction::encode_imm(OpCode::Call, 0, *func_idx as u16));
                        let rd = self.ensure_reg(*dst);
                        if rd != 0 {
                            self.bytecode.push(Instruction::encode_rr(OpCode::Move, rd, 0));
                        }
                    }
                    IrNode::Move(dst, src) => {
                        let rd = self.ensure_reg(*dst);
                        let rs = self.ensure_reg(*src);
                        self.bytecode.push(Instruction::encode_rr(OpCode::Move, rd, rs));
                    }
                }
            }
        }

        for (pos, target, is_branch) in &self.pending_jumps {
            let target_pc = *self.block_starts.get(target).unwrap() as u16;
            let insn = self.bytecode[*pos];
            let op = insn.opcode();
            let dst = insn.dst();
            let new_insn = if *is_branch {
                Instruction::encode_imm(op, dst, target_pc)
            } else {
                Instruction::encode_imm(op, 0, target_pc)
            };
            self.bytecode[*pos] = new_insn;
        }

        // Apply peephole optimizations
        let optimized = self.bytecode.clone(); 

        if cfg!(debug_assertions) && func.name == "main" {  // assuming func has a name field
        println!("Bytecode for main:");
        for (i, insn) in optimized.iter().enumerate() {
            println!("  {}: {:?} dst={} src1={} src2={} imm={}", 
                i, insn.opcode(), insn.dst(), insn.src1(), insn.src2(), insn.imm());
        }
    }

        (optimized, self.next_reg as usize)
    }

    pub fn lower_program(&mut self, program: &IrProgram) -> (Vec<(Rc<Vec<Instruction>>, usize, usize)>, Vec<String>) {
    let mut functions = Vec::new();
    for (idx, func) in program.functions.iter().enumerate() {
        let (bytecode, max_reg) = self.lower_function(func);
        let param_count = func.params.len();
        if idx == 0 {  // main function
            println!("Bytecode for main (function {}):", idx);
            for (i, insn) in bytecode.iter().enumerate() {
                println!("  {}: {:?} dst={} src1={} src2={} imm={}",
                    i, insn.opcode(), insn.dst(), insn.src1(), insn.src2(), insn.imm());
            }
        }
        functions.push((Rc::new(bytecode), param_count, max_reg));
    }
    (functions, self.string_pool.clone())
    }
}