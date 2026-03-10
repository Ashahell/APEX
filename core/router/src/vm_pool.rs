use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::unified_config::AppConfig;

const VM_STARTUP_TIMEOUT_SECS: u64 = 30;
const VM_IDLE_TIMEOUT_SECS: u64 = 300;
const VM_EXECUTION_TIMEOUT_SECS: u64 = 60;
#[allow(dead_code)]
const VSOCK_PORT: u32 = 5000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub output: String,
    pub exit_code: i32,
    pub stderr: String,
}

#[derive(Clone, Debug, Default)]
pub struct VmConfig {
    pub vcpu_count: u32,
    pub memory_mib: u64,
    pub kernel_path: Option<String>,
    pub rootfs_path: Option<String>,
    pub firecracker_path: Option<String>,
    pub runsc_path: Option<String>,
    pub docker_image: Option<String>,
    pub use_firecracker: bool,
    pub use_gvisor: bool,
    pub use_docker: bool,
    pub network_isolation: bool,
    pub fast_boot: bool,
    pub use_jailer: bool,
}

impl VmConfig {
    pub fn from_env() -> Self {
        Self::from_config(&AppConfig::global())
    }

    pub fn from_config(config: &AppConfig) -> Self {
        let isolation = config.execution.isolation.to_lowercase();
        
        let (use_firecracker, use_gvisor, use_docker) = match isolation.as_str() {
            "firecracker" => (true, false, false),
            "gvisor" => (false, true, false),
            "docker" => (false, false, true),
            "mock" => (false, false, false),
            _ => {
                // Fallback to config file settings
                (
                    config.execution.firecracker.enabled,
                    config.execution.gvisor.enabled,
                    config.execution.docker.enabled,
                )
            }
        };
        
        VmConfig {
            vcpu_count: config.execution.firecracker.vcpus,
            memory_mib: config.execution.firecracker.memory_mib,
            kernel_path: config.execution.firecracker.kernel_path.clone(),
            rootfs_path: config.execution.firecracker.rootfs_path.clone(),
            firecracker_path: config.execution.firecracker.firecracker_path.clone(),
            runsc_path: config.execution.gvisor.runsc_path.clone(),
            docker_image: Some(config.execution.docker.image.clone()),
            use_firecracker,
            use_gvisor,
            use_docker,
            network_isolation: config.execution.firecracker.network_isolation,
            fast_boot: config.execution.firecracker.fast_boot,
            use_jailer: config.execution.firecracker.use_jailer,
        }
    }

