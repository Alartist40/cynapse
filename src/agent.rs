use futures::StreamExt;
use crate::config::Config;
use crate::error::{Result};
// use crate::error::{CynapseError, Result};
use crate::llm::{self, Message, LLMProvider};
use crate::memory::{self, Memory};
use crate::tools::{self, ToolRegistry};

pub struct Agent {
    device_id: String,
    system_prompt: String,
    llm: Box<dyn LLMProvider>,
    memory: Box<dyn Memory>,
    tools: ToolRegistry,
}

impl Agent {
    /// Create a new agent from configuration
    pub fn new(config: &Config) -> Result<Self> {
        let llm = llm::create_provider(&config.llm)?;
        let memory = memory::create_memory(&config.memory)?;
        let tools = tools::create_tools(&config.tools)?;
        
        Ok(Self {
            device_id: config.agent.device_id.clone(),
            system_prompt: config.agent.system_prompt.clone(),
            llm,
            memory,
            tools,
        })
    }
    
    /// Process a user message and return response
    pub async fn process(&mut self, user_input: &str) -> Result<String> {
        // 1. Load conversation history
        let mut history = self.memory.get_history(&self.device_id, 20).await?;
        
        // 2. Add system prompt if history is empty
        if history.is_empty() {
            history.push(Message::system(&self.system_prompt));
        }
        
        // 3. Add user message
        let user_msg = Message::user(user_input);
        self.memory.save_message(&self.device_id, user_msg.clone()).await?;
        history.push(user_msg);
        
        // 4. Get LLM response
        let response = self.llm.complete(history).await?;
        
        // 5. Save assistant response
        let assistant_msg = Message::assistant(&response);
        self.memory.save_message(&self.device_id, assistant_msg).await?;
        
        Ok(response)
    }
    
    /// Process a user message with streaming output
    pub async fn process_stream<F>(&mut self, user_input: &str, mut on_chunk: F) -> Result<String>
    where
        F: FnMut(&str) + Send,
    {
        // 1. Load conversation history
        let mut history = self.memory.get_history(&self.device_id, 20).await?;
        
        // 2. Add system prompt if history is empty
        if history.is_empty() {
            history.push(Message::system(&self.system_prompt));
        }
        
        // 3. Add user message
        let user_msg = Message::user(user_input);
        self.memory.save_message(&self.device_id, user_msg.clone()).await?;
        history.push(user_msg);
        
        // 4. Stream LLM response
        let mut stream = self.llm.stream(history).await?;
        let mut full_response = String::new();
        
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(text) => {
                    on_chunk(&text);
                    full_response.push_str(&text);
                }
                Err(e) => return Err(e),
            }
        }
        
        // 5. Save assistant response
        let assistant_msg = Message::assistant(&full_response);
        self.memory.save_message(&self.device_id, assistant_msg).await?;
        
        Ok(full_response)
    }
    
    /// Execute a tool
    pub async fn execute_tool(&self, tool_name: &str, args: &str) -> Result<String> {
        self.tools.execute(tool_name, args).await
    }
    
    /// List available tools
    pub fn list_tools(&self) -> Vec<String> {
        self.tools.list()
    }
    
    /// Clear conversation history
    pub async fn clear_history(&mut self) -> Result<()> {
        self.memory.clear_history(&self.device_id).await
    }
}
