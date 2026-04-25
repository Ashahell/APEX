use std::process::Command;
use std::time::Duration;
use tokio::time::timeout;

pub struct ShellHooks {
    pre_tool_call: Option<String>,
    post_tool_call: Option<String>,
    session_start: Option<String>,
    session_end: Option<String>,
    timeout_secs: u64,
}

impl ShellHooks {
    pub fn new() -> Self {
        let config = crate::unified_config::AppConfig::global();
        ShellHooks {
            pre_tool_call: config.shell_hooks.pre_tool_call.clone(),
            post_tool_call: config.shell_hooks.post_tool_call.clone(),
            session_start: config.shell_hooks.session_start.clone(),
            session_end: config.shell_hooks.session_end.clone(),
            timeout_secs: config.shell_hooks.timeout_secs,
        }
    }
    
    pub async fn run_hook(&self, hook_name: &str, context: serde_json::Value) -> Option<String> {
        let hook = match hook_name {
            "pre_tool_call" => &self.pre_tool_call,
            "post_tool_call" => &self.post_tool_call,
            "session_start" => &self.session_start,
            "session_end" => &self.session_end,
            _ => return None,
        };
        
        let script = match hook {
            Some(s) => s,
            None => return None,
        };
        
        let result = timeout(
            Duration::from_secs(self.timeout_secs),
            run_hook_script(script, context)
        ).await;
        
        match result {
            Ok(Ok(output)) => Some(output),
            Ok(Err(e)) => {
                tracing::warn!(hook = hook_name, error = %e, "Hook failed");
                None
            }
            Err(_) => {
                tracing::warn!(hook = hook_name, "Hook timed out");
                None
            }
        }
    }
}

async fn run_hook_script(script: &str, context: serde_json::Value) -> Result<String, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(script)
        .output()
        .map_err(|e| e.to_string())?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}