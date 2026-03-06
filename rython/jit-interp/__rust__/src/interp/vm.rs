use crate::value::Value;
use crate::bytecode::{Instruction, OpCode};
use crate::arena::Arena;
use crate::jit::cranelift::{CraneliftJIT, NativeFn};
use std::rc::Rc;
use std::collections::HashMap;

struct Frame {
    registers: Vec<Value>,
    program: Rc<Vec<Instruction>>,
    pc: usize,
    return_register: u8,
    saved_regs: Vec<Value>,
}

impl Frame {
    fn new(program: Rc<Vec<Instruction>>, num_regs: usize) -> Self {
        Frame {
            registers: vec![Value::nil(); num_regs],
            program,
            pc: 0,
            return_register: 0,
            saved_regs: Vec::new(),
        }
    }
}

pub struct Profiler {
    call_counts: HashMap<usize, usize>,
    loop_counts: HashMap<usize, usize>,
    hot_threshold: usize,
}

impl Profiler {
    pub fn new(hot_threshold: usize) -> Self {
        Profiler {
            call_counts: HashMap::new(),
            loop_counts: HashMap::new(),
            hot_threshold,
        }
    }

    pub fn record_call(&mut self, func_idx: usize) {
        *self.call_counts.entry(func_idx).or_insert(0) += 1;
    }

    pub fn is_hot(&self, func_idx: usize) -> bool {
        self.call_counts.get(&func_idx).unwrap_or(&0) > &self.hot_threshold
    }

    pub fn record_loop(&mut self, _block_id: usize) {
        // optional
    }
}

pub struct VM {
    frames: Vec<Frame>,
    functions: Vec<(Rc<Vec<Instruction>>, usize, usize)>, // bytecode, #params, #regs
    string_pool: Vec<String>,
    arena: Arena,

    profiler: Profiler,
    jit_compiler: CraneliftJIT,
    threshold: usize,
}

impl VM {
    pub fn new(
        functions: Vec<(Rc<Vec<Instruction>>, usize, usize)>,
        string_pool: Vec<String>,
        arena_capacity: usize,
        hot_threshold: usize,
    ) -> Self {
        let mut vm = VM {
            frames: Vec::new(),
            functions,
            string_pool,
            arena: Arena::new(arena_capacity, std::ptr::null_mut()),
            profiler: Profiler::new(hot_threshold),
            jit_compiler: CraneliftJIT::new(),
            threshold: hot_threshold,
        };
        vm.arena.vm_ptr = &mut vm as *mut VM;
        let main_frame = Frame::new(vm.functions[0].0.clone(), vm.functions[0].2);
        vm.frames.push(main_frame);
        vm
    }

