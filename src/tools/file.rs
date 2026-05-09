use async_trait::async_trait;
use std::path::{PathBuf};
use crate::config::FileToolConfig;
use crate::error::{CynapseError, Result};
use super::Tool;

pub struct FileTool {
    working_dir: PathBuf,
    max_file_size_bytes: usize,
}

impl FileTool {
    pub fn new(config: &FileToolConfig) -> Result<Self> {
        Ok(Self {
            working_dir: PathBuf::from(&config.working_dir),
            max_file_size_bytes: config.max_file_size_mb * 1024 * 1024,
        })
    }
    
    fn resolve_path(&self, path: &str) -> Result<PathBuf> {
        let resolved = self.working_dir.join(path);
        let canonical = resolved.canonicalize().unwrap_or(resolved);
        
        // Security check - ensure path is within working directory
        if !canonical.starts_with(&self.working_dir) {
            return Err(CynapseError::ToolError(
                "Path escapes working directory".to_string()
            ));
        }
        
        Ok(canonical)
    }
}

#[async_trait]
impl Tool for FileTool {
    fn name(&self) -> &str {
        "file"
    }
    
    fn description(&self) -> &str {
        "Read, write, or list files in the working directory"
    }
    
    async fn execute(&self, args: &str) -> Result<String> {
        let parts: Vec<&str> = args.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(CynapseError::ToolError(
                "Invalid format. Use 'read:path', 'write:path:content', or 'list:path'".to_string()
            ));
        }
        
        let command = parts[0];
        let rest = parts[1];
        
        match command {
            "read" => {
                let path = self.resolve_path(rest)?;
                
                // Check file size
                let metadata = tokio::fs::metadata(&path).await?;
                if metadata.len() as usize > self.max_file_size_bytes {
                    return Err(CynapseError::ToolError(
                        format!("File too large (max {}MB)", self.max_file_size_bytes / 1024 / 1024)
                    ));
                }
                
                let content = tokio::fs::read_to_string(&path).await?;
                Ok(content)
            }
            
            "write" => {
                let write_parts: Vec<&str> = rest.splitn(2, ':').collect();
                if write_parts.len() != 2 {
                    return Err(CynapseError::ToolError(
                        "Write format: 'write:path:content'".to_string()
                    ));
                }
                
                let path = self.resolve_path(write_parts[0])?;
                let content = write_parts[1];
                
                // Create parent directory if needed
                if let Some(parent) = path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                
                tokio::fs::write(&path, content).await?;
                Ok(format!("Wrote {} bytes to {}", content.len(), path.display()))
            }
            
            "list" => {
                let path = self.resolve_path(rest)?;
                let mut entries = tokio::fs::read_dir(&path).await?;
                let mut files = Vec::new();
                
                while let Some(entry) = entries.next_entry().await? {
                    files.push(entry.file_name().to_string_lossy().to_string());
                }
                
                Ok(files.join("\n"))
            }
            
            _ => Err(CynapseError::ToolError(
                format!("Unknown command: {}. Use read, write, or list", command)
            )),
        }
    }
}
