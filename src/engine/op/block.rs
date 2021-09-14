use crate::engine::stack::{Frame, Label, StackContent};
use crate::engine::Engine;
use crate::engine::InstructionOutcome;
use crate::value::Arity;
use anyhow::{anyhow, Result};
use wasm_parser::core::{BlockType, CodeBlock};

impl Engine {
    pub(crate) fn block(
        &mut self,
        fr: &mut Frame,
        ty: &BlockType,
        block_instructions: &CodeBlock,
    ) -> Result<InstructionOutcome> {
        debug!("OP_BLOCK {:?}", ty);

        let arity_return_count = self.get_return_count_block(&ty)?;

        self.setup_stack_for_entering_block(ty, block_instructions, arity_return_count)?;

        let outcome = self.run_instructions(fr, &mut block_instructions.iter())?;

        Ok(outcome)
    }

    pub(crate) fn setup_stack_for_entering_block(
        &mut self,
        ty: &BlockType,
        block: &CodeBlock,
        arity: u32,
    ) -> Result<()> {
        let param_count = self.get_mut_param_count_block(&ty)?;
        let return_count = self.get_return_count_block(&ty)?;

        debug!("Arity for block ({:?}) is {}", ty, return_count);
        debug!("Stack size is {}", self.store.stack.len());

        let label = Label::new(arity, block.id);

        debug!("=> stack {:#?}", self.store.stack);

        // Extracting the parameters of the stack
        let mut block_args = self.get_mut_stack_elements_entering_block(param_count);

        // Pushing the label
        self.store.stack.push(StackContent::Label(label));

        // Pushing arguments back on the stack
        self.store.stack.append(&mut block_args);

        Ok(())
    }

    /// We need to enter block which expects parameters.
    /// We extract `arity` of stack.
    fn get_mut_stack_elements_entering_block(&mut self, param_count: u32) -> Vec<StackContent> {
        debug!(
            "For entering a block, popping off parameters {}",
            param_count
        );

        self
            .store
            .stack
            .split_off(self.store.stack.len() - param_count as usize)
    }

    /// By given block_ty, return the param count of the block
    pub fn get_mut_param_count_block(&mut self, block_ty: &BlockType) -> Result<Arity> {
        let arity = match block_ty {
            BlockType::Empty => 0,
            BlockType::ValueType(_) => 0,
            BlockType::FuncTy(ty) => self
                .module_instance
                .lookup_func_types(ty)
                .ok_or_else(|| anyhow!("Cannot find func type"))?
                .param_types
                .len(),
        };

        debug!("Arity is {}", arity);

        Ok(arity as u32)
    }

    /// By given block_ty, return the return count of the block
    pub fn get_return_count_block(&self, block_ty: &BlockType) -> Result<Arity> {
        let arity = match block_ty {
            BlockType::Empty => 0,
            BlockType::ValueType(_) => 1,
            BlockType::FuncTy(ty) => self
                .module_instance
                .lookup_func_types(ty)
                .ok_or_else(|| anyhow!("Cannot find func type"))?
                .return_types
                .len(),
        };

        debug!("Arity is {}", arity);

        Ok(arity as u32)
    }
}