    pub fn run(&mut self) -> Result<Value, String> {
        let mut instruction_count = 0;
        const MAX_INSTRUCTIONS: usize = 10_000_000_000;

        while !self.frames.is_empty() {
            instruction_count += 1;
            if instruction_count > MAX_INSTRUCTIONS {
                return Err(format!("Infinite loop detected after {} instructions", instruction_count));
            }

            let frame = self.frames.last_mut().unwrap();
            if frame.pc >= frame.program.len() {
                self.frames.pop();
                if self.frames.is_empty() {
                    return Ok(Value::nil());
                }
                continue;
            }

            let insn = frame.program[frame.pc];
            frame.pc += 1;

            // Debug prints for key opcodes
            if cfg!(debug_assertions) {
                match insn.opcode() {
                    OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div => {
                        println!("DEBUG: Arithmetic {:?} r{} = r{}, r{} at pc {}", 
                            insn.opcode(), insn.dst(), insn.src1(), insn.src2(), frame.pc-1);
                    }
                    OpCode::Lt | OpCode::Le => {
                        println!("DEBUG: Compare {:?} r{} = r{}, r{} at pc {}", 
                            insn.opcode(), insn.dst(), insn.src1(), insn.src2(), frame.pc-1);
                    }
                    OpCode::LoadConst => {
                        println!("DEBUG: LoadConst r{} = {} at pc {}", insn.dst(), insn.imm(), frame.pc-1);
                    }
                    OpCode::Move => {
                        println!("DEBUG: Move r{} = r{} at pc {}", insn.dst(), insn.src1(), frame.pc-1);
                    }
                    OpCode::Jump => {
                        println!("DEBUG: Jump to pc {} at pc {}", insn.imm(), frame.pc-1);
                    }
                    OpCode::Branch => {
                        println!("DEBUG: Branch r{} -> {} at pc {}", insn.dst(), insn.imm(), frame.pc-1);
                    }
                    OpCode::Return => {
                        println!("DEBUG: Return r{} at pc {}", insn.dst(), frame.pc-1);
                    }
                    OpCode::Call => {
                        println!("DEBUG: Call function {} at pc {}", insn.imm(), frame.pc-1);
                    }
                    _ => {}
                }
            }

            match insn.opcode() {
                OpCode::Add => {
                    let dst = insn.dst() as usize;
                    let src1 = insn.src1() as usize;
                    let src2 = insn.src2() as usize;
                    let a = frame.registers[src1];
                    let b = frame.registers[src2];
                    let result = a + b;
                    if cfg!(debug_assertions) {
                        println!("  => r{} = {:?} + {:?} = {:?}", dst, a, b, result);
                    }
                    frame.registers[dst] = result;
                }
                OpCode::Sub => {
                    let dst = insn.dst() as usize;
                    let src1 = insn.src1() as usize;
                    let src2 = insn.src2() as usize;
                    let a = frame.registers[src1];
                    let b = frame.registers[src2];
                    let result = a - b;
                    if cfg!(debug_assertions) {
                        println!("  => r{} = {:?} - {:?} = {:?}", dst, a, b, result);
                    }
                    frame.registers[dst] = result;
                }
                OpCode::Mul => {
                    let dst = insn.dst() as usize;
                    let src1 = insn.src1() as usize;
                    let src2 = insn.src2() as usize;
                    let a = frame.registers[src1];
                    let b = frame.registers[src2];
                    let result = a * b;
                    if cfg!(debug_assertions) {
                        println!("  => r{} = {:?} * {:?} = {:?}", dst, a, b, result);
                    }
                    frame.registers[dst] = result;
                }
                OpCode::Div => {
                    let dst = insn.dst() as usize;
                    let src1 = insn.src1() as usize;
                    let src2 = insn.src2() as usize;
                    let a = frame.registers[src1];
                    let b = frame.registers[src2];
                    let result = a / b;
                    if cfg!(debug_assertions) {
                        println!("  => r{} = {:?} / {:?} = {:?}", dst, a, b, result);
                    }
                    frame.registers[dst] = result;
                }
                OpCode::Lt => {
                    let dst = insn.dst() as usize;
                    let src1 = insn.src1() as usize;
                    let src2 = insn.src2() as usize;
                    let a = frame.registers[src1];
                    let b = frame.registers[src2];
                    let result = if let (Some(ai), Some(bi)) = (a.to_int(), b.to_int()) {
                        Value::from_bool(ai < bi)
                    } else {
                        Value::from_bool(false)
                    };
                    if cfg!(debug_assertions) {
                        println!("  => r{} = {:?} < {:?} = {:?}", dst, a, b, result);
                    }
                    frame.registers[dst] = result;
                }
                OpCode::Le => {
                    let dst = insn.dst() as usize;
                    let src1 = insn.src1() as usize;
                    let src2 = insn.src2() as usize;
                    let a = frame.registers[src1];
                    let b = frame.registers[src2];
                    let result = if let (Some(ai), Some(bi)) = (a.to_int(), b.to_int()) {
                        Value::from_bool(ai <= bi)
                    } else {
                        Value::from_bool(false)
                    };
                    if cfg!(debug_assertions) {
                        println!("  => r{} = {:?} <= {:?} = {:?}", dst, a, b, result);
                    }
                    frame.registers[dst] = result;
                }
                OpCode::LoadConst => {
                    let dst = insn.dst() as usize;
                    let imm = insn.imm() as i64;
                    frame.registers[dst] = Value::from_int(imm);
                    if cfg!(debug_assertions) {
                        println!("  => r{} = {}", dst, imm);
                    }
                }
                OpCode::LoadBool => {
                    let dst = insn.dst() as usize;
                    let b = insn.imm() != 0;
                    frame.registers[dst] = Value::from_bool(b);
                    if cfg!(debug_assertions) {
                        println!("  => r{} = {}", dst, b);
                    }
                }
                OpCode::LoadNil => {
                    let dst = insn.dst() as usize;
                    frame.registers[dst] = Value::nil();
                }
                OpCode::LoadString => {
                    let dst = insn.dst() as usize;
                    let idx = insn.imm() as usize;
                    if idx >= self.string_pool.len() {
                        panic!("LoadString: index {} out of bounds", idx);
                    }
                    let s = &self.string_pool[idx];
                    frame.registers[dst] = Value::from_string_in_arena(s, &mut self.arena);
                }
                OpCode::Jump => {
                    let target = insn.imm() as usize;
                    frame.pc = target;
                }
                OpCode::Branch => {
                    let cond_reg = insn.dst() as usize;
                    let target = insn.imm() as usize;
                    let cond = frame.registers[cond_reg];
                    let take = if let Some(b) = cond.to_bool() {
                        b
                    } else if let Some(i) = cond.to_int() {
                        i != 0
                    } else if let Some(f) = cond.to_f64() {
                        f != 0.0
                    } else {
                        false
                    };
                    if cfg!(debug_assertions) {
                        println!("  => cond {:?} -> take={}", cond, take);
                    }
                    if take {
                        frame.pc = target;
                    }
                }
                OpCode::Return => {
                    let val_reg = insn.dst() as usize;
                    let ret_val = frame.registers[val_reg];
                    if cfg!(debug_assertions) {
                        println!("DEBUG: Return from frame, value = {:?}", ret_val);
                    }
                    self.frames.pop();
                    if let Some(caller) = self.frames.last_mut() {
                        let ret_dst = caller.return_register as usize;
                        caller.registers[ret_dst] = ret_val;
                        // Restore saved registers, but avoid overwriting the return register
                        if !caller.saved_regs.is_empty() {
                            for (i, &val) in caller.saved_regs.iter().enumerate() {
                                if i < caller.registers.len() && i != ret_dst {
                                    caller.registers[i] = val;
                                }
                            }
                            caller.saved_regs.clear();
                        }
                    } else {
                        // This was the main frame, return the value
                        if cfg!(debug_assertions) {
                            println!("DEBUG: Main frame returning {:?}", ret_val);
                        }
                        return Ok(ret_val);
                    }
                }
                OpCode::Print => {
                    let reg = insn.dst() as usize;
                    let val = frame.registers[reg];
                    println!("{:?}", val);
                }
                OpCode::Move => {
                    let dst = insn.dst() as usize;
                    let src = insn.src1() as usize;
                    let val = frame.registers[src];
                    if cfg!(debug_assertions) {
                        println!("  => r{} = r{} ({:?})", dst, src, val);
                    }
                    frame.registers[dst] = val;
                }
                // TODO, fix the JIT switch block bug
                OpCode::Call => {
                    let func_idx = insn.imm() as usize;
                    if func_idx >= self.functions.len() {
                        panic!("Call to undefined function index {}", func_idx);
                    }

                    self.profiler.record_call(func_idx);

                    if self.profiler.is_hot(func_idx) {
                        if let Some(native_fn) = self.jit_compiler.get(func_idx) {
                            if cfg!(debug_assertions) {
                                println!("JIT: calling compiled version of function {}", func_idx);
                            }
                            let num_params = self.functions[func_idx].1;
                            let mut args = Vec::with_capacity(num_params);
                            for i in 0..num_params {
                                args.push(frame.registers[i]);
                            }

                            frame.saved_regs = frame.registers.clone();

                            let ret_val = unsafe {
                                native_fn(
                                    &mut self.arena as *mut Arena,
                                    args.as_ptr(),
                                    args.len(),
                                )
                            };

                            if !frame.saved_regs.is_empty() {
                                for (i, &val) in frame.saved_regs.iter().enumerate() {
                                    if i < frame.registers.len() && i != 0 {
                                        frame.registers[i] = val;
                                    }
                                }
                                frame.saved_regs.clear();
                            }

                            let dst = insn.dst() as usize;
                            frame.registers[dst] = ret_val;
                            continue;
                        } else {
                            if cfg!(debug_assertions) {
                                println!("JIT: compiling function {} (hot)", func_idx);
                            }
                            let (bytecode, num_params, num_regs) = self.functions[func_idx].clone();
                            self.jit_compiler.compile(func_idx, &bytecode, num_params, num_regs);
                        }
                    }

                    let (callee_bytecode, num_params, max_reg) = self.functions[func_idx].clone();
                    frame.saved_regs = frame.registers.clone();
                    let mut new_frame = Frame::new(callee_bytecode, max_reg);
                    new_frame.return_register = insn.dst();
                    for i in 0..num_params {
                        new_frame.registers[i] = frame.registers[i];
                    }
                    self.frames.push(new_frame);
                }
            }
        }
        Ok(Value::nil())
    }

