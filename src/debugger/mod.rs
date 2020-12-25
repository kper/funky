use anyhow::{anyhow, Result};
use log::debug;

/// The `ProgramCounter` trait defines how a program
/// can advance.
pub trait ProgramCounter : std::fmt::Debug {
    fn next_instruction(&mut self) -> Result<()>;
}

/// The default program counter because it doesn't hold.
/// It is relative because it doesn't keep up with
/// function calls or nested blocks.
#[derive(Debug)]
pub struct RelativeProgramCounter(usize);

impl RelativeProgramCounter {
    pub fn new() -> Self {
        RelativeProgramCounter(0)
    }
}

impl ProgramCounter for RelativeProgramCounter {
    fn next_instruction(&mut self) -> Result<()> {
        let r = self
            .0
            .checked_add(1)
            .ok_or(anyhow!("Program counter overflowed"))?;

        debug!("ip is now {:?}", r);

        Ok(())
    }
}