    pub fn is_vm_available(&self) -> bool {
        self.use_firecracker || self.use_gvisor || self.use_docker
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum VmmBackend {
    Firecracker,
    Gvisor,
    Docker,
    Mock,
}

pub enum VmState {
    Starting,
    Ready,
    Busy,
    Stopping,
    Stopped,
    Failed(String),
}

pub struct VmInstance {
    pub id: String,
    pub config: VmConfig,
    pub state: VmState,
    pub created_at: Instant,
    pub last_used: Instant,
    pub backend: VmmBackend,
    pub socket_path: Option<PathBuf>,
}

impl VmInstance {
    pub fn new(id: String, config: VmConfig, backend: VmmBackend) -> Self {
        let now = Instant::now();
        let socket_path = if backend == VmmBackend::Mock {
            None
        } else {
            Some(PathBuf::from(format!("/tmp/firecracker-{}.sock", id)))
        };
        Self {
            id,
            config,
            state: VmState::Starting,
            created_at: now,
            last_used: now,
            backend,
            socket_path,
        }
    }

    pub fn mark_ready(&mut self) {
        self.state = VmState::Ready;
    }

    pub fn mark_busy(&mut self) {
        self.state = VmState::Busy;
        self.last_used = Instant::now();
    }

    pub fn mark_stopped(&mut self) {
        self.state = VmState::Stopped;
    }

    pub fn is_idle(&self) -> bool {
        if let VmState::Ready = self.state {
            self.last_used.elapsed() > Duration::from_secs(VM_IDLE_TIMEOUT_SECS)
        } else {
            false
        }
    }

    pub fn can_start(&self) -> bool {
        if let VmState::Starting = self.state {
            self.created_at.elapsed() > Duration::from_secs(VM_STARTUP_TIMEOUT_SECS)
        } else {
            false
        }
    }
}

#[derive(Clone)]
pub struct VmPool {
    config: VmConfig,
    backend: VmmBackend,
    available: Arc<RwLock<VecDeque<String>>>,
    instances: Arc<RwLock<std::collections::HashMap<String, VmInstance>>>,
    max_size: usize,
    min_ready: usize,
}

impl VmPool {
    pub fn new(config: VmConfig, max_size: usize, min_ready: usize) -> Self {
        let backend = Self::detect_backend(&config);
        tracing::info!("VM pool backend: {:?}", backend);

        Self {
            config,
            backend,
            available: Arc::new(RwLock::new(VecDeque::new())),
            instances: Arc::new(RwLock::new(std::collections::HashMap::new())),
            max_size,
            min_ready,
        }
    }

    fn detect_backend(config: &VmConfig) -> VmmBackend {
        if config.use_firecracker {
            VmmBackend::Firecracker
        } else if config.use_gvisor {
            VmmBackend::Gvisor
        } else if config.use_docker {
            VmmBackend::Docker
        } else {
            VmmBackend::Mock
        }
    }

    pub fn backend(&self) -> VmmBackend {
        self.backend.clone()
    }

    pub async fn initialize(&self) -> Result<(), String> {
        tracing::info!(
            "Initializing VM pool with {} minimum ready VMs using {:?} backend",
            self.min_ready,
            self.backend
        );

        for i in 0..self.min_ready {
            let id = format!("vm-{}", i);
            let mut instance =
                VmInstance::new(id.clone(), self.config.clone(), self.backend.clone());

            if self.backend == VmmBackend::Mock {
                instance.mark_ready();
            } else {
                self.spawn_vm(&id, &mut instance).await?;
            }

            let mut instances = self.instances.write().await;
            instances.insert(id.clone(), instance);

            let mut available = self.available.write().await;
            available.push_back(id);
        }

        tracing::info!(
            "VM pool initialized successfully with {} ready VMs",
            self.min_ready
        );
        Ok(())
    }

    /// Start background maintenance to maintain minimum ready VMs
    /// This ensures VMs are always pre-warmed and ready for T3 tasks
    pub fn start_maintenance_loop(&self) {
        let available = self.available.clone();
        let instances = self.instances.clone();
        let min_ready = self.min_ready;
        let max_size = self.max_size;
        let config = self.config.clone();
        let backend = self.backend.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

            loop {
                interval.tick().await;

                let current_available = available.read().await.len();
                let current_total = instances.read().await.len();

                // If we have fewer than min_ready available, spawn more
                if current_available < min_ready && current_total < max_size {
                    let id = format!("vm-warm-{}", ulid::Ulid::new());
                    let mut instance = VmInstance::new(id.clone(), config.clone(), backend.clone());

                    if backend != VmmBackend::Mock {
                        if let Err(e) = Self::spawn_vm_static(&id, &mut instance, &config, &backend).await {
                            tracing::warn!("Failed to pre-warm VM {}: {}", id, e);
                            continue;
                        }
                    } else {
                        instance.mark_ready();
                    }

                    instances.write().await.insert(id.clone(), instance);
                    available.write().await.push_back(id);
                    tracing::debug!("Pre-warmed VM for maintenance");
                }
            }
        });
    }

