use crate::value::Value;
use crate::bytecode::{Instruction, OpCode};
use crate::arena::Arena;
use std::rc::Rc;

struct Function {
    bytecode: Rc<Vec<Instruction>>,
    num_params: usize,
    num_regs: usize,
}

struct Frame {
    reg_base: usize,
    func_idx: usize,
    pc: usize,
    return_reg: usize,
}

pub struct Profiler {
    call_counts: Vec<usize>,
    hot_threshold: usize,
}

impl Profiler {
    pub fn new(num_functions: usize, hot_threshold: usize) -> Self {
        Profiler {
            call_counts: vec![0; num_functions],
            hot_threshold,
        }
    }

    #[inline(always)]
    pub fn record_call(&mut self, func_idx: usize) {
        if func_idx < self.call_counts.len() {
            self.call_counts[func_idx] += 1;
        }
    }
}

pub struct VM {
    register_pool: Vec<Value>,
    frames: Vec<Frame>,
    functions: Vec<Function>,
    string_pool: Vec<String>,
    arena: Arena,
    profiler: Profiler,
    max_call_depth: usize,
}

impl VM {
    pub fn new(
        functions: Vec<(Rc<Vec<Instruction>>, usize, usize)>,
        string_pool: Vec<String>,
        arena_capacity: usize,
        hot_threshold: usize,
        max_call_depth: usize,
    ) -> Self {
        let num_functions = functions.len();
        let funcs: Vec<Function> = functions
            .into_iter()
            .map(|(bytecode, num_params, num_regs)| Function {
                bytecode,
                num_params,
                num_regs,
            })
            .collect();

        let max_regs = funcs.iter().map(|f| f.num_regs).max().unwrap_or(0);
        let pool_size = max_regs * max_call_depth;
        let register_pool = vec![Value::nil(); pool_size];
        let frames = Vec::with_capacity(max_call_depth);

        let mut vm = VM {
            register_pool,
            frames,
            functions: funcs,
            string_pool,
            arena: Arena::new(arena_capacity, std::ptr::null_mut()),
            profiler: Profiler::new(num_functions, hot_threshold),
            max_call_depth,
        };
        vm.arena.vm_ptr = &mut vm as *mut VM;

        let main_frame = Frame {
            reg_base: 0,
            func_idx: 0,
            pc: 0,
            return_reg: 0,
        };
        vm.frames.push(main_frame);
        vm
    }

