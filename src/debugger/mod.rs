use anyhow::{anyhow, Context, Result};
use log::debug;
use std::net::UdpSocket;

/// The `ProgramCounter` trait defines how a program
/// can advance.
pub trait ProgramCounter: std::fmt::Debug + Send {
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

/// This program counter does wait for an incoming udp signal before
/// it advances the program.
#[derive(Debug)]
pub struct DebuggerProgramCounter {
    pc: usize,
    socket: UdpSocket,
}

impl DebuggerProgramCounter {
    pub fn new(port: usize) -> Result<Self> {
        Ok(Self {
            pc: 0,
            socket: UdpSocket::bind(format!("127.0.0.1:{}", port))
                .context("Binding udp socket in DebuggerProgramCounter")?,
        })
    }
}

impl ProgramCounter for DebuggerProgramCounter {
    fn next_instruction(&mut self) -> Result<()> {
        debug!("Waiting for progress signal");
        // Wait for the incoming signal to proceed.
        let mut buf = [0; 1];
        let (_, _) = self.socket.recv_from(&mut buf)?; // blocking
        debug!("Progress signal received");

        let r = self
            .pc
            .checked_add(1)
            .ok_or(anyhow!("Program counter overflowed"))?;

        debug!("ip is now {:?}", r);

        Ok(())
    }
}
