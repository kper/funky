use crate::engine::stack::{Frame, StackContent};
use crate::engine::Engine;
use crate::engine::InstructionOutcome;
use anyhow::{bail, Context, Result};

impl Engine {
    pub(crate) fn br(&mut self, _fr: &mut Frame, label_idx: u32) -> Result<InstructionOutcome> {
        debug!("OP_BR {}", label_idx);

        debug!("stack {:#?}", self.store.stack);

        let labels: Vec<_> = self
            .store
            .stack
            .iter()
            .filter(|x| matches!(x, StackContent::Label(_)))
            .collect();

        debug!(
            "Getting label at {}",
            labels.len() as isize - 1 - label_idx as isize
        );
        let label = labels
            .get(labels.len() - 1 - label_idx as usize)
            .map(|x| x.clone());

        if let Some(StackContent::Label(lb)) = label {
            let arity = lb.get_arity();
            debug!("The arity of the label is {}", arity);

            // Pop off the `arity` elements off the stack.
            let mut return_elements = self
                .store
                .pop_off_stack(arity as usize)
                .context("Failed when fetching return elements")?;

            for _ in 0..(label_idx + 1) {
                while matches!(self.store.stack.last(), Some(StackContent::Value(_))) {
                    debug!("Popping off junk");
                    self.store.stack.pop();
                }

                if let Some(StackContent::Label(_)) = self.store.stack.pop() {
                    debug!("Popping off label");
                } else {
                    bail!("Popping off label failed");
                }
            }

            self.store.stack.append(&mut return_elements);
        } else {
            bail!("Label not found");
        }

        return Ok(InstructionOutcome::BRANCH(label_idx));
    }
}
