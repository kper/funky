/// IR datastructure for the webassembly module.
#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
}

/// IR datastructure for the webassembly function.
#[derive(Debug, Clone, Default)]
pub struct Function {
    /// Name of the function
    pub name: String,
    pub(crate) params: Vec<String>,
    pub(crate) definitions: Vec<String>,
    pub(crate) results_len: usize,
    pub(crate) instructions: Vec<Instruction>,
}

impl Function {
    pub fn get_num_definitions(&self) -> usize {
        self.definitions.len()
    }

    pub fn get_num_instructions(&self) -> usize {
        self.instructions.len()
    }
}

type Dest = String;
type Src = String;
type Reg = String;
type Regs = Vec<String>;

/// IR datastructure for abstracted webassembly instructions.
#[derive(Debug, Clone)]
pub enum Instruction {
    Block(String),
    Unop(Dest, Src),
    BinOp(Dest, Src, Src),
    Const(Dest, f64),
    Assign(Dest, Src),
    Jump(String),
    Call(String, Regs, Regs),
    Kill(Dest),
    Conditional(Src, Vec<String>),
    Return(Vec<Reg>),
    Table(Vec<String>),
    Phi(Dest, Src, Src),
    CallIndirect(Vec<String>, Vec<Reg>, Vec<Reg>), // names, parameters, dests
    Unknown(Dest),                                 // Value is not known statically
    Store(Src, f64, Src),
    Load(Dest, f64, Src),
}