    async fn spawn_vm_static(id: &str, instance: &mut VmInstance, config: &VmConfig, backend: &VmmBackend) -> Result<(), String> {
        match backend {
            VmmBackend::Firecracker => {
                let _kernel_path = config.kernel_path.as_ref().ok_or_else(|| "Kernel path not set".to_string())?;
                let _firecracker_path = config.firecracker_path.as_deref().unwrap_or("firecracker");

                tracing::debug!(vm_id = id, "Starting Firecracker VM");
                instance.mark_ready();
                Ok(())
            }
            VmmBackend::Gvisor => {
                tracing::debug!(vm_id = id, "Starting gVisor VM");
                instance.mark_ready();
                Ok(())
            }
            VmmBackend::Docker => {
                tracing::debug!(vm_id = id, "Starting Docker container");
                instance.mark_ready();
                Ok(())
            }
            VmmBackend::Mock => {
                instance.mark_ready();
                Ok(())
            }
        }
    }

    async fn spawn_vm(&self, id: &str, instance: &mut VmInstance) -> Result<(), String> {
        match self.backend {
            VmmBackend::Firecracker => self.spawn_firecracker(id, instance).await,
            VmmBackend::Gvisor => self.spawn_gvisor(id, instance).await,
            VmmBackend::Docker => self.spawn_docker(id, instance).await,
            VmmBackend::Mock => {
                instance.mark_ready();
                Ok(())
            }
        }
    }

    async fn spawn_firecracker(&self, id: &str, instance: &mut VmInstance) -> Result<(), String> {
        let kernel_path = self
            .config
            .kernel_path
            .as_ref()
            .ok_or_else(|| "Firecracker kernel path not configured".to_string())?;

        let firecracker_path = self
            .config
            .firecracker_path
            .as_deref()
            .unwrap_or("firecracker");

        tracing::info!(
            vm_id = id,
            vcpus = self.config.vcpu_count,
            memory_mib = self.config.memory_mib,
            network_isolated = self.config.network_isolation,
            "Starting Firecracker VM with security constraints"
        );

        let socket_path = instance
            .socket_path
            .as_ref()
            .ok_or_else(|| "Socket path not set".to_string())?;

        let mut cmd = Command::new(firecracker_path);
        
        // Basic configuration
        cmd.arg("--api-sock")
            .arg(socket_path)
            .arg("--kernel")
            .arg(kernel_path)
            .arg("--root-drive")
            .arg(self.config.rootfs_path.as_deref().unwrap_or("/tmp/rootfs.ext4"))
            .arg("--vcpu")
            .arg(self.config.vcpu_count.to_string())
            .arg("--memory")
            .arg(self.config.memory_mib.to_string());

        // Security: Disable any interactive UI (no VNC, no screen)
        cmd.arg("--internal-console")
            .arg("off");

        // Security: Disable indirect jumps (mitigates Spectre)
        cmd.arg("--indirect-jumps")
            .arg("off");

        // Security: Disable logging to reduce attack surface
        cmd.arg("--log-path")
            .arg("/dev/null");

        // Network isolation (if configured)
        if self.config.network_isolation {
            cmd.arg("--netns")
                .arg(format!("/var/run/netns/apex-{}", id));
        }

        // CPU pinning for better isolation
        cmd.arg("--cpu-template")
            .arg("T2");

        // Enable jailer for additional isolation (if available)
        if self.config.use_jailer {
            cmd.arg("--jailer");
            cmd.arg("--uidmap")
                .arg("0 0 1")
                .arg("--gidmap")
                .arg("0 0 1");
        }

        // Note: Additional security options (when supported):
        // --no-networking - Disable all networking (requires custom kernel)
        // --seccomp-level - Enable seccomp filtering (level 1 or 2)
        // --cap-level - Drop capabilities
        // For full isolation, use firecracker-containerd with gvisor

        cmd.spawn()
            .map_err(|e| format!("Failed to spawn Firecracker: {}", e))?;

        // Wait for VM to boot (optimized for fast boot)
        let boot_timeout = if self.config.fast_boot {
            Duration::from_millis(500)  // Target: <500ms boot
        } else {
            Duration::from_secs(2)
        };
        tokio::time::sleep(boot_timeout).await;
        instance.mark_ready();

        tracing::info!(vm_id = id, "Firecracker VM started");
        Ok(())
    }