    pub fn profiler(&self) -> &Profiler {
        &self.profiler
    }

    pub fn profiler_mut(&mut self) -> &mut Profiler {
        &mut self.profiler
    }
}

#[no_mangle]
pub unsafe extern "C" fn ry_load_string(arena_ptr: *mut Arena, idx: usize) -> Value {
    let arena = &mut *arena_ptr;
    let vm = &mut *(arena.vm_ptr as *mut VM);
    if idx >= vm.string_pool.len() {
        panic!("ry_load_string: index {} out of bounds", idx);
    }
    let s = &vm.string_pool[idx];
    Value::from_string_in_arena(s, arena)
}

#[no_mangle]
pub unsafe extern "C" fn ry_print(val: Value) {
    if cfg!(debug_assertions) {
        println!("{:?}", val);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ry_call_function(
    arena_ptr: *mut Arena,
    args_ptr: *const Value,
    nargs: usize,
    func_idx: usize,
) -> Value {
    let arena = &mut *arena_ptr;
    let vm = &mut *(arena.vm_ptr as *mut VM);

    if let Some(native_fn) = vm.jit_compiler.get(func_idx) {
        return native_fn(arena_ptr, args_ptr, nargs);
    }

    let (bytecode, num_params, num_regs) = vm.functions[func_idx].clone();
    let mut frame = Frame::new(bytecode, num_regs);
    for i in 0..num_params.min(nargs) {
        frame.registers[i] = *args_ptr.add(i);
    }
    let mut result = Value::nil();
    while frame.pc < frame.program.len() {
        let insn = frame.program[frame.pc];
        frame.pc += 1;
        match insn.opcode() {
            OpCode::Return => {
                let reg = insn.dst() as usize;
                result = frame.registers[reg];
                break;
            }
            _ => panic!("ry_call_function: fallback interpreter incomplete"),
        }
    }
    result
}