//! Heartbeat Curator — background memory consolidation.
//!
//! Periodically reviews recent conversation logs and asks the LLM
//! to extract important facts, preferences, and decisions into
//! DENDRITE memory nodes.

use crate::llm::{LLMProvider, Message};
use crate::memory::{Memory, HybridMemory};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

pub struct Curator {
    device_id: String,
    memory: Arc<Mutex<HybridMemory>>,
    interval_hours: u64,
}

impl Curator {
    pub fn new(device_id: String, memory: Arc<Mutex<HybridMemory>>, interval_hours: u64) -> Self {
        Self {
            device_id,
            memory,
            interval_hours: interval_hours.max(1),
        }
    }

    /// Start the background curation loop.
    pub fn start(self, llm: Arc<dyn LLMProvider + Send + Sync>) {
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(self.interval_hours * 3600));
            ticker.tick().await; // First tick fires immediately, skip it

            loop {
                ticker.tick().await;
                if let Err(e) = self.run_maintenance(llm.as_ref()).await {
                    tracing::error!("[CURATOR] Maintenance error: {}", e);
                }
            }
        });
    }

    /// Run a single maintenance pass. Can also be called manually.
    pub async fn run_maintenance(&self, llm: &dyn LLMProvider) -> crate::error::Result<()> {
        tracing::info!("[CURATOR] Running heartbeat for {}", self.device_id);

        // Get recent messages
        let recent = {
            let mem = self.memory.lock().await;
            mem.get_history(&self.device_id, 30).await?
        };

        if recent.len() < 3 {
            tracing::info!("[CURATOR] Not enough history, skipping");
            return Ok(());
        }

        // Build conversation text
        let mut conversation = String::new();
        for msg in &recent {
            let role = match msg.role {
                crate::llm::Role::User => "User",
                crate::llm::Role::Assistant => "Assistant",
                crate::llm::Role::Tool => "Tool",
                crate::llm::Role::System => continue, // Skip system messages
            };
            conversation.push_str(&format!("{}: {}\n", role, msg.content));
        }

        // Ask LLM to extract facts
        let prompt = format!(
            "Review this conversation and extract important facts worth saving to long-term memory.\
            Focus on: user preferences, decisions made, project details, recurring topics.\
            \
            Respond in JSON only:\
            {{\"facts\": [{{\"content\": \"...\", \"tags\": \"tag1,tag2\"}}]}}\
            \
            If nothing is worth saving, respond: {{\"facts\": []}}\
            \
            Conversation:\
            {}",
            conversation
        );

        let response = llm
            .complete(vec![
                Message::system("You are a memory curator. Extract facts in JSON only."),
                Message::user(&prompt),
            ])
            .await?;

        // Parse JSON response
        #[derive(serde::Deserialize)]
        struct FactList {
            facts: Vec<FactEntry>,
        }

        #[derive(serde::Deserialize)]
        struct FactEntry {
            content: String,
            tags: String,
        }

        let content = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let fact_list: FactList = match serde_json::from_str(content) {
            Ok(list) => list,
            Err(e) => {
                tracing::warn!("[CURATOR] Failed to parse fact JSON: {}", e);
                return Ok(());
            }
        };

        // Save facts
        let fact_count = fact_list.facts.len();
        let mut mem = self.memory.lock().await;
        for fact in fact_list.facts {
            let tags: Vec<String> = fact
                .tags
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();

            if let Err(e) = mem.save_fact(&fact.content, &tags) {
                tracing::warn!("[CURATOR] Failed to save fact: {}", e);
            } else {
                tracing::info!(
                    "[CURATOR] Saved fact: {}",
                    &fact.content[..fact.content.len().min(60)]
                );
            }
        }

        // Also compact session history
        if let Err(e) = mem.compact(&self.device_id, None) {
            tracing::warn!("[CURATOR] Compaction failed: {}", e);
        }

        tracing::info!("[CURATOR] Heartbeat complete. Saved {} facts.", fact_count);
        Ok(())
    }
}
