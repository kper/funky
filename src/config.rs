use crate::debugger::*;
use anyhow::{Context, Result};

#[derive(Debug)]
pub struct Configuration {
    /// Does not run the program, but waits for the debugger to advance it
    debugger: bool,
}

impl Configuration {
    pub fn new() -> Self {
        Self { debugger: false }
    }

    pub fn enable_debugger(&mut self) {
        self.debugger = true;
    }

    pub fn get_program_counter(&self) -> Result<Box<dyn ProgramCounter>> {
        if self.debugger {
            Ok(Box::new(
                DebuggerProgramCounter::new(34254).context("Calling from get_program_counter")?,
            ))
        } else {
            Ok(Box::new(RelativeProgramCounter::new()))
        }
    }
}