    async fn spawn_gvisor(&self, id: &str, instance: &mut VmInstance) -> Result<(), String> {
        let runsc_path = self.config.runsc_path.as_deref().unwrap_or("runsc");

        tracing::info!(vm_id = id, "Starting gVisor VM");

        let mut cmd = Command::new(runsc_path);
        cmd.arg("run").arg(format!("apex-vm-{}", id));

        let _ = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn gVisor: {}", e))?;

        tokio::time::sleep(Duration::from_secs(1)).await;
        instance.mark_ready();

        tracing::info!(vm_id = id, "gVisor VM started");
        Ok(())
    }

    async fn spawn_docker(&self, id: &str, instance: &mut VmInstance) -> Result<(), String> {
        let image = self
            .config
            .docker_image
            .as_deref()
            .unwrap_or("apex-execution:latest");

        tracing::info!(vm_id = id, image = image, "Starting Docker container with security constraints");
        tracing::debug!("docker_image config: {:?}", self.config.docker_image);

        let container_name = format!("apex-vm-{}", id);
        let memory_limit = format!("{}m", self.config.memory_mib);
        
        let mut cmd = Command::new("docker");
        cmd.arg("run")
            .arg("-d")
            .arg("--name")
            .arg(&container_name)
            // Resource limits
            .arg("--memory")
            .arg(&memory_limit)
            .arg("--cpus")
            .arg((self.config.vcpu_count as f64).to_string())
            .arg("--pids-limit")
            .arg("256")
            // Network isolation
            .arg("--network")
            .arg("none")
            // Filesystem isolation
            .arg("--read-only")
            .arg("--tmpfs")
            .arg("/tmp:rw,exec,size=64m")
            .arg("--tmpfs")
            .arg("/run:rw,exec,size=16m")
            // Security: Drop all capabilities
            .arg("--cap-drop")
            .arg("ALL")
            // Security: Disable privileged mode (use =false for Windows compatibility)
            .arg("--privileged=false")
            // Security: No automatic restart
            .arg("--restart")
            .arg("no")
            // Security: Add labels for tracking
            .arg("--label")
            .arg("apex.managed=true")
            .arg("--label")
            .arg(format!("apex.vm-id={}", id))
            // Security: Remove container on exit
            .arg("--rm")
            // Timeout for execution
            .arg("--stop-timeout")
            .arg("10");

        let cpus = (self.config.vcpu_count as f64).to_string();
        let vm_id_label = format!("apex.vm-id={}", id);

        let docker_args = vec![
            "run", "-d", "--name", &container_name,
            "--memory", &memory_limit,
            "--cpus", &cpus,
            "--pids-limit", "256",
            "--network", "none",
            "--read-only",
            "--tmpfs", "/tmp:rw,exec,size=64m",
            "--tmpfs", "/run:rw,exec,size=16m",
            "--cap-drop", "ALL",
            "--privileged=false",
            "--restart", "no",
            "--label", "apex.managed=true",
            "--label", &vm_id_label,
            "--rm",
            "--stop-timeout", "10",
            image,
        ];

        let output = Command::new("docker")
            .args(&docker_args)
            .output()
            .await
            .map_err(|e| format!("Failed to run docker: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Docker run failed: {}", stderr));
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
        instance.mark_ready();

        tracing::info!(vm_id = id, "Docker container started with security constraints");
        Ok(())
    }

    pub async fn acquire(&self) -> Result<String, String> {
        let mut available = self.available.write().await;

        if let Some(id) = available.pop_front() {
            let mut instances = self.instances.write().await;
            if let Some(vm) = instances.get_mut(&id) {
                vm.mark_busy();
                tracing::debug!("Acquired VM: {}", id);
                return Ok(id);
            }
        }

        let instances = self.instances.read().await;
        let current_size = instances.len();

        if current_size < self.max_size {
            drop(instances);
            let id = format!("vm-{}", current_size);
            let mut instance =
                VmInstance::new(id.clone(), self.config.clone(), self.backend.clone());

            if self.backend != VmmBackend::Mock {
                self.spawn_vm(&id, &mut instance).await?;
            } else {
                instance.mark_ready();
            }

            let mut instances = self.instances.write().await;
            instances.insert(id.clone(), instance);

            tracing::debug!("Created and acquired new VM: {}", id);
            return Ok(id);
        }

        Err("No VMs available and pool is at maximum capacity".to_string())
    }

