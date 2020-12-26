use crate::engine::stack::{Frame, Label, StackContent};
use crate::engine::Engine;
use crate::engine::InstructionOutcome;
use anyhow::{Result};
use wasm_parser::core::{BlockType, CodeBlock};

impl Engine {
    pub(crate) fn block(
        &mut self,
        fr: &mut Frame,
        ty: &BlockType,
        block_instructions: &CodeBlock,
    ) -> Result<InstructionOutcome> {
        debug!("OP_BLOCK {:?}", ty);

        let arity = self.get_block_ty_arity(&ty)?;

        debug!("Arity for block ({:?}) is {}", ty, arity);

        let label = Label::new(arity);

        self.store.stack.push(StackContent::Label(label));

        let outcome = self.run_instructions(fr, &mut block_instructions.iter())?;

        Ok(outcome)
    }
}
