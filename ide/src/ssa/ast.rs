#![allow(dead_code)]
/// This module is responsible for converting the IR to an AST.

#[derive(Debug)]
pub struct Program {
    pub(crate) functions: Vec<Function>
}

#[derive(Debug)]
pub struct Function {
    pub(crate) name: String,
    pub(crate) params: Vec<String>,
    pub(crate) results_len: usize,
    pub(crate) instructions: Vec<Instruction>,
}

type Dest = String;
type Src = String;
type Name = usize;
type Reg = String;
type Regs = Vec<String>;
type FunctionName = usize;

#[derive(Debug)]
pub enum Instruction {
    Block(String),
    Unop(Dest, Src),
    BinOp(Dest, Src, Src),
    Const(Dest, f64),
    Assign(Dest, Src),
    Jump(String),
    Call(String, Vec<Src>, Regs),
    /// kill the variable
    Kill(Dest),
}


