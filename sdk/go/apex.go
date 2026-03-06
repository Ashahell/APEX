package apex

import (
	"bytes"
	"crypto/hmac"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"
)

type Config struct {
	BaseURL      string
	SharedSecret string
	Timeout      time.Duration
}

type Client struct {
	config      Config
	httpClient  *http.Client
}

type TaskRequest struct {
	Content      string  `json:"content"`
	Channel      string  `json:"channel,omitempty"`
	ThreadID     string  `json:"thread_id,omitempty"`
	Author       string  `json:"author,omitempty"`
	MaxSteps     *int    `json:"max_steps,omitempty"`
	BudgetUSD    *float64 `json:"budget_usd,omitempty"`
	TimeLimitSecs *int   `json:"time_limit_secs,omitempty"`
	Project      string  `json:"project,omitempty"`
	Priority     string  `json:"priority,omitempty"`
	Category     string  `json:"category,omitempty"`
}

type TaskResponse struct {
	TaskID           string `json:"task_id"`
	Status           string `json:"status"`
	Tier             string `json:"tier"`
	CapabilityToken  string `json:"capability_token,omitempty"`
	InstantResponse string `json:"instant_response,omitempty"`
}

type TaskStatusResponse struct {
	TaskID    string  `json:"task_id"`
	Status    string  `json:"status"`
	Content   string  `json:"content,omitempty"`
	Output    string  `json:"output,omitempty"`
	Error     string  `json:"error,omitempty"`
	Project   string  `json:"project,omitempty"`
	Priority  string  `json:"priority,omitempty"`
	Category  string  `json:"category,omitempty"`
}

type TaskFilterRequest struct {
	Project   string `json:"project,omitempty"`
	Status    string `json:"status,omitempty"`
	Priority  string `json:"priority,omitempty"`
	Category  string `json:"category,omitempty"`
	Limit     int    `json:"limit,omitempty"`
	Offset    int    `json:"offset,omitempty"`
}

type Skill struct {
	Name              string `json:"name"`
	Version           string `json:"version"`
	Tier              string `json:"tier"`
	Description       string `json:"description,omitempty"`
	Healthy           bool   `json:"healthy"`
	LastHealthCheck   string `json:"last_health_check,omitempty"`
}

type ExecuteSkillRequest struct {
	SkillName string                 `json:"skill_name"`
	Input     map[string]interface{} `json:"input"`
}

type Metrics struct {
	Tasks          int             `json:"tasks"`
	TasksByTier    map[string]int  `json:"by_tier"`
	TasksByStatus  map[string]int   `json:"by_status"`
	TotalCostUSD   float64         `json:"total_cost_usd"`
}

type HealthResponse struct {
	Status string `json:"status"`
}

func NewClient(config Config) *Client {
	if config.Timeout == 0 {
		config.Timeout = 30 * time.Second
	}
	if config.BaseURL == "" {
		config.BaseURL = "http://localhost:3000"
	}
	return &Client{
		config: config,
		httpClient: &http.Client{
			Timeout: config.Timeout,
		},
	}
}

func (c *Client) signRequest(method, path, body string) (string, string) {
	timestamp := fmt.Sprintf("%d", time.Now().Unix())
	message := timestamp + method + path + body
	
	h := hmac.New(sha256.New, []byte(c.config.SharedSecret))
	h.Write([]byte(message))
	signature := hex.EncodeToString(h.Sum(nil))
	
	return signature, timestamp
}

func (c *Client) doRequest(method, path string, body interface{}) ([]byte, error) {
	var bodyStr string
	if body != nil {
		b, err := json.Marshal(body)
		if err != nil {
			return nil, err
		}
		bodyStr = string(b)
	}
	
	signature, timestamp := c.signRequest(method, path, bodyStr)
	
	url := c.config.BaseURL + path
	req, err := http.NewRequest(method, url, bytes.NewBufferString(bodyStr))
	if err != nil {
		return nil, err
	}
	
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("X-APEX-Signature", signature)
	req.Header.Set("X-APEX-Timestamp", timestamp)
	
	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	
	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}
	
	if resp.StatusCode >= 400 {
		return nil, fmt.Errorf("API error: %s", string(respBody))
	}
	
	return respBody, nil
}

