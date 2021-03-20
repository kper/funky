#![allow(dead_code)]
/// This module is responsible for converting the IR to an AST.

#[derive(Debug, Clone)]
pub struct Program {
    pub(crate) functions: Vec<Function>
}

#[derive(Debug, Clone, Default)]
pub struct Function {
    pub(crate) name: String,
    pub(crate) params: Vec<String>,
    pub(crate) definitions: Vec<String>,
    pub(crate) results_len: usize,
    pub(crate) instructions: Vec<Instruction>,
}

type Dest = String;
type Src = String;
type Name = usize;
type Reg = String;
type Regs = Vec<String>;
type FunctionName = usize;
type Count = usize;
type Cont = bool;

#[derive(Debug, Clone)]
pub enum Instruction {
    Block(String),
    Unop(Dest, Src),
    BinOp(Dest, Src, Src),
    Const(Dest, f64),
    Assign(Dest, Src),
    Jump(String),
    Call(String, Vec<Reg>, Regs),
    Kill(Dest),
    Conditional(Src, Vec<String>),
    Return(Vec<Reg>),
    Table(usize),
}