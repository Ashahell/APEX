use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ExecutionEvent {
    Thought {
        step: u32,
        content: String,
    },
    ToolCall {
        step: u32,
        tool: String,
        input: serde_json::Value,
    },
    ToolProgress {
        step: u32,
        tool: String,
        output: String,
    },
    ToolResult {
        step: u32,
        tool: String,
        success: bool,
        output: String,
    },
    ApprovalNeeded {
        step: u32,
        tier: String,
        action: String,
        consequences: ConsequencePreview,
    },
    Error {
        step: u32,
        message: String,
    },
    Complete {
        output: String,
        steps: u32,
        tools_used: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsequencePreview {
    pub files_read: Vec<String>,
    pub files_written: Vec<String>,
    pub commands_executed: Vec<String>,
    pub blast_radius: BlastRadius,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BlastRadius {
    Minimal,
    Limited,
    Extensive,
}

impl BlastRadius {
    pub fn as_str(&self) -> &'static str {
        match self {
            BlastRadius::Minimal => "minimal",
            BlastRadius::Limited => "limited", 
            BlastRadius::Extensive => "extensive",
        }
    }
}

impl Default for ConsequencePreview {
    fn default() -> Self {
        Self {
            files_read: vec![],
            files_written: vec![],
            commands_executed: vec![],
            blast_radius: BlastRadius::Minimal,
            summary: "No changes detected".to_string(),
        }
    }
}

pub struct ExecutionStream {
    sender: Arc<broadcast::Sender<ExecutionEvent>>,
    task_id: String,
}

impl Clone for ExecutionStream {
    fn clone(&self) -> Self {
        Self {
            sender: Arc::clone(&self.sender),
            task_id: self.task_id.clone(),
        }
    }
}

impl ExecutionStream {
    pub fn new(task_id: String) -> Self {
        let (sender, _) = broadcast::channel(100);
        Self { 
            sender: Arc::new(sender), 
            task_id 
        }
    }

    pub fn from_sender(sender: Arc<broadcast::Sender<ExecutionEvent>>, task_id: String) -> Self {
        Self { sender, task_id }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ExecutionEvent> {
        self.sender.subscribe()
    }

    pub async fn emit(&self, event: ExecutionEvent) {
        let _ = self.sender.send(event);
    }

    pub async fn emit_thought(&self, step: u32, content: String) {
        self.emit(ExecutionEvent::Thought { step, content }).await;
    }

    pub async fn emit_tool_call(&self, step: u32, tool: String, input: serde_json::Value) {
        self.emit(ExecutionEvent::ToolCall { step, tool, input }).await;
    }

    pub async fn emit_tool_progress(&self, step: u32, tool: String, output: String) {
        self.emit(ExecutionEvent::ToolProgress { step, tool, output }).await;
    }

    pub async fn emit_tool_result(&self, step: u32, tool: String, success: bool, output: String) {
        self.emit(ExecutionEvent::ToolResult { step, tool, success, output }).await;
    }

    pub async fn emit_approval(&self, step: u32, tier: String, action: String, consequences: ConsequencePreview) {
        self.emit(ExecutionEvent::ApprovalNeeded { step, tier, action, consequences }).await;
    }

    pub async fn emit_error(&self, step: u32, message: String) {
        self.emit(ExecutionEvent::Error { step, message }).await;
    }

    pub async fn emit_complete(&self, output: String, steps: u32, tools_used: Vec<String>) {
        self.emit(ExecutionEvent::Complete { output, steps, tools_used }).await;
    }

    pub fn try_emit(&self, event: ExecutionEvent) {
        let _ = self.sender.send(event);
    }

    pub fn try_emit_thought(&self, step: u32, content: String) {
        self.try_emit(ExecutionEvent::Thought { step, content });
    }

    pub fn try_emit_tool_call(&self, step: u32, tool: String, input: serde_json::Value) {
        self.try_emit(ExecutionEvent::ToolCall { step, tool, input });
    }

    pub fn try_emit_tool_progress(&self, step: u32, tool: String, output: String) {
        self.try_emit(ExecutionEvent::ToolProgress { step, tool, output });
    }

    pub fn try_emit_tool_result(&self, step: u32, tool: String, success: bool, output: String) {
        self.try_emit(ExecutionEvent::ToolResult { step, tool, success, output });
    }

    pub fn try_emit_approval(&self, step: u32, tier: String, action: String, consequences: ConsequencePreview) {
        self.try_emit(ExecutionEvent::ApprovalNeeded { step, tier, action, consequences });
    }

    pub fn try_emit_error(&self, step: u32, message: String) {
        self.try_emit(ExecutionEvent::Error { step, message });
    }

    pub fn try_emit_complete(&self, output: String, steps: u32, tools_used: Vec<String>) {
        self.try_emit(ExecutionEvent::Complete { output, steps, tools_used });
    }

    pub fn task_id(&self) -> &str {
        &self.task_id
    }
}

#[derive(Clone)]
pub struct ExecutionStreamManager {
    streams: Arc<std::sync::Mutex<std::collections::HashMap<String, Arc<broadcast::Sender<ExecutionEvent>>>>>,
}

impl ExecutionStreamManager {
    pub fn new() -> Self {
        Self {
            streams: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub fn create_stream(&self, task_id: &str) -> ExecutionStream {
        let (sender, _) = broadcast::channel(100);
        let sender = Arc::new(sender);
        let stream = ExecutionStream::from_sender(Arc::clone(&sender), task_id.to_string());
        
        let mut streams = self.streams.lock().unwrap();
        streams.insert(task_id.to_string(), sender);
        
        stream
    }

    pub fn get_stream(&self, task_id: &str) -> Option<ExecutionStream> {
        let streams = self.streams.lock().unwrap();
        streams.get(task_id).map(|sender| ExecutionStream::from_sender(Arc::clone(sender), task_id.to_string()))
    }

    pub fn subscribe(&self, task_id: &str) -> Option<broadcast::Receiver<ExecutionEvent>> {
        let streams = self.streams.lock().unwrap();
        streams.get(task_id).map(|sender| sender.subscribe())
    }

    pub fn remove_stream(&self, task_id: &str) {
        let mut streams = self.streams.lock().unwrap();
        streams.remove(task_id);
    }

    pub fn list_active_streams(&self) -> Vec<String> {
        let streams = self.streams.lock().unwrap();
        streams.keys().cloned().collect()
    }
}

impl Default for ExecutionStreamManager {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn predict_consequences(
    action: &str,
    input: &serde_json::Value,
    llm_url: &str,
) -> ConsequencePreview {
    use reqwest::Client;
    
    let client = Client::new();
    
    let prompt = format!(
        r#"Analyze this action and predict its consequences. Respond in JSON format:
{{
  "files_read": ["list of files that will be read"],
  "files_written": ["list of files that will be modified/created"],
  "commands_executed": ["list of shell commands that will run"],
  "blast_radius": "minimal|limited|extensive",
  "summary": "one sentence summary of impact"
}}

Action: {} {}
"#,
        action,
        input
    );

    let request_body = serde_json::json!({
        "model": "qwen3-4b",
        "messages": [{"role": "user", "content": prompt}],
        "max_tokens": 512,
    });

    match client.post(format!("{}/v1/chat/completions", llm_url))
        .json(&request_body)
        .send()
        .await
    {
        Ok(response) => {
            if let Ok(data) = response.json::<serde_json::Value>().await {
                if let Some(content) = data["choices"]
                    .as_array()
                    .and_then(|c| c.first())
                    .and_then(|c| c.get("message"))
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_str())
                {
                    if let Ok(preview) = serde_json::from_str::<ConsequencePreview>(content) {
                        return preview;
                    }
                }
            }
            ConsequencePreview::default()
        }
        Err(_) => ConsequencePreview::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consequence_preview_default() {
        let preview = ConsequencePreview::default();
        assert!(preview.files_read.is_empty());
        assert_eq!(preview.blast_radius, BlastRadius::Minimal);
    }

    #[test]
    fn test_blast_radius_strings() {
        assert_eq!(BlastRadius::Minimal.as_str(), "minimal");
        assert_eq!(BlastRadius::Limited.as_str(), "limited");
        assert_eq!(BlastRadius::Extensive.as_str(), "extensive");
    }

    #[tokio::test]
    async fn test_execution_stream() {
        let stream = ExecutionStream::new("test-task".to_string());
        
        let mut rx = stream.subscribe();
        
        stream.emit_thought(0, "Starting task".to_string()).await;
        
        let event = rx.recv().await.unwrap();
        
        match event {
            ExecutionEvent::Thought { step, content } => {
                assert_eq!(step, 0);
                assert_eq!(content, "Starting task");
            }
            _ => panic!("Expected Thought event"),
        }
    }

    #[tokio::test]
    async fn test_stream_manager() {
        let manager = ExecutionStreamManager::new();
        
        let stream1 = manager.create_stream("task-1");
        let stream2 = manager.get_stream("task-1");
        
        assert!(stream2.is_some());
        
        let mut rx1 = stream1.subscribe();
        let mut rx2 = stream2.unwrap().subscribe();
        
        stream1.emit(ExecutionEvent::Thought { step: 1, content: "test".to_string() }).await;
        
        let event1 = rx1.recv().await.unwrap();
        let event2 = rx2.recv().await.unwrap();
        
        match (&event1, &event2) {
            (ExecutionEvent::Thought { step: s1, content: c1 }, ExecutionEvent::Thought { step: s2, content: c2 }) => {
                assert_eq!(s1, s2);
                assert_eq!(c1, c2);
            }
            _ => panic!("Expected Thought events"),
        }
        
        manager.remove_stream("task-1");
        assert!(manager.get_stream("task-1").is_none());
    }
}
