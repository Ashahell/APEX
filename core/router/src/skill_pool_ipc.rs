use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::{ChildStdin, ChildStdout};
use tokio::sync::{oneshot, Mutex};
use uuid::Uuid;

pub type PendingMap = Arc<Mutex<HashMap<String, oneshot::Sender<IpcResponse>>>>;

#[derive(Debug, serde::Serialize, Clone)]
pub struct IpcRequest {
    pub id: String,
    pub skill: String,
    pub input: serde_json::Value,
    pub timeout_ms: u64,
    pub permitted_tier: Option<String>, // B1: tier passed from Router
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct IpcResponse {
    pub id: String,
    pub ok: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

pub struct IpcChannel {
    writer: Arc<Mutex<BufWriter<ChildStdin>>>,
    pending: PendingMap,
}

impl IpcChannel {
    pub fn new(stdin: ChildStdin, stdout: ChildStdout) -> Self {
        let writer = Arc::new(Mutex::new(BufWriter::new(stdin)));
        let pending: PendingMap = Arc::new(Mutex::new(HashMap::new()));

        let pending_clone = Arc::clone(&pending);
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if line.is_empty() {
                    continue;
                }

                if let Ok(resp) = serde_json::from_str::<IpcResponse>(&line) {
                    let mut map = pending_clone.lock().await;
                    if let Some(tx) = map.remove(&resp.id) {
                        let _ = tx.send(resp);
                    }
                }
            }
        });

        Self { writer, pending }
    }

    pub async fn send(
        &self,
        skill: &str,
        input: serde_json::Value,
        timeout_ms: u64,
        permitted_tier: Option<String>,
    ) -> Result<IpcResponse, SkillPoolError> {
        let id = Uuid::new_v4().to_string();
        let req = IpcRequest {
            id: id.clone(),
            skill: skill.to_string(),
            input,
            timeout_ms,
            permitted_tier,
        };

        let (tx, rx) = oneshot::channel::<IpcResponse>();

        {
            let mut map = self.pending.lock().await;
            map.insert(id.clone(), tx);
        }

        {
            let mut w = self.writer.lock().await;
            let mut line = serde_json::to_string(&req)?;
            line.push('\n');
            w.write_all(line.as_bytes()).await?;
            w.flush().await?;
        }

        let duration = std::time::Duration::from_millis(timeout_ms + 1000);
        match tokio::time::timeout(duration, rx).await {
            Ok(Ok(resp)) => Ok(resp),
            Ok(Err(_)) => Err(SkillPoolError::ChannelClosed),
            Err(_) => {
                self.pending.lock().await.remove(&id);
                Err(SkillPoolError::Timeout)
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SkillPoolError {
    #[error("skill pool: timeout waiting for Bun response")]
    Timeout,
    #[error("skill pool: IPC channel closed")]
    ChannelClosed,
    #[error("skill pool: no slots available")]
    NoSlots,
    #[error("skill pool: process failed to start: {0}")]
    SpawnError(String),
    #[error("skill pool: I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("skill pool: serialisation error: {0}")]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_request_serialization() {
        let req = IpcRequest {
            id: "test-uuid".to_string(),
            skill: "test.skill".to_string(),
            input: serde_json::json!({"key": "value"}),
            timeout_ms: 5000,
            permitted_tier: Some("T1".to_string()),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("test-uuid"));
        assert!(json.contains("test.skill"));
        assert!(json.contains("key"));
    }

    #[test]
    fn test_ipc_response_deserialization_success() {
        let resp: IpcResponse = serde_json::from_str(
            r#"{
            "id": "test-123",
            "ok": true,
            "output": "success",
            "error": null,
            "duration_ms": 15
        }"#,
        )
        .unwrap();
        assert_eq!(resp.id, "test-123");
        assert!(resp.ok);
        assert_eq!(resp.output, Some("success".to_string()));
        assert_eq!(resp.duration_ms, 15);
    }

    #[test]
    fn test_ipc_response_deserialization_error() {
        let resp: IpcResponse = serde_json::from_str(
            r#"{
            "id": "test-456",
            "ok": false,
            "output": null,
            "error": "skill not found",
            "duration_ms": 5
        }"#,
        )
        .unwrap();
        assert_eq!(resp.id, "test-456");
        assert!(!resp.ok);
        assert_eq!(resp.error, Some("skill not found".to_string()));
    }

    #[test]
    fn test_skill_pool_error_display() {
        assert_eq!(
            SkillPoolError::Timeout.to_string(),
            "skill pool: timeout waiting for Bun response"
        );
        assert_eq!(
            SkillPoolError::ChannelClosed.to_string(),
            "skill pool: IPC channel closed"
        );
        assert_eq!(
            SkillPoolError::NoSlots.to_string(),
            "skill pool: no slots available"
        );
    }
}
