use anyhow::{anyhow, Context, Result};
use log::debug;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};

/// The `ProgramCounter` trait defines how a program
/// can advance.
pub trait ProgramCounter: std::fmt::Debug + Send {
    /// we set the program pointer to
    /// a new value.
    fn set_pc(&mut self, n: usize) -> Result<()>;
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
    fn set_pc(&mut self, n: usize) -> Result<()> {
        debug!("Setting pc to {}", n);
        self.0 = n;

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
    instruction_watcher: Sender<usize>,
    /// Informs the receiver if it can proceed one step
    /// The runtime will own the receiver
    instruction_advancer: Receiver<()>,
}

impl DebuggerProgramCounter {
    pub fn new(watcher: Sender<usize>, advancer: Receiver<()>) -> Result<Self> {
        Ok(Self {
            pc: 0,
            instruction_watcher: watcher,
            instruction_advancer: advancer,
            //socket: UdpSocket::bind(format!("127.0.0.1:{}", port))
            //.context("Binding udp socket in DebuggerProgramCounter")?,
        })
    }
}

impl ProgramCounter for DebuggerProgramCounter {
    fn set_pc(&mut self, n: usize) -> Result<()> {
        debug!("Waiting for progress signal");
        // Wait for the incoming signal to proceed.
        //let mut buf = [0; 1];
        //let (_, _) = self.socket.recv_from(&mut buf)?; // blocking

        self.instruction_advancer.recv().unwrap();

        debug!("Progress signal received");

        debug!("Setting pc to {}", n);
        self.pc = n;

        self.instruction_watcher.send(n).unwrap();

        Ok(())
    }
}
