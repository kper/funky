use crate::engine::Page;
use std::fmt;

#[derive(Clone)]
pub struct MemoryInstance {
    pub data: Vec<u8>,
    pub max: Option<u32>,
}

// Overwritten debug implementation
// Because `data` can have a lot of entries, which
// can be a problem when printing
impl fmt::Debug for MemoryInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MemoryInstance")
            .field("data (only length)", &self.data.len())
            .field("max", &self.max)
            .finish()
    }
}

/// Returns Err when paging failed
/// Returns new length in pages
/// https://webassembly.github.io/spec/core/exec/modules.html#growing-memories
pub(crate) fn grow_memory(instance: &mut MemoryInstance, n: Page) -> Result<Page, ()> {
    if n.is_zero() {
        return Ok(Page::from_count(instance.data.len()));
    }

    let len = n + Page::from_count(instance.data.len());

    if len.pages() > usize::pow(2, 16) {
        error!("Length exceeded. Too many memory pages");
        return Err(());
    }

    if let Some(max) = instance.max {
        debug!("Checking limit len {:?} < max {}", len.pages(), max);
        if len.pages() > max as usize {
            error!("Memory growing failed. Limit exceded");
            return Err(());
        }
    }

    let new_length = Page::from_count(instance.data.len()) + n;
    debug!("Resize by {} bytes", new_length.elements());

    // Create new vec and fill it with 0u8
    let extension = vec![0u8; n.elements()];

    // Append the new vec to the instance
    instance.data.extend_from_slice(&extension);

    // Return pages
    Ok(new_length)
}
