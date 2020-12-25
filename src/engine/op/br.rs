use wasm_parser::core::LabelIdx;
use crate::engine::InstructionOutcome;
use anyhow::Result;
use crate::engine::Engine;

impl Engine {
    pub(crate) fn br(&mut self, label_idx: &LabelIdx) -> Result<InstructionOutcome> {
        debug!("OP_BR {}", label_idx);

        return Ok(InstructionOutcome::BRANCH(*label_idx));
    }
}
