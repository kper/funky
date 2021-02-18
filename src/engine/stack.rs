use crate::engine::prelude::*;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum CtrlStackContent {
    Frame(Frame),
    Label(Label),
}

impl CtrlStackContent {
    pub fn is_label(&self) -> bool {
        matches!(self, CtrlStackContent::Label(_))
    }

    pub fn is_frame(&self) -> bool {
        matches!(self, CtrlStackContent::Frame(_))
    }
}

impl fmt::Display for CtrlStackContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self {
            CtrlStackContent::Label(label) => write!(f, "{:?}", label),
            CtrlStackContent::Frame(frame) => write!(f, "{:?}", frame),
        }
    }
}

/// A label is an object which
/// we can jump to in webassembly
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Label {
    arity: Arity,
    /// the `id` of the block to which the
    /// program counter has to jump for the
    /// beginning.
    start_block_id: usize,
}

impl Label {
    /// Create new label
    pub fn new(arity: Arity, block_id: usize) -> Self {
        Label { arity, start_block_id: block_id }
    }

    /// Get the arity of the label
    pub fn get_arity(&self) -> Arity {
        self.arity
    }

    /// Get the `id` of the start block
    pub fn get_start_block(&self) -> usize {
        self.start_block_id
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub arity: u32,
    pub locals: Vec<Value>,
}

impl PartialEq for Frame {
    fn eq(&self, other: &Self) -> bool {
        self.arity == other.arity && self.locals == other.locals
    }
}
