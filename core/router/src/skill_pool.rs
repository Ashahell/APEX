use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::io::AsyncBufReadExt;
use tokio::process::Command;
use tokio::sync::{mpsc, Mutex};
use tracing::{error, info, warn};

use crate::skill_pool_ipc::{IpcChannel, IpcResponse, SkillPoolError};

#[derive(Debug, Clone)]
pub struct SkillPoolConfig {
    pub pool_size: usize,
    pub worker_script: PathBuf,
    pub skills_dir: PathBuf,
    pub request_timeout_ms: u64,
    pub acquire_timeout_ms: u64,
    pub health_check_interval: Duration,
}

impl Default for SkillPoolConfig {
    fn default() -> Self {
        Self {
            pool_size: 4,
            worker_script: PathBuf::from("./skills/pool_worker.ts"),
            skills_dir: PathBuf::from("./skills"),
            request_timeout_ms: 30_000,
            acquire_timeout_ms: 5_000,
            health_check_interval: Duration::from_secs(30),
        }
    }
}

struct PoolSlot {
    channel: IpcChannel,
    pid: u32,
}

pub struct SkillPool {
    free_tx: mpsc::Sender<usize>,
    free_rx: Arc<Mutex<mpsc::Receiver<usize>>>,
    slots: Arc<Vec<Mutex<PoolSlot>>>,
    config: SkillPoolConfig,
    total_requests: AtomicU64,
    total_errors: AtomicU64,
    total_latency_ms: AtomicU64,
    pool_size: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SkillPoolStats {
    pub pool_size: usize,
    pub available_slots: usize,
    pub total_requests: u64,
    pub total_errors: u64,
    pub avg_latency_ms: u64,
    pub enabled: bool,
}

impl SkillPool {
    pub fn config(&self) -> &SkillPoolConfig {
        &self.config
    }

    pub async fn new(config: SkillPoolConfig) -> Result<Arc<Self>, SkillPoolError> {
        let (free_tx, free_rx) = mpsc::channel(config.pool_size);
        let mut slots = Vec::with_capacity(config.pool_size);

        for i in 0..config.pool_size {
            match Self::spawn_slot(&config).await {
                Ok(slot) => {
                    info!("SkillPool: slot {} ready (pid {})", i, slot.pid);
                    slots.push(Mutex::new(slot));
                    let _ = free_tx.send(i).await;
                }
                Err(e) => {
                    error!("SkillPool: failed to start slot {}: {}", i, e);
                    return Err(e);
                }
            }
        }

        let pool = Arc::new(Self {
            free_tx,
            free_rx: Arc::new(Mutex::new(free_rx)),
            slots: Arc::new(slots),
            config: config.clone(),
            total_requests: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            total_latency_ms: AtomicU64::new(0),
            pool_size: config.pool_size,
        });

        let pool_clone = Arc::clone(&pool);
        tokio::spawn(async move {
            pool_clone.health_check_loop().await;
        });

        Ok(pool)
    }

    async fn spawn_slot(config: &SkillPoolConfig) -> Result<PoolSlot, SkillPoolError> {
        let mut child = Command::new("bun")
            .arg("run")
            .arg(&config.worker_script)
            .env("APEX_SKILLS_DIR", &config.skills_dir)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| SkillPoolError::SpawnError(e.to_string()))?;

        let pid = child.id().unwrap_or(0);
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        if let Some(stderr) = child.stderr.take() {
            let pid_copy = pid;
            tokio::spawn(async move {
                let mut lines = tokio::io::BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    warn!("SkillPool [pid {}] stderr: {}", pid_copy, line);
                }
            });
        }

        tokio::spawn(async move {
            let _ = child.wait().await;
        });

        let mut stdout_reader = tokio::io::BufReader::new(stdout);
        let mut ready_line = String::new();

        tokio::time::timeout::<_>(Duration::from_secs(10), stdout_reader.read_line(&mut ready_line))
            .await
            .map_err(|_| SkillPoolError::SpawnError("Timed out waiting for READY".into()))?
            .map_err(SkillPoolError::Io)?;

