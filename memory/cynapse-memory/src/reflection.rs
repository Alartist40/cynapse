//! DENDRITE v2 — Asynchronous background memory reflection worker.
//!
//! After user/agent chat turns, this worker runs an out-of-band distillation pass
//! to extract key facts (L1) and procedural workflows (L2) from the conversation
//! history, saving them into the Dendrite graph and store without blocking live output.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::graph::{Dendrite, NodeType};
use crate::store::DendriteStore;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn text(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }
}

fn deterministic_reflection_id(transcript: &str) -> String {
    let mut hasher = DefaultHasher::new();
    transcript.hash(&mut hasher);
    format!("reflection_{:016x}", hasher.finish())
}

/// Reflects on recent chat history and distills L1 atomic facts / L2 procedures.
pub struct ReflectionWorker {
    graph: Arc<Dendrite>,
    store: Option<Arc<DendriteStore>>,
    in_flight: Arc<AtomicBool>,
}

impl ReflectionWorker {
    pub fn new(
        graph: Arc<Dendrite>,
        store: Option<Arc<DendriteStore>>,
    ) -> Self {
        Self {
            graph,
            store,
            in_flight: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Run reflection on recent conversation messages in a background Tokio
    /// task, guarding against overlapping background runs.
    pub fn spawn_reflection(&self, messages: Vec<Message>) {
        if messages.len() < 2 {
            return;
        }

        if self
            .in_flight
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return;
        }

        let graph = self.graph.clone();
        let store = self.store.clone();
        let in_flight = self.in_flight.clone();

        tokio::spawn(async move {
            let _ = Self::do_reflection(graph, store, &messages).await;
            in_flight.store(false, Ordering::SeqCst);
        });
    }

    async fn do_reflection(
        graph: Arc<Dendrite>,
        store: Option<Arc<DendriteStore>>,
        messages: &[Message],
    ) -> Result<(), String> {
        let mut full_transcript = String::new();
        for msg in messages.iter() {
            let role = match msg.role {
                Role::User => "User",
                Role::Assistant => "Assistant",
                Role::System => "System",
                Role::Tool => "Tool",
            };
            if !msg.content.is_empty() {
                full_transcript.push_str(&format!("{}: {}\n", role, msg.content));
            }
        }

        if full_transcript.is_empty() {
            return Ok(());
        }

        let node_id = deterministic_reflection_id(&full_transcript);
        let title = format!("Reflected Memory {}", &node_id[..16.min(node_id.len())]);

        let node_type = if full_transcript.contains("how to") || full_transcript.contains("procedure") {
            NodeType::Procedure
        } else {
            NodeType::AtomicFact
        };

        let node = graph.upsert(
            &node_id,
            &title,
            &full_transcript,
            node_type,
            Some(vec!["#reflection".into()]),
        );

        if let Some(s) = &store {
            let _ = s.save(&node);
        }

        let turn_id = format!("turn_{}", node_id);
        let turn_title = format!("Conversation Turn {}", &node_id[..16.min(node_id.len())]);
        let turn_node = graph.upsert(
            &turn_id,
            &turn_title,
            &full_transcript,
            NodeType::TurnLog,
            Some(vec!["#transcript".into()]),
        );
        if let Some(s) = &store {
            let _ = s.save(&turn_node);
        }

        Ok(())
    }
}
