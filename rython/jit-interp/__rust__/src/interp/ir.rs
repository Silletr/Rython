pub type ValueId = u32;
pub type BasicBlockId = u32;
pub type FunctionId = usize;

#[derive(Debug, Clone)]
pub enum Constant {
    Int(i64),
    Bool(bool),
    Nil,
    String(String),
}

#[derive(Debug, Clone)]
pub enum IrNode {
    Add(ValueId, ValueId, ValueId),
    Sub(ValueId, ValueId, ValueId),
    Mul(ValueId, ValueId, ValueId),
    Div(ValueId, ValueId, ValueId),
    Lt(ValueId, ValueId, ValueId),
    Le(ValueId, ValueId, ValueId),
    Const(ValueId, Constant),
    Param(ValueId, usize),
    Jump(BasicBlockId),
    Branch(ValueId, BasicBlockId, BasicBlockId),
    Return(ValueId),
    Print(ValueId),
    Call(ValueId, FunctionId, Vec<ValueId>),
    Move(ValueId, ValueId),
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BasicBlockId,
    pub instructions: Vec<IrNode>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub blocks: Vec<BasicBlock>,
    pub entry: BasicBlockId,
}

#[derive(Debug, Clone)]
pub struct IrProgram {
    pub functions: Vec<Function>,
    pub main: FunctionId,
}

impl IrProgram {
    pub fn new() -> Self {
        IrProgram {
            functions: Vec::new(),
            main: 0,
        }
    }
}