package llm

import (
	"bufio"
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"
	"time"

	"github.com/Alartist40/cynapse-mini/internal/config"
)

// ─── Types ───────────────────────────────────────────────────────────────────

type Role string

const (
	RoleSystem    Role = "system"
	RoleUser      Role = "user"
	RoleAssistant Role = "assistant"
	RoleTool      Role = "tool"
)

type Message struct {
	Role       Role       `json:"role"`
	Content    string     `json:"content"`
	ToolCallID string     `json:"tool_call_id,omitempty"`
	ToolCalls  []ToolCall `json:"tool_calls,omitempty"`
}

type ToolCall struct {
	ID        string          `json:"id"`
	Name      string          `json:"name"`
	Arguments json.RawMessage `json:"arguments"`
}

type ToolSchema struct {
	Name        string         `json:"name"`
	Description string         `json:"description"`
	Parameters  map[string]any `json:"parameters"`
}

type Request struct {
	SystemPrompt string
	Messages     []Message
	Tools        []ToolSchema
	MaxTokens    int
	Temperature  float64
}

type Response struct {
	Content   string
	ToolCalls []ToolCall
	Usage     Usage
}

type Usage struct {
	InputTokens  int
	OutputTokens int
}

// ─── Interface ───────────────────────────────────────────────────────────────

type Client interface {
	Chat(ctx context.Context, req *Request) (*Response, error)
	ChatStream(ctx context.Context, req *Request) (<-chan string, <-chan error)
	Provider() string
}

// ─── Model Discovery ─────────────────────────────────────────────────────────