        let ready: serde_json::Value = serde_json::from_str(ready_line.trim())
            .map_err(|_| SkillPoolError::SpawnError(format!("Unexpected READY: {}", ready_line.trim())))?;

        if ready.get("ready").and_then(|v| v.as_bool()) != Some(true) {
            return Err(SkillPoolError::SpawnError(format!("Expected ready:true, got: {}", ready_line.trim())));
        }

        let stdout_inner = stdout_reader.into_inner();
        let channel = IpcChannel::new(stdin, stdout_inner);

        Ok(PoolSlot { channel, pid })
    }

    pub async fn execute(
        &self,
        skill: &str,
        input: serde_json::Value,
        tier: Option<String>,
    ) -> Result<IpcResponse, SkillPoolError> {
        let start = std::time::Instant::now();
        let acquire_timeout = Duration::from_millis(self.config.acquire_timeout_ms);

        let slot_index = tokio::time::timeout(
            acquire_timeout,
            async { self.free_rx.lock().await.recv().await }
        )
        .await
        .map_err(|_| {
            self.total_errors.fetch_add(1, Ordering::Relaxed);
            SkillPoolError::NoSlots
        })?
        .ok_or({
            self.total_errors.fetch_add(1, Ordering::Relaxed);
            SkillPoolError::ChannelClosed
        })?;

        let result = {
            let slot = self.slots[slot_index].lock().await;
            slot.channel.send(skill, input, self.config.request_timeout_ms, tier).await
        };

        let _ = self.free_tx.send(slot_index).await;

        let latency = start.elapsed().as_millis() as u64;
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.total_latency_ms.fetch_add(latency, Ordering::Relaxed);

        if result.is_err() {
            self.total_errors.fetch_add(1, Ordering::Relaxed);
        }

        result
    }

    pub async fn stats(&self) -> SkillPoolStats {
        let available = self.free_rx.lock().await.len();
        let total = self.total_requests.load(Ordering::Relaxed);
        let errors = self.total_errors.load(Ordering::Relaxed);
        let total_latency = self.total_latency_ms.load(Ordering::Relaxed);
        let avg_latency = if total > 0 { total_latency / total } else { 0 };

        SkillPoolStats {
            pool_size: self.pool_size,
            available_slots: available,
            total_requests: total,
            total_errors: errors,
            avg_latency_ms: avg_latency,
            enabled: true,
        }
    }

    async fn health_check_loop(self: Arc<Self>) {
        let mut interval = tokio::time::interval(self.config.health_check_interval);
        loop {
            interval.tick().await;

            for i in 0..self.config.pool_size {
                let is_healthy = {
                    let slot = self.slots[i].lock().await;
                    match tokio::time::timeout(
                        Duration::from_secs(2),
                        slot.channel.send("__ping__", serde_json::json!({}), 2000, None)
                    ).await {
                        Ok(Ok(resp)) => resp.ok,
                        _ => false,
                    }
                };

                if !is_healthy {
                    warn!("SkillPool: slot {} is unhealthy — restarting", i);
                    match Self::spawn_slot(&self.config).await {
                        Ok(new_slot) => {
                            let mut slot = self.slots[i].lock().await;
                            *slot = new_slot;
                            info!("SkillPool: slot {} restarted", i);
                        }
                        Err(e) => {
                            error!("SkillPool: failed to restart slot {}: {}", i, e);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_pool_config_default() {
        let config = SkillPoolConfig::default();
        assert_eq!(config.pool_size, 4);
        assert_eq!(config.request_timeout_ms, 30_000);
        assert_eq!(config.acquire_timeout_ms, 5_000);
    }

    #[test]
    fn test_skill_pool_config_custom() {
        let config = SkillPoolConfig {
            pool_size: 8,
            worker_script: PathBuf::from("/custom/script.ts"),
            skills_dir: PathBuf::from("/custom/skills"),
            request_timeout_ms: 60_000,
            acquire_timeout_ms: 10_000,
            health_check_interval: Duration::from_secs(60),
        };
        assert_eq!(config.pool_size, 8);
        assert_eq!(config.request_timeout_ms, 60_000);
        assert_eq!(config.acquire_timeout_ms, 10_000);
    }
}
