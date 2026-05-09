package agent

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/Alartist40/cynapse-mini/internal/config"
	"github.com/Alartist40/cynapse-mini/internal/llm"
	"github.com/Alartist40/cynapse-mini/internal/memory"
)

// Agent is a lightweight agent that processes user input and generates responses
type Agent struct {
	deviceID string
	llm      llm.Client
	store    *memory.LightweightStore
	cfg      *config.Config
}

// New creates a new lightweight agent
func New(
	deviceID string,
	llmClient llm.Client,
	store *memory.LightweightStore,
	cfg *config.Config,
) *Agent {
	return &Agent{
		deviceID: deviceID,
		llm:      llmClient,
		store:    store,
		cfg:      cfg,
	}
}

// ProcessStreaming processes user input and streams the response
func (a *Agent) ProcessStreaming(ctx context.Context, input string, onChunk func(string)) error {
	// Save user message
	a.store.Save(memory.Message{
		Role:      "user",
		Content:   input,
		Timestamp: time.Now(),
	})

	// Build minimal context
	req := a.buildRequest()

	// Stream response from LLM
	chunks, errs := a.llm.ChatStream(ctx, req)

	fullResponse := ""
	for {
		select {
		case chunk, ok := <-chunks:
			if !ok {
				chunks = nil
			} else {
				fullResponse += chunk
				onChunk(chunk)
			}
		case err := <-errs:
			if err != nil {
				return err
			}
			errs = nil
		}
		if chunks == nil && errs == nil {
			break
		}
	}

	onChunk("\n")

	// Save assistant response
	a.store.Save(memory.Message{
		Role:      "assistant",
		Content:   fullResponse,
		Timestamp: time.Now(),
	})

	return nil
}

// buildRequest builds the LLM request from recent messages
func (a *Agent) buildRequest() *llm.Request {
	recent := a.store.GetRecent(20)

	var msgs []llm.Message
	for _, msg := range recent {
		msgs = append(msgs, llm.Message{
			Role:    llm.Role(msg.Role),
			Content: msg.Content,
		})
	}

	return &llm.Request{
		Messages:    msgs,
		MaxTokens:   2048,
		Temperature: 0.7,
	}
}

// ProcessSync processes user input and returns full response at once
func (a *Agent) ProcessSync(ctx context.Context, input string) (string, error) {
	a.store.Save(memory.Message{
		Role:      "user",
		Content:   input,
		Timestamp: time.Now(),
	})

	req := a.buildRequest()
	resp, err := a.llm.Chat(ctx, req)
	if err != nil {
		return "", err
	}

	a.store.Save(memory.Message{
		Role:      "assistant",
		Content:   resp.Content,
		Timestamp: time.Now(),
	})

	return resp.Content, nil
}

// GetSummary returns a brief summary of the conversation
func (a *Agent) GetSummary() string {
	recent := a.store.GetRecent(5)

	if len(recent) == 0 {
		return "No conversation yet"
	}

	summary := fmt.Sprintf("Last %d messages:\n", len(recent))
	for _, msg := range recent {
		label := "User"
		if msg.Role == "assistant" {
			label = "Agent"
		}

		content := msg.Content
		if len(content) > 80 {
			content = content[:80] + "..."
		}

		summary += fmt.Sprintf("  [%s] %s\n", label, strings.ReplaceAll(content, "\n", " "))
	}

	return summary
}

// ProcessCommand processes special commands
func (a *Agent) ProcessCommand(cmd string) string {
	switch cmd {
	case "status":
		return "CYNAPSE Mini is running normally\n"
	case "stats":
		stats := a.store.Statistics()
		return fmt.Sprintf("Memory Statistics:\n  Total messages: %v\n  Cached in memory: %v/%v\n",
			stats["total_messages"],
			stats["memory_cached_messages"],
			stats["memory_limit"],
		)
	default:
		return "Unknown command: " + cmd
	}
}
