package main

import (
	"bufio"
	"context"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/Alartist40/cynapse-mini/internal/agent"
	"github.com/Alartist40/cynapse-mini/internal/config"
	"github.com/Alartist40/cynapse-mini/internal/llm"
	"github.com/Alartist40/cynapse-mini/internal/memory"
	"github.com/Alartist40/cynapse-mini/internal/synapse"
)

const version = "1.0.0-lightweight"

func main() {
	homeDir := getHomeDir()
	ensureDir(homeDir)
	ensureDir(filepath.Join(homeDir, "synapses"))
	ensureDir(filepath.Join(homeDir, "data"))

	if len(os.Args) < 2 {
		runChat()
		return
	}

	command := os.Args[1]
	args := os.Args[2:]

	switch command {
	case "synapse":
		handleSynapseCommand(args)
	case "config":
		handleConfigCommand(args)
	case "version":
		fmt.Printf("CYNAPSE Mini v%s\n", version)
	case "help":
		printHelp()
	default:
		runChat()
	}
}

func runChat() {
	cfg, err := config.Load(getConfigPath())
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error loading config: %v\n", err)
		fmt.Fprintf(os.Stderr, "Run 'cynapse-mini config init' to create default config\n")
		os.Exit(1)
	}

	store, err := memory.NewLightweightStore(filepath.Join(getHomeDir(), "data"))
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error initializing memory: %v\n", err)
		os.Exit(1)
	}

	llmClient, err := llm.New(&config.LLMConfig{
		Provider:      cfg.LLM.Provider,
		Model:         cfg.LLM.Model,
		OllamaBaseURL: cfg.LLM.OllamaBaseURL,
		AnthropicKey:  cfg.LLM.AnthropicKey,
		OpenAIKey:     cfg.LLM.OpenAIKey,
		GeminiKey:     cfg.LLM.GeminiKey,
		MaxTokens:     cfg.LLM.MaxTokens,
		Temperature:   cfg.LLM.Temperature,
		MaxRetries:    cfg.LLM.MaxRetries,
	})
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error initializing LLM: %v\n", err)
		os.Exit(1)
	}

	agentInstance := agent.New(
		"cynapse_mini_01",
		llmClient,
		store,
		cfg,
	)

	registry := synapse.NewRegistry()
	synapseDir := filepath.Join(getHomeDir(), "synapses")
	if err := registry.Discover(synapseDir); err != nil {
		// Synapses are optional
	}

	runCLI(agentInstance, store)
}

func runCLI(ag *agent.Agent, store *memory.LightweightStore) {
	reader := bufio.NewReader(os.Stdin)
	printWelcome()

	for {
		fmt.Print("\n> ")
		input, err := reader.ReadString('\n')
		if err != nil {
			return
		}

		input = strings.TrimSpace(input)
		if input == "" {
			continue
		}

		if strings.HasPrefix(input, "/") {
			handleCommand(input, ag, store)
			continue
		}

		fmt.Print("\n🧠 ")

		ctx, cancel := context.WithTimeout(context.Background(), 180*time.Second)
		err = ag.ProcessStreaming(ctx, input, func(chunk string) {
			fmt.Print(chunk)
		})
		cancel()

		if err != nil {
			fmt.Fprintf(os.Stderr, "\n❌ Error: %v\n", err)
		}
	}
}

func handleCommand(cmd string, ag *agent.Agent, store *memory.LightweightStore) {
	parts := strings.Fields(cmd)
	if len(parts) == 0 {
		return
	}

	command := strings.ToLower(parts[0])

	switch command {
	case "/help":
		printCommandHelp()
	case "/models":
		fmt.Println("Available LLM providers:")
		fmt.Println("  ollama    - Local inference (Ollama)")
		fmt.Println("  anthropic - Claude API")
		fmt.Println("  openai    - GPT API")
		fmt.Println("  gemini    - Google Gemini API")
	case "/clear":
		store.Clear()
		fmt.Println("✓ Conversation history cleared")
	case "/memory":
		msgs := store.GetRecent(5)
		fmt.Printf("\nLast %d messages:\n", len(msgs))
		for _, msg := range msgs {
			content := msg.Content
			if len(content) > 50 {
				content = content[:50] + "..."
			}
			fmt.Printf("[%s] %s\n", msg.Role, content)
		}
	case "/info":
		fmt.Println("\n🧠 CYNAPSE Mini v1.0.0")
		fmt.Println("Lightweight AI Agent for Raspberry Pi")
		fmt.Println("\nMinimalist CLI Interface")
		fmt.Println("Disk-based Memory System")
		fmt.Println("Streaming Response Output")
	case "/quit", "/exit":
		fmt.Println("\nGoodbye! 👋")
		os.Exit(0)
	default:
		fmt.Printf("Unknown command: %s\n", command)
		fmt.Println("Type /help for available commands")
	}
}