// CreateTask creates a new task
func (c *Client) CreateTask(req TaskRequest) (*TaskResponse, error) {
	respBody, err := c.doRequest("POST", "/api/v1/tasks", req)
	if err != nil {
		return nil, err
	}
	
	var resp TaskResponse
	if err := json.Unmarshal(respBody, &resp); err != nil {
		return nil, err
	}
	
	return &resp, nil
}

// GetTask gets task status
func (c *Client) GetTask(taskID string) (*TaskStatusResponse, error) {
	path := fmt.Sprintf("/api/v1/tasks/%s", taskID)
	respBody, err := c.doRequest("GET", path, nil)
	if err != nil {
		return nil, err
	}
	
	var resp TaskStatusResponse
	if err := json.Unmarshal(respBody, &resp); err != nil {
		return nil, err
	}
	
	return &resp, nil
}

// ListTasks lists tasks with filters
func (c *Client) ListTasks(filter TaskFilterRequest) ([]TaskStatusResponse, error) {
	path := "/api/v1/tasks"
	if filter.Project != "" || filter.Status != "" || filter.Priority != "" || filter.Category != "" {
		path += "?"
		if filter.Project != "" {
			path += "project=" + filter.Project + "&"
		}
		if filter.Status != "" {
			path += "status=" + filter.Status + "&"
		}
		if filter.Priority != "" {
			path += "priority=" + filter.Priority + "&"
		}
		if filter.Category != "" {
			path += "category=" + filter.Category + "&"
		}
		if filter.Limit > 0 {
			path += fmt.Sprintf("limit=%d&", filter.Limit)
		}
		if filter.Offset > 0 {
			path += fmt.Sprintf("offset=%d", filter.Offset)
		}
	}
	
	respBody, err := c.doRequest("GET", path, nil)
	if err != nil {
		return nil, err
	}
	
	var resp []TaskStatusResponse
	if err := json.Unmarshal(respBody, &resp); err != nil {
		return nil, err
	}
	
	return resp, nil
}

// CancelTask cancels a task
func (c *Client) CancelTask(taskID string) (*TaskStatusResponse, error) {
	path := fmt.Sprintf("/api/v1/tasks/%s/cancel", taskID)
	respBody, err := c.doRequest("POST", path, nil)
	if err != nil {
		return nil, err
	}
	
	var resp TaskStatusResponse
	if err := json.Unmarshal(respBody, &resp); err != nil {
		return nil, err
	}
	
	return &resp, nil
}

// ListSkills lists all skills
func (c *Client) ListSkills() ([]Skill, error) {
	respBody, err := c.doRequest("GET", "/api/v1/skills", nil)
	if err != nil {
		return nil, err
	}
	
	var resp []Skill
	if err := json.Unmarshal(respBody, &resp); err != nil {
		return nil, err
	}
	
	return resp, nil
}

// ExecuteSkill executes a skill
func (c *Client) ExecuteSkill(req ExecuteSkillRequest) (map[string]interface{}, error) {
	respBody, err := c.doRequest("POST", "/api/v1/skills/execute", req)
	if err != nil {
		return nil, err
	}
	
	var resp map[string]interface{}
	if err := json.Unmarshal(respBody, &resp); err != nil {
		return nil, err
	}
	
	return resp, nil
}

// GetMetrics gets system metrics
func (c *Client) GetMetrics() (*Metrics, error) {
	respBody, err := c.doRequest("GET", "/api/v1/metrics", nil)
	if err != nil {
		return nil, err
	}
	
	var resp Metrics
	if err := json.Unmarshal(respBody, &resp); err != nil {
		return nil, err
	}
	
	return &resp, nil
}

// HealthCheck checks router health
func (c *Client) HealthCheck() (*HealthResponse, error) {
	respBody, err := c.doRequest("GET", "/health", nil)
	if err != nil {
		return nil, err
	}
	
	var resp HealthResponse
	if err := json.Unmarshal(respBody, &resp); err != nil {
		return nil, err
	}
	
	return &resp, nil
}
