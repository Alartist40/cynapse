package memory

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"time"
)

// Message represents a single message in the conversation
type Message struct {
	Role      string    `json:"role"`
	Content   string    `json:"content"`
	Timestamp time.Time `json:"timestamp"`
}

// LightweightStore is a disk-based memory system with minimal RAM overhead
type LightweightStore struct {
	basePath     string
	sessionPath  string
	mu           sync.Mutex
	currentMsgs  []Message
	maxMsgMemory int
	sessionFile  *os.File
	encoder      *json.Encoder
}

// NewLightweightStore creates a new disk-based memory store
func NewLightweightStore(basePath string) (*LightweightStore, error) {
	if err := os.MkdirAll(basePath, 0755); err != nil {
		return nil, fmt.Errorf("creating memory directory: %w", err)
	}

	sessionPath := filepath.Join(basePath, "session.jsonl")
	f, err := os.OpenFile(sessionPath, os.O_CREATE|os.O_APPEND|os.O_WRONLY, 0644)
	if err != nil {
		return nil, fmt.Errorf("opening session file: %w", err)
	}

	store := &LightweightStore{
		basePath:     basePath,
		sessionPath:  sessionPath,
		currentMsgs:  make([]Message, 0, 20),
		maxMsgMemory: 20,
		sessionFile:  f,
		encoder:      json.NewEncoder(f),
	}

	store.loadRecent(20)
	return store, nil
}

// Save saves a message to disk immediately
func (s *LightweightStore) Save(msg Message) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	msg.Timestamp = time.Now()

	if err := s.encoder.Encode(msg); err != nil {
		return fmt.Errorf("encoding message: %w", err)
	}

	if len(s.currentMsgs) < s.maxMsgMemory {
		s.currentMsgs = append(s.currentMsgs, msg)
	} else {
		s.currentMsgs = append(s.currentMsgs[1:], msg)
	}

	return nil
}

// GetRecent returns the last N messages from memory
func (s *LightweightStore) GetRecent(count int) []Message {
	s.mu.Lock()
	defer s.mu.Unlock()

	if count > len(s.currentMsgs) {
		count = len(s.currentMsgs)
	}

	result := make([]Message, count)
	copy(result, s.currentMsgs[len(s.currentMsgs)-count:])
	return result
}

// Clear wipes the session history
func (s *LightweightStore) Clear() error {
	s.mu.Lock()
	defer s.mu.Unlock()

	s.currentMsgs = make([]Message, 0, 20)
	s.sessionFile.Close()

	f, err := os.Create(s.sessionPath)
	if err != nil {
		return fmt.Errorf("clearing session file: %w", err)
	}

	s.sessionFile = f
	s.encoder = json.NewEncoder(f)
	return nil
}

// Close closes the memory store
func (s *LightweightStore) Close() error {
	s.mu.Lock()
	defer s.mu.Unlock()

	if s.sessionFile != nil {
		return s.sessionFile.Close()
	}
	return nil
}

// loadRecent loads recent messages from disk into memory
func (s *LightweightStore) loadRecent(count int) error {
	data, err := os.ReadFile(s.sessionPath)
	if err != nil {
		if os.IsNotExist(err) {
			return nil
		}
		return fmt.Errorf("reading session file: %w", err)
	}

	lines := strings.Split(string(data), "\n")
	var allMsgs []Message

	for _, line := range lines {
		if line = strings.TrimSpace(line); line == "" {
			continue
		}

		var msg Message
		if err := json.Unmarshal([]byte(line), &msg); err != nil {
			continue
		}
		allMsgs = append(allMsgs, msg)
	}

	start := 0
	if len(allMsgs) > count {
		start = len(allMsgs) - count
	}

	s.currentMsgs = allMsgs[start:]
	return nil
}

// Statistics returns memory usage info
func (s *LightweightStore) Statistics() map[string]interface{} {
	s.mu.Lock()
	defer s.mu.Unlock()

	return map[string]interface{}{
		"total_messages":         len(s.currentMsgs),
		"memory_cached_messages": len(s.currentMsgs),
		"memory_limit":           s.maxMsgMemory,
		"storage_path":           s.sessionPath,
	}
}
