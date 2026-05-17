use futures::StreamExt;
use serde::{Deserialize, Serialize};
use crate::config::Config;
use crate::error::Result;
use crate::llm::{self, Message, LLMProvider};
use crate::memory::{Memory, HybridMemory};
use crate::tools::ToolRegistry;

const MAX_TOOL_ITERATIONS: usize = 10;

/// A tool call requested by the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToolCall {
    tool: String,
    args: String,
}

pub struct Agent {
    device_id: String,
    llm: Box<dyn LLMProvider>,
    memory: HybridMemory,
    tools: ToolRegistry,
}

impl Agent {
    /// Create a new agent from configuration
    pub fn new(config: &Config) -> Result<Self> {
        let llm = llm::create_provider(&config.llm)?;
        let memory = HybridMemory::new(&config.memory)?;
        let tools = crate::tools::create_tools(&config.tools)?;
        
        Ok(Self {
            device_id: config.agent.device_id.clone(),
            llm,
            memory,
            tools,
        })
    }
    
    /// Process a user message with tool-calling loop.
    pub async fn process(&mut self, user_input: &str) -> Result<String> {
        // Save user message
        let user_msg = Message::user(user_input);
        self.memory.save_message(&self.device_id, user_msg.clone()).await?;
        
        // Build dynamic system prompt from DENDRITE
        let system_prompt = self.memory.build_system_prompt(user_input);
        
        // Run tool loop
        let mut history = vec![
            Message::system(&system_prompt),
            user_msg,
        ];
        
        let mut final_response = String::new();
        
        for _ in 0..MAX_TOOL_ITERATIONS {
            let response = self.llm.complete(history.clone()).await?;
            
            // Check if response contains a tool call
            if let Some(tool_call) = parse_tool_call(&response) {
                // Save assistant's tool request
                self.memory.save_message(&self.device_id, Message::assistant(&response)).await?;
                history.push(Message::assistant(&response));
                
                // Execute tool
                tracing::info!("Executing tool: {}({})", tool_call.tool, tool_call.args);
                let result = match self.tools.execute(&tool_call.tool, &tool_call.args).await {
                    Ok(output) => output,
                    Err(e) => format!("Error: {}", e),
                };
                
                // Show tool execution to user
                println!("\n🔧 {} → {}", tool_call.tool, result.lines().next().unwrap_or(""));
                
                // Save tool result
                let tool_msg = Message::tool(&result);
                self.memory.save_message(&self.device_id, tool_msg.clone()).await?;
                history.push(tool_msg);
            } else {
                // No tool call - this is the final response
                final_response = response;
                break;
            }
        }
        
        if final_response.is_empty() {
            final_response = "(Reached tool iteration limit)".to_string();
        }
        
        // Save assistant response
        self.memory.save_message(&self.device_id, Message::assistant(&final_response)).await?;
        
        // Compact if needed
        self.memory.compact(&self.device_id, None).ok();
        
        Ok(final_response)
    }
    
    /// Process a user message with streaming output and tool-calling loop.
    pub async fn process_stream<F>(&mut self, user_input: &str, mut on_chunk: F) -> Result<String>
    where
        F: FnMut(&str) + Send,
    {
        // Save user message
        let user_msg = Message::user(user_input);
        self.memory.save_message(&self.device_id, user_msg.clone()).await?;
        
        // Build dynamic system prompt from DENDRITE
        let system_prompt = self.memory.build_system_prompt(user_input);
        
        let mut history = vec![
            Message::system(&system_prompt),
            user_msg,
        ];
        
        let mut tool_calls_made = 0;
        
        let final_response = loop {
            if tool_calls_made >= MAX_TOOL_ITERATIONS {
                break "(Reached tool iteration limit)".to_string();
            }
            
            // Stream LLM response
            let mut stream = self.llm.stream(history.clone()).await?;
            let mut accumulated = String::new();
            
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(text) => {
                        on_chunk(&text);
                        accumulated.push_str(&text);
                    }
                    Err(e) => return Err(e),
                }
            }
            
            // Check if the accumulated response contains a tool call
            if let Some(tool_call) = parse_tool_call(&accumulated) {
                // Save assistant's tool request
                self.memory.save_message(&self.device_id, Message::assistant(&accumulated)).await?;
                history.push(Message::assistant(&accumulated));
                
                // Execute tool
                tracing::info!("Executing tool: {}({})", tool_call.tool, tool_call.args);
                let result = match self.tools.execute(&tool_call.tool, &tool_call.args).await {
                    Ok(output) => output,
                    Err(e) => format!("Error: {}", e),
                };
                
                println!("\n🔧 {} → {}", tool_call.tool, result.lines().next().unwrap_or(""));
                
                // Save tool result and continue loop
                let tool_msg = Message::tool(&result);
                self.memory.save_message(&self.device_id, tool_msg.clone()).await?;
                history.push(tool_msg);
                tool_calls_made += 1;
                
                // Print separator for next response
                print!("\nAssistant: ");
            } else {
                // No tool call - final response
                break accumulated;
            }
        };
        
        let final_response = if final_response.is_empty() {
            "(No response)".to_string()
        } else {
            final_response
        };
        
        // Save assistant response
        self.memory.save_message(&self.device_id, Message::assistant(&final_response)).await?;
        
        // Compact if needed
        self.memory.compact(&self.device_id, None).ok();
        
        Ok(final_response)
    }
    
    /// Execute a tool directly.
    pub async fn execute_tool(&self, tool_name: &str, args: &str) -> Result<String> {
        self.tools.execute(tool_name, args).await
    }
    
    /// List available tools.
    pub fn list_tools(&self) -> Vec<String> {
        self.tools.list()
    }
    
    /// Clear conversation history.
    pub async fn clear_history(&mut self) -> Result<()> {
        self.memory.clear_history(&self.device_id).await
    }
    
    /// Save a fact to long-term memory.
    pub fn save_fact(&mut self, fact: &str, tags: &[String]) -> Result<()> {
        self.memory.save_fact(fact, tags)
    }
    
    /// Get DENDRITE graph for external use (e.g., curator).
    pub fn graph(&self) -> &crate::dendrite::Dendrite {
        &self.memory.graph
    }
    
    /// Access the LLM provider for curator use.
    pub fn llm(&self) -> &dyn LLMProvider {
        self.llm.as_ref()
    }
    
    /// Access mutable memory for curator use.
    pub fn memory_mut(&mut self) -> &mut HybridMemory {
        &mut self.memory
    }
}

/// Parse a tool call from LLM response.
/// Looks for JSON like: {"tool": "bash", "args": "ls -la"}
fn parse_tool_call(response: &str) -> Option<ToolCall> {
    // Try to find JSON tool call in the response
    // The LLM might output it inline or as the sole content
    let trimmed = response.trim();
    
    // If the entire response is valid JSON, try parsing
    if let Ok(call) = serde_json::from_str::<ToolCall>(trimmed) {
        if !call.tool.is_empty() {
            return Some(call);
        }
    }
    
    // Look for JSON object in the text
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            let json = &trimmed[start..=end];
            if let Ok(call) = serde_json::from_str::<ToolCall>(json) {
                if !call.tool.is_empty() {
                    return Some(call);
                }
            }
        }
    }
    
    None
}
