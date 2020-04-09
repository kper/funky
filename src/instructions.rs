#[derive(Debug)]
pub enum Instruction {
    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
    F64Const(f64),
    IAdd,
    IMul,
    End,
}
