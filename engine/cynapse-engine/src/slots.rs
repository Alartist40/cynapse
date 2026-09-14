//! SlotManager for llama-server KV-cache pinning.
//!
//! Pinned slots:
//! - Slot 0: Interactive user chat session (never evicted by background tasks)
//! - Slot 1: Out-of-band memory reflection and procedural distillation

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotPurpose {
    Interactive,
    Reflection,
}

impl SlotPurpose {
    pub fn id(&self) -> u32 {
        match self {
            SlotPurpose::Interactive => 0,
            SlotPurpose::Reflection => 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SlotManager {
    reflection_busy: Arc<AtomicBool>,
}

impl Default for SlotManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SlotManager {
    pub fn new() -> Self {
        Self {
            reflection_busy: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Acquires the reflection slot if available. Returns false if already busy.
    pub fn try_acquire_reflection(&self) -> bool {
        self.reflection_busy
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    /// Releases the reflection slot.
    pub fn release_reflection(&self) {
        self.reflection_busy.store(false, Ordering::SeqCst);
    }

    /// Returns default completion params for slot pinning:
    /// `slot_id`, `id_slot`, and `cache_prompt: true`.
    pub fn slot_params(purpose: SlotPurpose) -> (u32, bool) {
        (purpose.id(), true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_allocation_and_release() {
        let mgr = SlotManager::new();
        assert_eq!(SlotPurpose::Interactive.id(), 0);
        assert_eq!(SlotPurpose::Reflection.id(), 1);

        assert!(mgr.try_acquire_reflection());
        // Second concurrent acquisition should fail
        assert!(!mgr.try_acquire_reflection());

        mgr.release_reflection();
        assert!(mgr.try_acquire_reflection());
    }
}
