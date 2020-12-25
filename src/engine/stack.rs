use crate::engine::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum StackContent {
    Value(Value),
    Frame(Frame),
    Label(Label),
}

impl StackContent {
    pub fn is_value(&self) -> bool {
        match self {
            StackContent::Value(_) => true,
            _ => false,
        }
    }

    pub fn is_label(&self) -> bool {
        match self {
            StackContent::Label(_) => true,
            _ => false,
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
        Label {
            arity
        }
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
