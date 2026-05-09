use clap::{Parser, Subcommand};
use cynapse_mini::{Agent, Config};
use std::io::{self, Write};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "CYNAPSE Mini")]
#[command(about = "Lightweight Rust AI Agent for Embedded Systems", version)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, default_value = "config.yaml")]
    config: String,
    
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
    
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start interactive chat
    Chat,
    
    /// Execute a single query
    Query {
        /// The query to send to the agent
        query: String,
    },
    
    /// Initialize default configuration
    Init,
    
    /// Show version information
    Version,
    
    /// Clear conversation history
    Clear,
    
    /// List available tools
    Tools,
}

#[tokio::main]
async fn main() -> cynapse_mini::Result<()> {
    let cli = Cli::parse();
    
    // Setup logging
    let filter = if cli.debug {
        EnvFilter::new("cynapse_mini=debug")
    } else {
        EnvFilter::new("cynapse_mini=info")
    };
    
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();
    
    match cli.command {
        Some(Commands::Init) => {
            let config = Config::default();
            config.save(&cli.config)?;
            println!("Created default configuration at: {}", cli.config);
            Ok(())
        }
        
        Some(Commands::Version) => {
            println!("CYNAPSE Mini v{}", env!("CARGO_PKG_VERSION"));
            println!("Rust AI Agent for Embedded Systems");
            Ok(())
        }
        
        Some(Commands::Chat) => {
            let config = Config::load(&cli.config)?;
            let mut agent = Agent::new(&config)?;
            
            println!("🦀 CYNAPSE Mini - Interactive Chat");
            println!("Type 'quit' or 'exit' to stop\n");
            
            loop {
                print!("> ");
                io::stdout().flush()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let input = input.trim();
                
                if input.is_empty() {
                    continue;
                }
                
                if input == "quit" || input == "exit" {
                    break;
                }
                
                // Stream response
                print!("Assistant: ");
                io::stdout().flush()?;
                
                agent.process_stream(input, |chunk| {
                    print!("{}", chunk);
                    io::stdout().flush().ok();
                }).await?;
                
                println!("\n");
            }
            
            println!("Goodbye!");
            Ok(())
        }
        
        Some(Commands::Query { query }) => {
            let config = Config::load(&cli.config)?;
            let mut agent = Agent::new(&config)?;
            
            let response = agent.process(&query).await?;
            println!("{}", response);
            Ok(())
        }
        
        Some(Commands::Clear) => {
            let config = Config::load(&cli.config)?;
            let mut agent = Agent::new(&config)?;
            
            agent.clear_history().await?;
            println!("Conversation history cleared");
            Ok(())
        }
        
        Some(Commands::Tools) => {
            let config = Config::load(&cli.config)?;
            let agent = Agent::new(&config)?;
            
            println!("Available tools:");
            for tool in agent.list_tools() {
                println!("  - {}", tool);
            }
            Ok(())
        }
        
        None => {
            // Default to chat if no command specified
            let config = match Config::load(&cli.config) {
                Ok(c) => c,
                Err(_) => {
                    println!("Config file not found. Run with 'init' to create one.");
                    return Ok(());
                }
            };
            let mut agent = Agent::new(&config)?;
            
            println!("🦀 CYNAPSE Mini");
            println!("Type 'quit' to exit\n");
            
            loop {
                print!("> ");
                io::stdout().flush()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let input = input.trim();
                
                if input.is_empty() {
                    continue;
                }
                
                if input == "quit" || input == "exit" {
                    break;
                }
                
                print!("Assistant: ");
                io::stdout().flush()?;
                
                agent.process_stream(input, |chunk| {
                    print!("{}", chunk);
                    io::stdout().flush().ok();
                }).await?;
                
                println!("\n");
            }
            
            Ok(())
        }
    }
}
