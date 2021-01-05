use crate::engine::stack::StackContent;
use crate::value::Value;
use anyhow::Result;
use log::debug;
use std::fmt;
use std::sync::mpsc::{Receiver, Sender};

/// It has the same function as `ProgramState`,
/// however it has borrowed members.
/// The reason for this is performance. The `set_pc` function
/// for the `DebuggerProgramCounter` needs the state, but the
/// `RelativeProgramCounter` does not. Therefore, we should avoid allocations.
pub struct BorrowedProgramState<'a> {
    current_pc: usize,
    stack: &'a [StackContent],
    locals: &'a [Value]
}

impl<'a> BorrowedProgramState<'a> {
    pub fn new(current_pc: usize, stack: &'a [StackContent], locals: &'a [Value]) -> Self {
        Self {
            current_pc,
            stack,
            locals,
        }
    }
}

impl<'a> Into<ProgramState> for BorrowedProgramState<'a> {
    fn into(self) -> ProgramState {
        ProgramState {
            current_pc: self.current_pc,
            stack: self.stack.to_vec(),
            locals: self.locals.to_vec()
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProgramState {
    current_pc: usize,
    stack: Vec<StackContent>,
    locals: Vec<Value>,
}

impl ProgramState {
    pub fn new(current_pc: usize, stack: Vec<StackContent>, locals: Vec<Value>) -> Self {
        Self {
            current_pc,
            stack,
            locals,
        }
    }

    pub fn get_pc(&self) -> usize {
        self.current_pc
    }
}

impl fmt::Display for ProgramState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let elements: Vec<_> = self.stack.iter().map(|w| format!("{}", w)).collect();

        write!(
            f,
            "Current pc {}\n Stack: \n{:#?}\n Locals: \n {:#?}",
            self.current_pc, elements, self.locals
        )
    }
}

/// The `ProgramCounter` trait defines how a program
/// can advance.
pub trait ProgramCounter: std::fmt::Debug + Send {
    /// we set the program pointer to
    /// a new value.
    fn set_pc<'a>(&mut self, n: BorrowedProgramState<'a>) -> Result<()>;
}

/// The default program counter because it doesn't hold.
/// It is relative because it doesn't keep up with
/// function calls or nested blocks.
#[derive(Debug, Default)]
pub struct RelativeProgramCounter(usize);

impl ProgramCounter for RelativeProgramCounter {
    fn set_pc<'a>(&mut self, n: BorrowedProgramState<'a>) -> Result<()> {
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
    fn set_pc<'a>(&mut self, state: BorrowedProgramState<'a>) -> Result<()> {
        debug!("Waiting for progress signal");

        self.instruction_advancer.recv().unwrap();

        debug!("Progress signal received");

        debug!("Setting pc to {}", state.current_pc);
        self.pc = state.current_pc;

        // we defered the `clone` until here
        self.instruction_watcher.send(state.into()).unwrap();

        Ok(())
    }
}