// ListOllamaModels returns all available models from a running Ollama instance
func ListOllamaModels(baseURL string) ([]string, error) {
	if baseURL == "" {
		baseURL = "http://localhost:11434"
	}

	// Create HTTP client with timeout
	client := &http.Client{Timeout: 30 * time.Second}

	url := strings.TrimRight(baseURL, "/") + "/api/tags"
	resp, err := client.Get(url)
	if err != nil {
		return nil, fmt.Errorf("connecting to ollama: %w", err)
	}
	defer resp.Body.Close()

	var result struct {
		Models []struct {
			Name string `json:"name"`
		} `json:"models"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, fmt.Errorf("parsing ollama response: %w", err)
	}

	var names []string
	for _, m := range result.Models {
		names = append(names, m.Name)
	}
	return names, nil
}

// ─── Factory ─────────────────────────────────────────────────────────────────

func New(cfg *config.LLMConfig) (Client, error) {
	base := &baseClient{
		http:       &http.Client{Timeout: 300 * time.Second},
		maxRetries: cfg.MaxRetries,
	}

	switch strings.ToLower(cfg.Provider) {
	case "anthropic":
		if cfg.AnthropicKey == "" {
			return nil, fmt.Errorf("ANTHROPIC_API_KEY not set")
		}
		return &anthropicClient{baseClient: base, apiKey: cfg.AnthropicKey, model: cfg.Model}, nil

	case "openai":
		if cfg.OpenAIKey == "" {
			return nil, fmt.Errorf("OPENAI_API_KEY not set")
		}
		return &openaiClient{baseClient: base, apiKey: cfg.OpenAIKey, model: cfg.Model}, nil

	case "gemini":
		if cfg.GeminiKey == "" {
			return nil, fmt.Errorf("GEMINI_API_KEY not set")
		}
		return &geminiClient{baseClient: base, apiKey: cfg.GeminiKey, model: cfg.Model}, nil

	case "ollama":
		baseURL := cfg.OllamaBaseURL
		if baseURL == "" {
			baseURL = "http://localhost:11434"
		}
		return &ollamaClient{baseClient: base, baseURL: baseURL, model: cfg.Model}, nil

	default:
		return nil, fmt.Errorf("unknown LLM provider: %q (use ollama|anthropic|openai|gemini)", cfg.Provider)
	}
}

// ─── Base client (shared HTTP + retry logic) ─────────────────────────────────

type baseClient struct {
	http       *http.Client
	maxRetries int
}

func (b *baseClient) do(ctx context.Context, method, url string, headers map[string]string, body any) ([]byte, error) {
	var bodyReader io.Reader
	if body != nil {
		data, err := json.Marshal(body)
		if err != nil {
			return nil, err
		}
		bodyReader = bytes.NewReader(data)
	}

	var lastErr error
	for attempt := 0; attempt <= b.maxRetries; attempt++ {
		if attempt > 0 {
			select {
			case <-ctx.Done():
				return nil, ctx.Err()
			case <-time.After(time.Duration(attempt) * 2 * time.Second):
			}
		}

		req, err := http.NewRequestWithContext(ctx, method, url, bodyReader)
		if err != nil {
			return nil, err
		}

		req.Header.Set("Content-Type", "application/json")
		for k, v := range headers {
			req.Header.Set(k, v)
		}

		resp, err := b.http.Do(req)
		if err != nil {
			lastErr = err
			continue
		}
		defer resp.Body.Close()

		data, err := io.ReadAll(resp.Body)
		if err != nil {
			lastErr = err
			continue
		}

		if resp.StatusCode == 429 || resp.StatusCode >= 500 {
			lastErr = fmt.Errorf("HTTP %d: %s", resp.StatusCode, truncate(string(data), 200))
			// Reset body for retry
			if body != nil {
				b2, _ := json.Marshal(body)
				bodyReader = bytes.NewReader(b2)
			}
			continue
		}

		if resp.StatusCode >= 400 {
			return nil, fmt.Errorf("HTTP %d: %s", resp.StatusCode, truncate(string(data), 300))
		}

		return data, nil
	}

	return nil, fmt.Errorf("after %d retries: %w", b.maxRetries, lastErr)
}

// ─── Anthropic ───────────────────────────────────────────────────────────────

type anthropicClient struct {
	*baseClient
	apiKey string
	model  string
}

func (c *anthropicClient) Provider() string { return "anthropic" }

func (c *anthropicClient) Chat(ctx context.Context, req *Request) (*Response, error) {
	type aContent struct {
		Type  string `json:"type"`
		Text  string `json:"text,omitempty"`
		ID    string `json:"id,omitempty"`
		Name  string `json:"name,omitempty"`
		Input any    `json:"input,omitempty"`
	}
	type aMsg struct {
		Role    string     `json:"role"`
		Content []aContent `json:"content"`
	}
	type aTool struct {
		Name        string         `json:"name"`
		Description string         `json:"description"`
		InputSchema map[string]any `json:"input_schema"`
	}
	type aReq struct {
		Model     string  `json:"model"`
		MaxTokens int     `json:"max_tokens"`
		System    string  `json:"system,omitempty"`
		Messages  []aMsg  `json:"messages"`
		Tools     []aTool `json:"tools,omitempty"`
	}

	apiReq := aReq{
		Model:     c.model,
		MaxTokens: req.MaxTokens,
		System:    req.SystemPrompt,
	}

	for _, m := range req.Messages {
		apiReq.Messages = append(apiReq.Messages, aMsg{
			Role:    string(m.Role),
			Content: []aContent{{Type: "text", Text: m.Content}},
		})
	}
	for _, t := range req.Tools {
		apiReq.Tools = append(apiReq.Tools, aTool{
			Name: t.Name, Description: t.Description, InputSchema: t.Parameters,
		})
	}

	headers := map[string]string{
		"x-api-key":         c.apiKey,
		"anthropic-version": "2023-06-01",
	}
	data, err := c.do(ctx, "POST", "https://api.anthropic.com/v1/messages", headers, apiReq)
	if err != nil {
		return nil, err
	}

	var aResp struct {
		Content []struct {
			Type  string          `json:"type"`
			Text  string          `json:"text"`
			ID    string          `json:"id"`
			Name  string          `json:"name"`
			Input json.RawMessage `json:"input"`
		} `json:"content"`
		Usage struct {
			InputTokens  int `json:"input_tokens"`
			OutputTokens int `json:"output_tokens"`
		} `json:"usage"`
	}
	if err := json.Unmarshal(data, &aResp); err != nil {
		return nil, err
	}

	result := &Response{Usage: Usage{InputTokens: aResp.Usage.InputTokens, OutputTokens: aResp.Usage.OutputTokens}}
	for _, c := range aResp.Content {
		if c.Type == "text" {
			result.Content += c.Text
		} else if c.Type == "tool_use" {
			result.ToolCalls = append(result.ToolCalls, ToolCall{ID: c.ID, Name: c.Name, Arguments: c.Input})
		}
	}
	return result, nil
}

// ─── OpenAI ──────────────────────────────────────────────────────────────────

type openaiClient struct {
	*baseClient
	apiKey string
	model  string
}

func (c *anthropicClient) ChatStream(ctx context.Context, req *Request) (<-chan string, <-chan error) {
	chunks := make(chan string)
	errors := make(chan error, 1)
	close(chunks)
	errors <- fmt.Errorf("streaming not implemented for anthropic")
	return chunks, errors
}

func (c *openaiClient) Provider() string { return "openai" }

func (c *openaiClient) Chat(ctx context.Context, req *Request) (*Response, error) {
	type oMsg struct {
		Role    string `json:"role"`
		Content string `json:"content"`
	}
	type oTool struct {
		Type     string `json:"type"`
		Function struct {
			Name        string         `json:"name"`
			Description string         `json:"description"`
			Parameters  map[string]any `json:"parameters"`
		} `json:"function"`
	}
	type oReq struct {
		Model       string  `json:"model"`
		MaxTokens   int     `json:"max_tokens"`
		Temperature float64 `json:"temperature"`
		Messages    []oMsg  `json:"messages"`
		Tools       []oTool `json:"tools,omitempty"`
	}

	apiReq := oReq{Model: c.model, MaxTokens: req.MaxTokens, Temperature: req.Temperature}

	if req.SystemPrompt != "" {
		apiReq.Messages = append(apiReq.Messages, oMsg{Role: "system", Content: req.SystemPrompt})
	}
	for _, m := range req.Messages {
		apiReq.Messages = append(apiReq.Messages, oMsg{Role: string(m.Role), Content: m.Content})
	}
	for _, t := range req.Tools {
		ot := oTool{Type: "function"}
		ot.Function.Name = t.Name
		ot.Function.Description = t.Description
		ot.Function.Parameters = t.Parameters
		apiReq.Tools = append(apiReq.Tools, ot)
	}

	headers := map[string]string{"Authorization": "Bearer " + c.apiKey}
	data, err := c.do(ctx, "POST", "https://api.openai.com/v1/chat/completions", headers, apiReq)
	if err != nil {
		return nil, err
	}

	var oResp struct {
		Choices []struct {
			Message struct {
				Content   string `json:"content"`
				ToolCalls []struct {
					ID       string `json:"id"`
					Function struct {
						Name      string          `json:"name"`
						Arguments json.RawMessage `json:"arguments"`
					} `json:"function"`
				} `json:"tool_calls"`
			} `json:"message"`
		} `json:"choices"`
		Usage struct {
			PromptTokens     int `json:"prompt_tokens"`
			CompletionTokens int `json:"completion_tokens"`
		} `json:"usage"`
	}
	if err := json.Unmarshal(data, &oResp); err != nil {
		return nil, err
	}

	result := &Response{Usage: Usage{InputTokens: oResp.Usage.PromptTokens, OutputTokens: oResp.Usage.CompletionTokens}}
	if len(oResp.Choices) > 0 {
		msg := oResp.Choices[0].Message
		result.Content = msg.Content
		for _, tc := range msg.ToolCalls {
			result.ToolCalls = append(result.ToolCalls, ToolCall{
				ID: tc.ID, Name: tc.Function.Name, Arguments: tc.Function.Arguments,
			})
		}
	}
	return result, nil
}

// ─── Gemini ──────────────────────────────────────────────────────────────────

type geminiClient struct {
	*baseClient
	apiKey string
	model  string
}

func (c *geminiClient) Provider() string { return "gemini" }

func (c *geminiClient) Chat(ctx context.Context, req *Request) (*Response, error) {
	type gPart struct {
		Text string `json:"text"`
	}
	type gContent struct {
		Role  string  `json:"role"`
		Parts []gPart `json:"parts"`
	}
	type gReq struct {
		SystemInstruction *gContent  `json:"systemInstruction,omitempty"`
		Contents          []gContent `json:"contents"`
	}

	apiReq := gReq{}
	if req.SystemPrompt != "" {
		apiReq.SystemInstruction = &gContent{Parts: []gPart{{Text: req.SystemPrompt}}}
	}
	for _, m := range req.Messages {
		role := string(m.Role)
		if role == "assistant" {
			role = "model"
		}
		apiReq.Contents = append(apiReq.Contents, gContent{Role: role, Parts: []gPart{{Text: m.Content}}})
	}

	url := fmt.Sprintf("https://generativelanguage.googleapis.com/v1beta/models/%s:generateContent?key=%s", c.model, c.apiKey)
	data, err := c.do(ctx, "POST", url, nil, apiReq)
	if err != nil {
		return nil, err
	}

	var gResp struct {
		Candidates []struct {
			Content struct {
				Parts []struct {
					Text string `json:"text"`
				} `json:"parts"`
			} `json:"content"`
		} `json:"candidates"`
	}
	if err := json.Unmarshal(data, &gResp); err != nil {
		return nil, err
	}

	result := &Response{}
	if len(gResp.Candidates) > 0 {
		for _, p := range gResp.Candidates[0].Content.Parts {
			result.Content += p.Text
		}
	}
	return result, nil
}

func (c *openaiClient) ChatStream(ctx context.Context, req *Request) (<-chan string, <-chan error) {
	chunks := make(chan string)
	errors := make(chan error, 1)
	close(chunks)
	errors <- fmt.Errorf("streaming not implemented for openai")
	return chunks, errors
}

func (c *geminiClient) ChatStream(ctx context.Context, req *Request) (<-chan string, <-chan error) {
	chunks := make(chan string)
	errors := make(chan error, 1)
	close(chunks)
	errors <- fmt.Errorf("streaming not implemented for gemini")
	return chunks, errors
}

// ─── Ollama ──────────────────────────────────────────────────────────────────

type ollamaClient struct {
	*baseClient
	baseURL string
	model   string
}

func (c *ollamaClient) Provider() string { return "ollama" }

func (c *ollamaClient) Chat(ctx context.Context, req *Request) (*Response, error) {
	type oMsg struct {
		Role    string `json:"role"`
		Content string `json:"content"`
	}
	type oTool struct {
		Type     string `json:"type"`
		Function struct {
			Name        string         `json:"name"`
			Description string         `json:"description"`
			Parameters  map[string]any `json:"parameters"`
		} `json:"function"`
	}
	type oReq struct {
		Model    string  `json:"model"`
		Messages []oMsg  `json:"messages"`
		Tools    []oTool `json:"tools,omitempty"`
		Stream   bool    `json:"stream"`
		Options  struct {
			NumPredict  int     `json:"num_predict,omitempty"`
			Temperature float64 `json:"temperature,omitempty"`
		} `json:"options"`
	}

	apiReq := oReq{Model: c.model, Stream: false}
	apiReq.Options.NumPredict = req.MaxTokens
	apiReq.Options.Temperature = req.Temperature

	if req.SystemPrompt != "" {
		apiReq.Messages = append(apiReq.Messages, oMsg{Role: "system", Content: req.SystemPrompt})
	}
	for _, m := range req.Messages {
		apiReq.Messages = append(apiReq.Messages, oMsg{Role: string(m.Role), Content: m.Content})
	}
	for _, t := range req.Tools {
		ot := oTool{Type: "function"}
		ot.Function.Name = t.Name
		ot.Function.Description = t.Description
		ot.Function.Parameters = t.Parameters
		apiReq.Tools = append(apiReq.Tools, ot)
	}

	url := strings.TrimRight(c.baseURL, "/") + "/api/chat"
	data, err := c.do(ctx, "POST", url, nil, apiReq)
	if err != nil {
		return nil, err
	}

	var oResp struct {
		Message struct {
			Content   string `json:"content"`
			ToolCalls []struct {
				Function struct {
					Name      string          `json:"name"`
					Arguments json.RawMessage `json:"arguments"`
				} `json:"function"`
			} `json:"tool_calls"`
		} `json:"message"`
		PromptEvalCount int `json:"prompt_eval_count"`
		EvalCount       int `json:"eval_count"`
	}
	if err := json.Unmarshal(data, &oResp); err != nil {
		return nil, err
	}

	result := &Response{
		Content: oResp.Message.Content,
		Usage:   Usage{InputTokens: oResp.PromptEvalCount, OutputTokens: oResp.EvalCount},
	}
	for _, tc := range oResp.Message.ToolCalls {
		result.ToolCalls = append(result.ToolCalls, ToolCall{
			Name: tc.Function.Name, Arguments: tc.Function.Arguments,
		})
	}
	return result, nil
}

func (c *ollamaClient) ChatStream(ctx context.Context, req *Request) (<-chan string, <-chan error) {
	chunks := make(chan string, 10)
	errors := make(chan error, 1)

	go func() {
		defer close(chunks)
		defer close(errors)

		type oMsg struct {
			Role    string `json:"role"`
			Content string `json:"content"`
		}
		type oReq struct {
			Model    string `json:"model"`
			Messages []oMsg `json:"messages"`
			Stream   bool   `json:"stream"`
			Options  struct {
				NumPredict  int     `json:"num_predict,omitempty"`
				Temperature float64 `json:"temperature,omitempty"`
			} `json:"options"`
		}

		apiReq := oReq{Model: c.model, Stream: true}
		apiReq.Options.NumPredict = req.MaxTokens
		apiReq.Options.Temperature = req.Temperature

		if req.SystemPrompt != "" {
			apiReq.Messages = append(apiReq.Messages, oMsg{Role: "system", Content: req.SystemPrompt})
		}
		for _, m := range req.Messages {
			apiReq.Messages = append(apiReq.Messages, oMsg{Role: string(m.Role), Content: m.Content})
		}

		body, err := json.Marshal(apiReq)
		if err != nil {
			errors <- fmt.Errorf("marshaling request: %w", err)
			return
		}

		httpReq, err := http.NewRequestWithContext(ctx, "POST", c.baseURL+"/api/chat", bytes.NewReader(body))
		if err != nil {
			errors <- fmt.Errorf("creating request: %w", err)
			return
		}
		httpReq.Header.Set("Content-Type", "application/json")

		resp, err := c.http.Do(httpReq)
		if err != nil {
			errors <- fmt.Errorf("ollama request failed: %w", err)
			return
		}
		defer resp.Body.Close()

		if resp.StatusCode != http.StatusOK {
			bodyBytes, _ := io.ReadAll(resp.Body)
			errors <- fmt.Errorf("ollama HTTP %d: %s", resp.StatusCode, string(bodyBytes))
			return
		}

		// Read streaming response line by line (NDJSON)
		scanner := bufio.NewScanner(resp.Body)
		for scanner.Scan() {
			line := scanner.Bytes()

			var streamResp struct {
				Message struct {
					Content string `json:"content"`
				} `json:"message"`
				Done bool `json:"done"`
			}

			if err := json.Unmarshal(line, &streamResp); err != nil {
				continue // Skip malformed lines
			}

			if streamResp.Message.Content != "" {
				select {
				case chunks <- streamResp.Message.Content:
				case <-ctx.Done():
					errors <- ctx.Err()
					return
				}
			}

			if streamResp.Done {
				return
			}
		}

		if err := scanner.Err(); err != nil {
			errors <- fmt.Errorf("reading stream: %w", err)
		}
	}()

	return chunks, errors
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

func truncate(s string, n int) string {
	if len(s) <= n {
		return s
	}
	return s[:n] + "…"
}
