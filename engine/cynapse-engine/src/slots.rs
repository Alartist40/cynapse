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
    Subagent(u32),
    Draft(u32),
}

impl SlotPurpose {
    pub fn id(&self) -> u32 {
        match self {
            SlotPurpose::Interactive => 0,
            SlotPurpose::Reflection => 1,
            SlotPurpose::Subagent(sub_id) => 2 + *sub_id,
            SlotPurpose::Draft(draft_id) => 16 + *draft_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SlotManager {
    reflection_busy: Arc<AtomicBool>,
    active_subagents: Arc<std::sync::Mutex<std::collections::HashSet<u32>>>,
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
            active_subagents: Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())),
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

    /// Allocates an available subagent slot ID (returns SlotPurpose::Subagent).
    pub fn acquire_subagent_slot(&self, max_concurrent: u32) -> Option<SlotPurpose> {
        let mut active = self.active_subagents.lock().ok()?;
        for i in 0..max_concurrent {
            if !active.contains(&i) {
                active.insert(i);
                return Some(SlotPurpose::Subagent(i));
            }
        }
        None
    }

    /// Releases an allocated subagent slot.
    pub fn release_subagent_slot(&self, purpose: SlotPurpose) {
        if let SlotPurpose::Subagent(id) = purpose {
            if let Ok(mut active) = self.active_subagents.lock() {
                active.remove(&id);
            }
        }
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

    #[test]
    fn test_subagent_dynamic_slot_allocation() {
        let mgr = SlotManager::new();
        let slot0 = mgr.acquire_subagent_slot(2).expect("slot 0");
        assert_eq!(slot0.id(), 2);

        let slot1 = mgr.acquire_subagent_slot(2).expect("slot 1");
        assert_eq!(slot1.id(), 3);

        // Cap of 2 reached
        assert!(mgr.acquire_subagent_slot(2).is_none());

        // Release slot0
        mgr.release_subagent_slot(slot0);
        let slot_reacquired = mgr.acquire_subagent_slot(2).expect("reacquired");
        assert_eq!(slot_reacquired.id(), 2);
    }
}