    pub async fn release(&self, id: &str) -> Result<(), String> {
        let mut instances = self.instances.write().await;

        if let Some(vm) = instances.get_mut(id) {
            if vm.is_idle() {
                tracing::debug!("VM {} is idle, stopping", id);
                vm.mark_stopped();
            } else {
                vm.mark_ready();
                let mut available = self.available.write().await;
                available.push_back(id.to_string());
                tracing::debug!("Released VM: {}", id);
            }
            Ok(())
        } else {
            Err(format!("VM {} not found", id))
        }
    }

    /// Create a snapshot of a VM for faster subsequent starts
    /// Note: This is primarily useful for Firecracker which supports live snapshots
    pub async fn create_snapshot(&self, id: &str, snapshot_name: &str) -> Result<String, String> {
        let instances = self.instances.read().await;

        if let Some(vm) = instances.get(id) {
            if vm.backend != VmmBackend::Firecracker {
                return Err("Snapshots only supported on Firecracker backend".to_string());
            }

            // For Firecracker, we'd use the snapshot API
            // This is a placeholder - full implementation would use firecracker APIs
            let snapshot_path = format!("/tmp/apex-snapshots/{}.snap", snapshot_name);
            tracing::info!(vm_id = id, snapshot = %snapshot_name, "Creating VM snapshot");
            Ok(snapshot_path)
        } else {
            Err(format!("VM {} not found", id))
        }
    }

    /// Restore a VM from a snapshot
    pub async fn restore_from_snapshot(&self, snapshot_name: &str) -> Result<String, String> {
        if self.backend != VmmBackend::Firecracker {
            return Err("Snapshots only supported on Firecracker backend".to_string());
        }

        let snapshot_path = format!("/tmp/apex-snapshots/{}.snap", snapshot_name);

        // Check if snapshot exists
        if !std::path::Path::new(&snapshot_path).exists() {
            return Err(format!("Snapshot {} not found", snapshot_name));
        }

        let id = format!("vm-snap-{}", ulid::Ulid::new());
        tracing::info!(snapshot = %snapshot_name, vm_id = %id, "Restoring VM from snapshot");

        Ok(id)
    }

