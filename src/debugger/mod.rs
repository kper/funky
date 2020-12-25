use anyhow::{anyhow, Context, Result};
use log::debug;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use crate::engine::StackContent;

#[derive(Debug)]
pub struct ProgramState {
    current_pc: usize,
    stack: Vec<StackContent>,
}

impl ProgramState {
    pub fn new(current_pc: usize, stack: Vec<StackContent>) -> Self {
        Self { current_pc, stack }
    }
}

/// The `ProgramCounter` trait defines how a program
/// can advance.
pub trait ProgramCounter: std::fmt::Debug + Send {
    /// we set the program pointer to
    /// a new value.
    fn set_pc(&mut self, n: ProgramState) -> Result<()>;
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
    fn set_pc(&mut self, n: ProgramState) -> Result<()> {
        debug!("Setting pc to {}", n.current_pc);
        self.0 = n.current_pc;

        Ok(())
    }
}

/// This program counter does wait for an incoming udp signal before
/// it advances the program.
#[derive(Debug)]
pub struct DebuggerProgramCounter {
    pc: usize,
    /// Informs the receiver what is the next `instruction_id` program counter
    /// `hustensaft` will own the receiver
    instruction_watcher: Sender<ProgramState>,
    /// Informs the receiver if it can proceed one step
    /// The runtime will own the receiver
    instruction_advancer: Receiver<()>,
}

impl DebuggerProgramCounter {
    pub fn new(watcher: Sender<ProgramState>, advancer: Receiver<()>) -> Result<Self> {
        Ok(Self {
            pc: 0,
            instruction_watcher: watcher,
            instruction_advancer: advancer,
        })
    }
}

impl ProgramCounter for DebuggerProgramCounter {
    fn set_pc(&mut self, state: ProgramState) -> Result<()> {
        debug!("Waiting for progress signal");

        self.instruction_advancer.recv().unwrap();

        debug!("Progress signal received");

        debug!("Setting pc to {}", state.current_pc);
        self.pc = state.current_pc;

        self.instruction_watcher.send(state).unwrap();

        Ok(())
    }
}
