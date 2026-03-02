use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::sync::RwLock;

const VM_STARTUP_TIMEOUT_SECS: u64 = 30;
const VM_IDLE_TIMEOUT_SECS: u64 = 300;

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
}

impl VmConfig {
    pub fn from_env() -> Self {
        Self {
            vcpu_count: std::env::var("APEX_VM_VCPU")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2),
            memory_mib: std::env::var("APEX_VM_MEMORY_MIB")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2048),
            kernel_path: std::env::var("APEX_VM_KERNEL").ok(),
            rootfs_path: std::env::var("APEX_VM_ROOTFS").ok(),
            firecracker_path: std::env::var("APEX_FIRECRACKER_PATH").ok(),
            runsc_path: std::env::var("APEX_RUNSC_PATH").ok(),
            docker_image: std::env::var("APEX_DOCKER_IMAGE").ok(),
            use_firecracker: std::env::var("APEX_USE_FIRECRACKER")
                .map(|v| v == "1")
                .unwrap_or(false),
            use_gvisor: std::env::var("APEX_USE_GVISOR")
                .map(|v| v == "1")
                .unwrap_or(false),
            use_docker: std::env::var("APEX_USE_DOCKER")
                .map(|v| v == "1")
                .unwrap_or(false),
        }
    }

    pub fn is_vm_available(&self) -> bool {
        self.use_firecracker
            || self.use_gvisor
            || self.use_docker
            || std::env::var("APEX_USE_FIRECRACKER").is_ok()
            || std::env::var("APEX_USE_GVISOR").is_ok()
            || std::env::var("APEX_USE_DOCKER").is_ok()
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

        tracing::info!("VM pool initialized");
        Ok(())
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

        tracing::info!(vm_id = id, "Starting Firecracker VM");

        let socket_path = instance
            .socket_path
            .as_ref()
            .ok_or_else(|| "Socket path not set".to_string())?;

        let mut cmd = Command::new(firecracker_path);
        cmd.arg("--api-sock")
            .arg(socket_path)
            .arg("--kernel")
            .arg(kernel_path)
            .arg("--root-drive")
            .arg(
                self.config
                    .rootfs_path
                    .as_deref()
                    .unwrap_or("/tmp/rootfs.ext4"),
            )
            .arg("--vcpu")
            .arg(self.config.vcpu_count.to_string())
            .arg("--memory")
            .arg(self.config.memory_mib.to_string());

        cmd.spawn()
            .map_err(|e| format!("Failed to spawn Firecracker: {}", e))?;

        tokio::time::sleep(Duration::from_secs(2)).await;
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

        tracing::info!(vm_id = id, image = image, "Starting Docker container");

        let container_name = format!("apex-{}", id);

        let output = Command::new("docker")
            .args(["run", "-d", "--name", &container_name, image])
            .output()
            .await
            .map_err(|e| format!("Failed to run docker: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Docker run failed: {}", stderr));
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
        instance.mark_ready();

        tracing::info!(vm_id = id, "Docker container started");
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

        let output = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "echo '{}' | nc -U {}",
                command.join(" "),
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
        let container_name = format!("apex-{}", vm_id);

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