    pub fn run(&mut self) -> Result<Value, String> {
        let mut instruction_count = 0;
        const MAX_INSTRUCTIONS: usize = 10_000_000_000;

        while let Some(frame) = self.frames.last_mut() {
            instruction_count += 1;
            if instruction_count > MAX_INSTRUCTIONS {
                return Err(format!("Infinite loop detected after {} instructions", instruction_count));
            }

            let func = &self.functions[frame.func_idx];
            if frame.pc >= func.bytecode.len() {
                self.frames.pop();
                continue;
            }

            let insn = func.bytecode[frame.pc];
            frame.pc += 1;

            match insn.opcode() {
                OpCode::Add => {
                    let dst = frame.reg_base + insn.dst() as usize;
                    let src1 = frame.reg_base + insn.src1() as usize;
                    let src2 = frame.reg_base + insn.src2() as usize;
                    self.register_pool[dst] = self.register_pool[src1] + self.register_pool[src2];
                }
                OpCode::Sub => {
                    let dst = frame.reg_base + insn.dst() as usize;
                    let src1 = frame.reg_base + insn.src1() as usize;
                    let src2 = frame.reg_base + insn.src2() as usize;
                    self.register_pool[dst] = self.register_pool[src1] - self.register_pool[src2];
                }
                OpCode::Mul => {
                    let dst = frame.reg_base + insn.dst() as usize;
                    let src1 = frame.reg_base + insn.src1() as usize;
                    let src2 = frame.reg_base + insn.src2() as usize;
                    self.register_pool[dst] = self.register_pool[src1] * self.register_pool[src2];
                }
                OpCode::Div => {
                    let dst = frame.reg_base + insn.dst() as usize;
                    let src1 = frame.reg_base + insn.src1() as usize;
                    let src2 = frame.reg_base + insn.src2() as usize;
                    self.register_pool[dst] = self.register_pool[src1] / self.register_pool[src2];
                }
                OpCode::Lt => {
                    let dst = frame.reg_base + insn.dst() as usize;
                    let src1 = frame.reg_base + insn.src1() as usize;
                    let src2 = frame.reg_base + insn.src2() as usize;
                    let a = self.register_pool[src1];
                    let b = self.register_pool[src2];
                    self.register_pool[dst] = if let (Some(ai), Some(bi)) = (a.to_int(), b.to_int()) {
                        Value::from_bool(ai < bi)
                    } else {
                        Value::from_bool(false)
                    };
                }
                OpCode::Le => {
                    let dst = frame.reg_base + insn.dst() as usize;
                    let src1 = frame.reg_base + insn.src1() as usize;
                    let src2 = frame.reg_base + insn.src2() as usize;
                    let a = self.register_pool[src1];
                    let b = self.register_pool[src2];
                    self.register_pool[dst] = if let (Some(ai), Some(bi)) = (a.to_int(), b.to_int()) {
                        Value::from_bool(ai <= bi)
                    } else {
                        Value::from_bool(false)
                    };
                }
                OpCode::LoadConst => {
                    let dst = frame.reg_base + insn.dst() as usize;
                    let imm = insn.imm() as i64;
                    if frame.func_idx == 0 {
                        println!("main LoadConst: imm={}, dst_reg={}", imm, insn.dst());
                    }
                    self.register_pool[dst] = Value::from_int(imm);
                }
                OpCode::LoadBool => {
                    let dst = frame.reg_base + insn.dst() as usize;
                    self.register_pool[dst] = Value::from_bool(insn.imm() != 0);
                }
                OpCode::LoadNil => {
                    let dst = frame.reg_base + insn.dst() as usize;
                    self.register_pool[dst] = Value::nil();
                }
                OpCode::LoadString => {
                    let dst = frame.reg_base + insn.dst() as usize;
                    let idx = insn.imm() as usize;
                    if idx >= self.string_pool.len() {
                        panic!("LoadString: index {} out of bounds", idx);
                    }
                    let s = &self.string_pool[idx];
                    let val = Value::from_string_in_arena(s, &mut self.arena);
                    self.register_pool[dst] = val;
                }
                OpCode::Jump => {
                    frame.pc = insn.imm() as usize;
                }
                OpCode::Branch => {
                    let cond_reg = frame.reg_base + insn.dst() as usize;
                    let cond = self.register_pool[cond_reg];
                    let take = if let Some(b) = cond.to_bool() {
                        b
                    } else if let Some(i) = cond.to_int() {
                        i != 0
                    } else if let Some(f) = cond.to_f64() {
                        f != 0.0
                    } else {
                        false
                    };
                    if take {
                        frame.pc = insn.imm() as usize;
                    }
                }
                OpCode::Return => {
                    let val_reg = frame.reg_base + insn.dst() as usize;
                    let ret_val = self.register_pool[val_reg];
                    if frame.func_idx == 1 {
                        println!("heavy returning: {:?}", ret_val);
                    }
                    self.frames.pop(); // remove current frame
                    if let Some(caller) = self.frames.last_mut() {
                        self.register_pool[caller.return_reg] = ret_val;
                    } else {
                        return Ok(ret_val);
                    }
                }
                OpCode::Print => {
                    let reg = frame.reg_base + insn.dst() as usize;
                    println!("{:?}", self.register_pool[reg]);
                }
                OpCode::Move => {
                    let dst = frame.reg_base + insn.dst() as usize;
                    let src = frame.reg_base + insn.src1() as usize;
                    self.register_pool[dst] = self.register_pool[src];
                }
                OpCode::Call => {
                    let func_idx = insn.imm() as usize;
                    if func_idx == 1 {
                        let arg = self.register_pool[frame.reg_base];
                        println!("Calling heavy with argument: {:?}", arg);
                    }
                    if func_idx >= self.functions.len() {
                        panic!("Call to undefined function index {}", func_idx);
                    }

                    let callee = &self.functions[func_idx];
                    let num_params = callee.num_params;
                    let return_reg = frame.reg_base + insn.dst() as usize;
                    let new_base = frame.reg_base + self.functions[frame.func_idx].num_regs;

                    if new_base + callee.num_regs > self.register_pool.len() {
                        panic!("Call stack overflow");
                    }

                    for i in 0..num_params {
                        self.register_pool[new_base + i] = self.register_pool[frame.reg_base + i];
                    }

                    let new_frame = Frame {
                        reg_base: new_base,
                        func_idx,
                        pc: 0,
                        return_reg,
                    };
                    self.frames.push(new_frame);
                    self.profiler.record_call(func_idx);
                }
                OpCode::Inc => {
                    let reg = frame.reg_base + insn.dst() as usize;
                    self.register_pool[reg] = self.register_pool[reg] + Value::from_int(1);
                }
            }
        }
        Ok(Value::nil())
    }

    pub fn profiler(&self) -> &Profiler {
        &self.profiler
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
    println!("{:?}", val);
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

    let (bytecode, num_params, num_regs) = (
        vm.functions[func_idx].bytecode.clone(),
        vm.functions[func_idx].num_params,
        vm.functions[func_idx].num_regs,
    );
    let mut registers = vec![Value::nil(); num_regs];
    for i in 0..num_params.min(nargs) {
        registers[i] = *args_ptr.add(i);
    }
    let mut pc = 0;
    while pc < bytecode.len() {
        let insn = bytecode[pc];
        pc += 1;
        match insn.opcode() {
            OpCode::Return => {
                let reg = insn.dst() as usize;
                return registers[reg];
            }
            _ => {}
        }
    }
    Value::nil()
}