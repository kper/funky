mod unit_tests;
mod wasm;

use wasm_parser::core::*;

pub struct UnannotatedCodeBlock {
    instructions: Vec<Instruction>,
}

impl UnannotatedCodeBlock {
    /// Creates an `UnannotatedCodeBlock`.
    /// This is like a `CodeBlock` but without a `id`
    pub fn wrap(instructions: Vec<Instruction>) -> Self {
        Self { instructions }
    }

    /// Convert to a real `CodeBlock`
    pub fn annotate(self, counter: &mut Counter) -> CodeBlock {
        CodeBlock::new(counter, self.instructions)
    }
}