func handleSynapseCommand(args []string) {
	if len(args) == 0 {
		printSynapseHelp()
		return
	}

	cmd := args[0]
	subargs := args[1:]

	registry := synapse.NewRegistry()
	synapseDir := filepath.Join(getHomeDir(), "synapses")

	switch cmd {
	case "list":
		if err := registry.Discover(synapseDir); err != nil {
			fmt.Fprintf(os.Stderr, "Error: %v\n", err)
			os.Exit(1)
		}
		registry.List()
	case "add", "install":
		if len(subargs) == 0 {
			fmt.Println("Usage: cynapse-mini synapse add <name>")
			os.Exit(1)
		}
		if err := registry.Install(subargs[0], synapseDir); err != nil {
			fmt.Fprintf(os.Stderr, "Error: %v\n", err)
			os.Exit(1)
		}
	case "remove", "uninstall":
		if len(subargs) == 0 {
			fmt.Println("Usage: cynapse-mini synapse remove <name>")
			os.Exit(1)
		}
		if err := registry.Uninstall(subargs[0], synapseDir); err != nil {
			fmt.Fprintf(os.Stderr, "Error: %v\n", err)
			os.Exit(1)
		}
	case "search":
		if len(subargs) == 0 {
			registry.SearchAll()
		} else {
			registry.Search(subargs[0])
		}
	default:
		fmt.Printf("Unknown synapse command: %s\n", cmd)
		printSynapseHelp()
		os.Exit(1)
	}
}

func handleConfigCommand(args []string) {
	if len(args) == 0 {
		fmt.Printf("Config location: %s\n", getConfigPath())
		return
	}

	cmd := args[0]

	switch cmd {
	case "init":
		if err := config.CreateDefault(getConfigPath()); err != nil {
			fmt.Fprintf(os.Stderr, "Error creating config: %v\n", err)
			os.Exit(1)
		}
		fmt.Printf("✓ Created default config at: %s\n", getConfigPath())
	default:
		fmt.Printf("Unknown config command: %s\n", cmd)
		fmt.Print("Available commands: init\n")
		os.Exit(1)
	}
}

func printWelcome() {
	fmt.Print(`
╔════════════════════════════════════════╗
║  🧠 CYNAPSE Mini v1.0.0                ║
║  Lightweight AI Agent                  ║
║  For Raspberry Pi & Embedded Systems   ║
╚════════════════════════════════════════╝

Type /help for commands
Type your message to chat
`)
}

func printHelp() {
	fmt.Print(`
🧠 CYNAPSE Mini - Lightweight AI Agent

USAGE:
  cynapse-mini              Start interactive chat
  cynapse-mini synapse      Manage synapses
  cynapse-mini config       Manage configuration
  cynapse-mini version      Show version
  cynapse-mini help         Show this help

CHAT COMMANDS (type in chat):
  /help                     Show command help
  /models                   List available LLM providers
  /clear                    Clear conversation history
  /memory                   Show recent messages
  /info                     Show system info
  /quit                     Exit application

For more information, visit: https://github.com/Alartist40/cynapse-mini
`)
}

func printSynapseHelp() {
	fmt.Print(`
Synapse commands:
  list                 List installed synapses
  add <name>           Install a synapse from registry
  remove <name>        Remove an installed synapse
  search [query]       Search available synapses
`)
}

func printCommandHelp() {
	fmt.Print(`
Chat Commands:
  /help                 Show this help
  /models               List available LLM providers
  /clear                Clear conversation history
  /memory               Show recent messages in memory
  /info                 Show CYNAPSE Mini info
  /quit, /exit          Exit application

Normal chat:
  Just type your message and press Enter
`)
}

func getHomeDir() string {
	home := os.Getenv("HOME")
	if home == "" {
		home = os.Getenv("USERPROFILE")
	}
	return filepath.Join(home, ".cynapse-mini")
}

func getConfigPath() string {
	return filepath.Join(getHomeDir(), "config.yaml")
}

func ensureDir(path string) {
	if err := os.MkdirAll(path, 0755); err != nil {
		fmt.Fprintf(os.Stderr, "Warning: Failed to create directory %s: %v\n", path, err)
	}
}