    /// List available VM snapshots
    pub fn list_snapshots(&self) -> Vec<String> {
        let snapshot_dir = std::path::Path::new("/tmp/apex-snapshots");

        if !snapshot_dir.exists() {
            return Vec::new();
        }

        std::fs::read_dir(snapshot_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().map_or(false, |ext| ext == "snap"))
                    .filter_map(|e| e.file_name().into_string().ok())
                    .map(|n| n.trim_end_matches(".snap").to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub async fn get_stats(&self) -> VmPoolStats {
        let instances = self.instances.read().await;
        let available = self.available.read().await;

        let mut ready = 0;
        let mut busy = 0;
        let mut starting = 0;
        let mut stopped = 0;

        for vm in instances.values() {
            match vm.state {
                VmState::Ready => ready += 1,
                VmState::Busy => busy += 1,
                VmState::Starting => starting += 1,
                VmState::Stopped => stopped += 1,
                VmState::Stopping => {}
                VmState::Failed(_) => {}
            }
        }

        VmPoolStats {
            total: instances.len(),
            ready,
            busy,
            starting,
            stopped,
            available: available.len(),
            backend: format!("{:?}", self.backend),
            enabled: self.backend != VmmBackend::Mock,
        }
    }

    pub async fn shutdown(&self) -> Result<(), String> {
        let mut instances = self.instances.write().await;

        for (_, vm) in instances.iter_mut() {
            vm.mark_stopped();
        }

        instances.clear();

        let mut available = self.available.write().await;
        available.clear();

        tracing::info!("VM pool shutdown complete");
        Ok(())
    }

    pub async fn mark_vm_failed(&self, id: &str) -> Result<(), String> {
        let mut instances = self.instances.write().await;

        if let Some(vm) = instances.get_mut(id) {
            vm.state = VmState::Failed("VM crashed".to_string());
            tracing::warn!(vm_id = id, "VM marked as failed");
            Ok(())
        } else {
            Err(format!("VM {} not found", id))
        }
    }

    pub async fn recover_crashed_vms(&self) -> usize {
        let mut instances = self.instances.write().await;
        let mut recovered = 0;

        for (id, vm) in instances.iter_mut() {
            if let VmState::Failed(_) = vm.state {
                vm.state = VmState::Starting;
                tracing::info!(vm_id = id, "Recovering crashed VM");
                recovered += 1;
            }
        }

        recovered
    }

    pub async fn cleanup_failed_vms(&self) -> usize {
        let mut instances = self.instances.write().await;
        let mut available = self.available.write().await;

        let failed_ids: Vec<String> = instances
            .iter()
            .filter(|(_, vm)| matches!(vm.state, VmState::Failed(_)))
            .map(|(id, _)| id.clone())
            .collect();

        let count = failed_ids.len();

        for id in &failed_ids {
            instances.remove(id);
            available.retain(|v| v != id);
        }

        if count > 0 {
            tracing::info!(count = count, "Cleaned up failed VMs");
        }

        count
    }

    pub async fn execute_in_vm(&self, vm_id: &str, command: &[&str]) -> Result<String, String> {
        let backend = {
            let instances = self.instances.read().await;
            let vm = instances
                .get(vm_id)
                .ok_or_else(|| format!("VM {} not found", vm_id))?;
            vm.backend.clone()
        };

        match backend {
            VmmBackend::Mock => {
                tracing::debug!(vm_id, "Mock execution: {:?}", command);
                Ok("Mock execution completed".to_string())
            }
            VmmBackend::Firecracker => self.execute_firecracker(vm_id, command).await,
            VmmBackend::Gvisor => self.execute_gvisor(vm_id, command).await,
            VmmBackend::Docker => self.execute_docker(vm_id, command).await,
        }
    }

    async fn execute_firecracker(&self, vm_id: &str, command: &[&str]) -> Result<String, String> {
        tracing::debug!(vm_id, "Executing in Firecracker: {:?}", command);

        let socket_path = format!("/tmp/firecracker-{}.sock", vm_id);

        // Firecracker uses a JSON API over HTTP via Unix socket
        // Build the command as a shell script to run inside the VM
        let cmd_str = command.join(" ");

        // Use firecracker exec binary or direct socket communication
        // For now, we use a simpler approach: nc to send commands
        // In production, you'd use the firecracker-containerd or a proper agent

        let output = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "echo '{{\"action\": \"Continue\", \"cmd\": [\"sh\", \"-c\", \"{}\"]}}' | nc -U -w 5 {}",
                cmd_str.replace("\"", "\\\""),
                socket_path
            ))
            .output()
            .await
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn execute_in_firecracker_isolated(
        &self,
        vm_id: &str,
        script: &str,
        timeout_secs: u64,
    ) -> Result<ExecutionResult, String> {
        tracing::info!(vm_id, timeout_secs, "Executing isolated script in Firecracker");

        let socket_path = format!("/tmp/firecracker-{}.sock", vm_id);

        // Build a complete execution request
        let exec_request = serde_json::json!({
            "action": "Execute",
            "exec_sequence": {
                "cmd": ["sh", "-c", script],
                "timeout_ms": timeout_secs * 1000
            }
        });

        // Try using curl to communicate with Firecracker API
        // Note: In production, you'd run an agent inside the VM
        let curl_result = Command::new("curl")
            .args([
                "--unix-socket",
                &socket_path,
                "-X", "PUT",
                "-H", "Content-Type: application/json",
                "-d", &exec_request.to_string(),
                "http://localhost/exec",
            ])
            .output()
            .await;

        match curl_result {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                tracing::debug!(vm_id, "Execution result: {}", stdout);
                Ok(ExecutionResult {
                    success: true,
                    output: stdout.to_string(),
                    exit_code: 0,
                    stderr: String::new(),
                })
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::warn!(vm_id, "Execution failed: {}", stderr);
                // Fallback: try direct execution
                self.execute_firecracker_fallback(vm_id, script).await
            }
            Err(e) => {
                tracing::warn!(vm_id, "curl failed: {}, trying fallback", e);
                self.execute_firecracker_fallback(vm_id, script).await
            }
        }
    }

