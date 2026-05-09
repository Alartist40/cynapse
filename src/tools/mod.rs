use async_trait::async_trait;
// use serde::{Deserialize, Serialize};
use crate::error::Result;

/// Tool trait - abstraction for executable capabilities
#[async_trait]
pub trait Tool: Send + Sync {
    /// Tool name
    fn name(&self) -> &str;
    
    /// Tool description for LLM
    fn description(&self) -> &str;
    
    /// Execute the tool with given arguments
    async fn execute(&self, args: &str) -> Result<String>;
}

/// Tool registry manages available tools
pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }
    
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }
    
    pub fn get(&self, name: &str) -> Option<&Box<dyn Tool>> {
        self.tools.iter().find(|t| t.name() == name)
    }
    
    pub fn list(&self) -> Vec<String> {
        self.tools.iter().map(|t| t.name().to_string()).collect()
    }
    
    pub async fn execute(&self, name: &str, args: &str) -> Result<String> {
        match self.get(name) {
            Some(tool) => tool.execute(args).await,
            None => Err(crate::error::CynapseError::ToolError(
                format!("Tool '{}' not found", name)
            )),
        }
    }
}

pub mod bash;
pub mod memory;
pub mod file;

use crate::config::ToolsConfig;

/// Create tool registry from configuration
pub fn create_tools(config: &ToolsConfig) -> Result<ToolRegistry> {
    let mut registry = ToolRegistry::new();
    
    for tool_name in &config.enabled {
        match tool_name.as_str() {
            "bash" => registry.register(Box::new(bash::BashTool::new(&config.bash))),
            "memory" => registry.register(Box::new(memory::MemoryTool::new())),
            "file" => registry.register(Box::new(file::FileTool::new(&config.file)?)),
            _ => tracing::warn!("Unknown tool: {}", tool_name),
        }
    }
    
    Ok(registry)
}
