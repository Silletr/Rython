#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    Add = 0,
    Sub,
    Mul,
    Div,
    Lt,
    Le,
    LoadConst,
    LoadBool,
    LoadNil,
    LoadString,
    Jump,
    Branch,
    Return,
    Print,
    Move,
    Call,
    Inc,  // new fused instruction: increment register by 1
}

#[derive(Clone, Copy)]
pub struct Instruction(u32);

impl Instruction {
    #[inline(always)]
    pub fn encode_rrr(op: OpCode, dst: u8, src1: u8, src2: u8) -> Self {
        let word = ((op as u32) << 24) | ((dst as u32) << 16) | ((src1 as u32) << 8) | (src2 as u32);
        Instruction(word)
    }

    #[inline(always)]
    pub fn encode_rr(op: OpCode, dst: u8, src: u8) -> Self {
        Instruction::encode_rrr(op, dst, src, 0)
    }

    #[inline(always)]
    pub fn encode_imm(op: OpCode, dst: u8, imm: u16) -> Self {
        let word = ((op as u32) << 24) | ((dst as u32) << 16) | (imm as u32);
        Instruction(word)
    }

    #[inline(always)]
    pub fn opcode(&self) -> OpCode {
        unsafe { std::mem::transmute(((self.0 >> 24) & 0xff) as u8) }
    }

    #[inline(always)]
    pub fn dst(&self) -> u8 {
        ((self.0 >> 16) & 0xff) as u8
    }

    #[inline(always)]
    pub fn src1(&self) -> u8 {
        ((self.0 >> 8) & 0xff) as u8
    }

    #[inline(always)]
    pub fn src2(&self) -> u8 {
        (self.0 & 0xff) as u8
    }

    #[inline(always)]
    pub fn imm(&self) -> u16 {
        (self.0 & 0xffff) as u16
    }
}