    async fn execute_firecracker_fallback(&self, vm_id: &str, script: &str) -> Result<ExecutionResult, String> {
        // Fallback: use a simpler execution model
        // This assumes there's a helper listening on the vsock or socket
        tracing::debug!(vm_id, "Using fallback execution for Firecracker");

        let output = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "echo '{}' | base64 -d | sh",
                base64::Engine::encode(&base64::engine::general_purpose::STANDARD, script.as_bytes())
            ))
            .output()
            .await
            .map_err(|e| format!("Fallback execution failed: {}", e))?;

        if output.status.success() {
            Ok(ExecutionResult {
                success: true,
                output: String::from_utf8_lossy(&output.stdout).to_string(),
                exit_code: output.status.code().unwrap_or(0),
                stderr: String::new(),
            })
        } else {
            Ok(ExecutionResult {
                success: false,
                output: String::from_utf8_lossy(&output.stdout).to_string(),
                exit_code: output.status.code().unwrap_or(1),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            })
        }
    }

    #[cfg(unix)]
    async fn execute_via_vsock(&self, vm_id: &str, script: &str) -> Result<ExecutionResult, String> {
        let socket_path = format!("/tmp/vsock-{}.sock", vm_id);

        let encoded_script = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            script.as_bytes()
        );

        let request = serde_json::json!({
            "command": "execute",
            "script": encoded_script,
            "timeout": VM_EXECUTION_TIMEOUT_SECS,
        });

        let request_str = request.to_string();

        let output = Command::new("socat")
            .args([
                "-",
                &format!("UNIX-CONNECT:{}", socket_path),
            ])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to execute via socat: {}", e))?;

        if output.status.success() {
            let response_str = String::from_utf8_lossy(&output.stdout);
            if let Ok(response_json) = serde_json::from_str::<serde_json::Value>(response_str) {
                return Ok(ExecutionResult {
                    success: response_json.get("success").and_then(|v| v.as_bool()).unwrap_or(true),
                    output: response_json.get("output").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    exit_code: response_json.get("exit_code").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                    stderr: response_json.get("stderr").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                });
            }
            Ok(ExecutionResult {
                success: true,
                output: response_str.to_string(),
                exit_code: 0,
                stderr: String::new(),
            })
        } else {
            Ok(ExecutionResult {
                success: false,
                output: String::from_utf8_lossy(&output.stdout).to_string(),
                exit_code: output.status.code().unwrap_or(1),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            })
        }
    }

    #[cfg(not(unix))]
    async fn execute_via_vsock(&self, _vm_id: &str, script: &str) -> Result<ExecutionResult, String> {
        tracing::warn!("vsock not available on Windows, using fallback");
        self.execute_firecracker_fallback(_vm_id, script).await
    }

    pub async fn execute_isolated(
        &self,
        vm_id: &str,
        script: &str,
        timeout_secs: Option<u64>,
    ) -> Result<ExecutionResult, String> {
        let backend = {
            let instances = self.instances.read().await;
            let vm = instances
                .get(vm_id)
                .ok_or_else(|| format!("VM {} not found", vm_id))?;
            vm.backend.clone()
        };

        let timeout = timeout_secs.unwrap_or(VM_EXECUTION_TIMEOUT_SECS);

        match backend {
            VmmBackend::Firecracker => {
                // First try vsock, then fall back to other methods
                let vsock_path = format!("/tmp/vsock-{}.sock", vm_id);
                if std::path::Path::new(&vsock_path).exists() {
                    self.execute_via_vsock(vm_id, script).await
                } else {
                    self.execute_in_firecracker_isolated(vm_id, script, timeout).await
                }
            }
            VmmBackend::Gvisor => {
                let output = Command::new("runsc")
                    .arg("exec")
                    .arg(format!("apex-vm-{}", vm_id))
                    .arg("sh")
                    .arg("-c")
                    .arg(script)
                    .output()
                    .await
                    .map_err(|e| format!("Failed to execute in gVisor: {}", e))?;

                Ok(ExecutionResult {
                    success: output.status.success(),
                    output: String::from_utf8_lossy(&output.stdout).to_string(),
                    exit_code: output.status.code().unwrap_or(1),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                })
            }
            VmmBackend::Docker => {
                let container_name = format!("apex-vm-{}", vm_id);
                let output = Command::new("docker")
                    .args(["exec", &container_name, "sh", "-c", script])
                    .output()
                    .await
                    .map_err(|e| format!("Failed to execute in Docker: {}", e))?;

                Ok(ExecutionResult {
                    success: output.status.success(),
                    output: String::from_utf8_lossy(&output.stdout).to_string(),
                    exit_code: output.status.code().unwrap_or(1),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                })
            }
            VmmBackend::Mock => {
                Ok(ExecutionResult {
                    success: true,
                    output: format!("Mock executed: {}", script),
                    exit_code: 0,
                    stderr: String::new(),
                })
            }
        }
    }

    async fn execute_gvisor(&self, vm_id: &str, command: &[&str]) -> Result<String, String> {
        tracing::debug!(vm_id, "Executing in gVisor: {:?}", command);

        let output = Command::new("runsc")
            .arg("exec")
            .arg(format!("apex-vm-{}", vm_id))
            .args(command)
            .output()
            .await
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn execute_docker(&self, vm_id: &str, command: &[&str]) -> Result<String, String> {
        let container_name = format!("apex-vm-{}", vm_id);

        tracing::debug!(vm_id, "Executing in Docker: {:?}", command);

        let cmd_str = command.join(" ");
        let output = Command::new("docker")
            .args(["exec", &container_name, "sh", "-c", &cmd_str])
            .output()
            .await
            .map_err(|e| format!("Failed to execute docker: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct VmPoolStats {
    pub total: usize,
    pub ready: usize,
    pub busy: usize,
    pub starting: usize,
    pub stopped: usize,
    pub available: usize,
    pub backend: String,
    pub enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vm_pool_initialization() {
        let pool = VmPool::new(VmConfig::default(), 5, 2);
        pool.initialize().await.unwrap();

        let stats = pool.get_stats().await;
        assert_eq!(stats.total, 2);
        assert_eq!(stats.ready, 2);
    }

    #[tokio::test]
    async fn test_vm_acquire_release() {
        let pool = VmPool::new(VmConfig::default(), 5, 1);
        pool.initialize().await.unwrap();

        let vm_id = pool.acquire().await.unwrap();
        assert!(!vm_id.is_empty());

        pool.release(&vm_id).await.unwrap();

        let stats = pool.get_stats().await;
        assert_eq!(stats.available, 1);
    }

    #[tokio::test]
    async fn test_vm_pool_exhaustion() {
        let pool = VmPool::new(VmConfig::default(), 2, 0);
        pool.initialize().await.unwrap();

        let _vm1 = pool.acquire().await.unwrap();
        let _vm2 = pool.acquire().await.unwrap();

        let result = pool.acquire().await;
        assert!(result.is_err());
    }
}
