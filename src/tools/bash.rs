use async_trait::async_trait;
use std::process::Stdio;
use tokio::process::Command;
use crate::config::BashToolConfig;
use crate::error::{CynapseError, Result};
use super::Tool;

pub struct BashTool {
    working_dir: String,
    timeout_seconds: u64,
}

impl BashTool {
    pub fn new(config: &BashToolConfig) -> Self {
        Self {
            working_dir: config.working_dir.clone(),
            timeout_seconds: config.timeout_seconds,
        }
    }
}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }
    
    fn description(&self) -> &str {
        "Execute bash commands in the working directory"
    }
    
    async fn execute(&self, args: &str) -> Result<String> {
        tracing::info!("Executing bash command: {}", args);
        
        // Safety check - don't allow certain dangerous commands
        if args.contains("rm -rf /") || args.contains(":(){ :|:& };:") {
            return Err(CynapseError::ToolError(
                "Dangerous command blocked".to_string()
            ));
        }
        
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(self.timeout_seconds),
            Command::new("sh")
                .arg("-c")
                .arg(args)
                .current_dir(&self.working_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
        )
        .await
        .map_err(|_| CynapseError::ToolError("Command timeout".to_string()))??;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        if !output.status.success() {
            Ok(format!("Exit code: {}\nStderr: {}\nStdout: {}", 
                      output.status.code().unwrap_or(-1),
                      stderr,
                      stdout))
        } else {
            Ok(stdout.to_string())
        }
    }
}
