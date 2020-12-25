use crate::debugger::*;

#[derive(Debug)]
pub struct Configuration {
    /// Does not run the program, but waits for the debugger to advance it
    debugger: bool
}

impl Configuration {
    pub fn new() -> Self {
        Self {
            debugger: false
        }
    }

    pub fn enable_debugger(&mut self) {
        self.debugger = true;
    }

    pub fn get_program_counter(&self) -> impl ProgramCounter {
        if self.debugger {
            RelativeProgramCounter::new()
        }
        else {
            RelativeProgramCounter::new()
        }
    }
}
