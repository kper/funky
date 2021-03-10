#![allow(dead_code)]
/// This module is responsible for converting the IR to an AST.

use funky::engine::Engine;

#[derive(Debug)]
pub struct Program {
    pub(crate) functions: Function
}

#[derive(Debug)]
pub struct Function {
    pub(crate) name: String,
    pub(crate) instructions: Vec<Instruction>,
}

type Dest = String;
type Src = String;
type Name = usize;
type FunctionName = usize;

#[derive(Debug)]
pub enum Instruction {
    Block(String),
    Unop(Dest, Src),
    BinOp(Dest, Src, Src),
    Const(Dest, f64),
    Assign(Dest, Src),
    Jump(String),
    Call(FunctionName),
    /// kill the variable
    Kill(Dest),
}


