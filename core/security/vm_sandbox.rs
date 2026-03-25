// Minimal VM sandbox config skeleton
pub struct VmSandboxConfig {
    pub cpu_limit: u32,
    pub memory_limit_mb: u32,
    pub timeout_sec: u64,
    pub network_isolation: bool,
}

impl VmSandboxConfig {
    pub fn default() -> Self {
        Self {
            cpu_limit: 2,
            memory_limit_mb: 2048,
            timeout_sec: 3600,
            network_isolation: true,
        }
    }
}
