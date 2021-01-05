use crate::engine::prelude::*;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum StackContent {
    Value(Value),
    Frame(Frame),
    Label(Label),
}

impl StackContent {
    pub fn is_value(&self) -> bool {
        matches!(self, StackContent::Value(_))
    }

    pub fn is_label(&self) -> bool {
        matches!(self, StackContent::Label(_))
    }
}

impl fmt::Display for StackContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self {
            StackContent::Label(label) => write!(f, "{:?}", label),
            StackContent::Frame(frame) => write!(f, "{:?}", frame),
            StackContent::Value(value) => write!(f, "{:?}", value),
        }
    }
}

/// A label is an object which
/// we can jump to in webassembly
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Label {
    arity: Arity,
}

impl Label {
    /// Create new label
    pub fn new(arity: Arity) -> Self {
        Label { arity }
    }

    /// Get the arity of the label
    pub fn get_arity(&self) -> Arity {
        self.arity
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub arity: u32,
    pub locals: Vec<Value>,
    //pub module_instance: Weak<RefCell<ModuleInstance>>,
}

impl PartialEq for Frame {
    fn eq(&self, other: &Self) -> bool {
        self.arity == other.arity && self.locals == other.locals
    }
}
