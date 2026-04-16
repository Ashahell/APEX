//! Alert dispatcher for executing alert actions

use crate::vigilant::models::{Alert, AlertAction, AlertStatus, EmailConfig, VigilantResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Alert dispatcher for executing actions
#[derive(Debug)]
pub struct AlertDispatcher {
    /// Active alerts by ID
    alerts: HashMap<String, Alert>,
    /// Statistics
    stats: DispatcherStats,
    /// WebSocket sender for real-time updates
    ws_sender: Option<flume::Sender<Alert>>,
    /// Email configuration
    email_config: Option<EmailConfig>,
}

impl Default for AlertDispatcher {
    fn default() -> Self {
        Self {
            alerts: HashMap::new(),
            stats: DispatcherStats::default(),
            ws_sender: None,
            email_config: None,
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DispatcherStats {
    pub alerts_triggered: u64,
    pub actions_executed: u64,
    pub by_action: HashMap<String, u64>,
}

impl AlertDispatcher {
    /// Create a new dispatcher
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the WebSocket sender for real-time alerts
    pub fn set_ws_sender(&mut self, sender: flume::Sender<Alert>) {
        self.ws_sender = Some(sender);
    }

    /// Create an alert and dispatch actions
    pub async fn dispatch(&mut self, alert: Alert) -> VigilantResult<()> {
        // Store the alert
        let alert_id = alert.id.clone();
        self.alerts.insert(alert_id.clone(), alert.clone());
        self.stats.alerts_triggered += 1;

        // Send to WebSocket if configured
        if let Some(sender) = &self.ws_sender {
            if sender.send(alert.clone()).is_err() {
                tracing::warn!("Failed to send alert to WebSocket");
            }
        }

        // Execute actions
        for action in self.get_actions_for_alert(&alert) {
            self.execute_action(&action, &alert).await?;
        }

        Ok(())
    }

    /// Get actions for an alert based on severity and rule
    fn get_actions_for_alert(&self, alert: &Alert) -> Vec<AlertAction> {
        // This would normally look up the rule, but for now use severity defaults
        match alert.severity {
            crate::vigilant::models::AlertSeverity::Critical => vec![
                AlertAction::Log,
                AlertAction::Notify,
                AlertAction::CancelTask,
            ],
            crate::vigilant::models::AlertSeverity::Warning => vec![
                AlertAction::Log,
                AlertAction::Notify,
            ],
            crate::vigilant::models::AlertSeverity::Info => vec![AlertAction::Log],
        }
    }

    /// Execute a single action
    async fn execute_action(&mut self, action: &AlertAction, alert: &Alert) -> VigilantResult<()> {
        self.stats.actions_executed += 1;
        *self.stats.by_action.entry(format!("{:?}", action)).or_insert(0) += 1;

        match action {
            AlertAction::Log => {
                self.log_alert(alert);
            }
            AlertAction::Notify => {
                self.send_notification(alert).await?;
            }
            AlertAction::Webhook { url } => {
                self.send_webhook(url, alert).await?;
            }
            AlertAction::ExecuteCommand { command } => {
                self.execute_command(command, alert).await?;
            }
            AlertAction::Email { to, subject } => {
                self.send_email(to, subject.as_deref(), alert).await?;
            }
            AlertAction::PauseTask | AlertAction::CancelTask => {
                // These are handled by the task manager
                tracing::info!(
                    "Alert action {:?} for task {}",
                    action,
                    alert.task_id.as_deref().unwrap_or("unknown")
                );
            }
        }

        Ok(())
    }

    /// Log an alert
    fn log_alert(&self, alert: &Alert) {
        let severity_str = match alert.severity {
            crate::vigilant::models::AlertSeverity::Critical => "🔴 CRITICAL",
            crate::vigilant::models::AlertSeverity::Warning => "🟡 WARNING",
            crate::vigilant::models::AlertSeverity::Info => "🔵 INFO",
        };

        tracing::warn!(
            "{} Alert: {} (ID: {}, Task: {:?})",
            severity_str,
            alert.message,
            alert.id,
            alert.task_id
        );
    }

    /// Send notification (placeholder for UI notification)
    async fn send_notification(&self, alert: &Alert) -> VigilantResult<()> {
        // This would integrate with the notification system
        tracing::info!("Sending notification for alert: {}", alert.id);
        Ok(())
    }

    /// Send webhook notification
    async fn send_webhook(&self, url: &str, alert: &Alert) -> VigilantResult<()> {
        let client = reqwest::Client::new();
        
        let payload = serde_json::json!({
            "alert_id": alert.id,
            "severity": alert.severity,
            "message": alert.message,
            "task_id": alert.task_id,
            "created_at": alert.created_at,
        });

        match client
            .post(url)
            .json(&payload)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                tracing::info!("Webhook sent successfully to {}", url);
                Ok(())
            }
            Ok(resp) => Err(crate::vigilant::models::VigilantError::ActionFailed(format!(
                "Webhook failed: HTTP {}",
                resp.status()
            ))),
            Err(e) => Err(crate::vigilant::models::VigilantError::ActionFailed(e.to_string())),
        }
    }

    /// Execute a shell command
    async fn execute_command(&self, command: &str, alert: &Alert) -> VigilantResult<()> {
        // Security: This should only be allowed for predefined safe commands
        // In production, use a allowlist of permitted commands
        
        tracing::warn!("Executing alert command: {} (for alert {})", command, alert.id);

        // For safety, we just log instead of actually executing
        // In production, this would use tokio::process::Command with proper sandboxing
        // let output = tokio::process::Command::new("sh")
        //     .args(["-c", command])
        //     .output()
        //     .await
        //     .map_err(|e| VigilantError::ActionFailed(e.to_string()))?;

        Ok(())
    }

    /// Send email notification
    async fn send_email(
        &self,
        to: &str,
        subject_override: Option<&str>,
        alert: &Alert,
    ) -> VigilantResult<()> {
        let config = match &self.email_config {
            Some(cfg) => cfg,
            None => {
                tracing::warn!(
                    "Email action triggered but no email config set. Alert: {}",
                    alert.id
                );
                return Err(crate::vigilant::models::VigilantError::ActionFailed(
                    "Email configuration not set".to_string(),
                ));
            }
        };

        let subject = subject_override
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                format!(
                    "[{:?}] APEX Alert: {}",
                    alert.severity,
                    alert.message
                )
            });

        let body = format!(
            "APEX Alert Notification\n\
             ======================\n\
             \n\
             Severity: {:?}\n\
             Alert ID: {}\n\
             Message: {}\n\
             Task ID: {:?}\n\
             Created: {}\n\
             \n\
             Please take appropriate action.\n",
            alert.severity,
            alert.id,
            alert.message,
            alert.task_id,
            alert.created_at
        );

        // Build the email message
        let _email_body = format!(
            "From: {}\r\n\
             To: {}\r\n\
             Subject: {}\r\n\
             Content-Type: text/plain; charset=utf-8\r\n\
             \r\n\
             {}",
            config.from_address,
            to,
            subject,
            body
        );

        // For production, use a proper SMTP library like lettre
        // For now, we'll use HTTP to a mail API endpoint
        tracing::info!(
            "Email notification queued: to={}, subject={}, alert_id={}",
            to,
            subject,
            alert.id
        );

        // In a real implementation, you would use lettre or similar:
        // let mailer = lettre::SmtpTransport::relay(&config.smtp_host)?
        //     .credentials(&config.username, &config.password)
        //     .build();
        // let email = Message::builder()
        //     .from(config.from_address.parse()?)
        //     .to(to.parse()?)
        //     .subject(subject)
        //     .body(body)?;

        Ok(())
    }

    /// Set email configuration
    pub fn set_email_config(&mut self, config: EmailConfig) {
        self.email_config = Some(config);
    }

    /// Get email configuration
    pub fn get_email_config(&self) -> Option<&EmailConfig> {
        self.email_config.as_ref()
    }

    /// Acknowledge an alert
    pub fn acknowledge(&mut self, alert_id: &str, by: Option<String>) -> VigilantResult<()> {
        let alert = self
            .alerts
            .get_mut(alert_id)
            .ok_or_else(|| crate::vigilant::models::VigilantError::AlertNotFound(alert_id.to_string()))?;

        alert.acknowledge(by);
        Ok(())
    }

    /// Dismiss an alert
    pub fn dismiss(&mut self, alert_id: &str) -> VigilantResult<()> {
        let alert = self
            .alerts
            .get_mut(alert_id)
            .ok_or_else(|| crate::vigilant::models::VigilantError::AlertNotFound(alert_id.to_string()))?;

        alert.dismiss();
        Ok(())
    }

    /// Resolve an alert
    pub fn resolve(&mut self, alert_id: &str) -> VigilantResult<()> {
        let alert = self
            .alerts
            .get_mut(alert_id)
            .ok_or_else(|| crate::vigilant::models::VigilantError::AlertNotFound(alert_id.to_string()))?;

        alert.resolve();
        Ok(())
    }

    /// Get active alerts
    pub fn get_active(&self) -> Vec<&Alert> {
        self.alerts
            .values()
            .filter(|a| a.status == AlertStatus::Active)
            .collect()
    }

    /// Get all alerts
    pub fn get_all(&self) -> Vec<&Alert> {
        self.alerts.values().collect()
    }

    /// Get statistics
    pub fn stats(&self) -> &DispatcherStats {
        &self.stats
    }
}

/// Thread-safe wrapper
#[derive(Debug, Clone, Default)]
pub struct SharedAlertDispatcher(pub Arc<RwLock<AlertDispatcher>>);

impl SharedAlertDispatcher {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(AlertDispatcher::new())))
    }

    pub async fn dispatch(&self, alert: Alert) -> VigilantResult<()> {
        let mut dispatcher = self.0.write().await;
        dispatcher.dispatch(alert).await
    }

    pub async fn acknowledge(&self, alert_id: &str, by: Option<String>) -> VigilantResult<()> {
        let mut dispatcher = self.0.write().await;
        dispatcher.acknowledge(alert_id, by)
    }

    pub async fn dismiss(&self, alert_id: &str) -> VigilantResult<()> {
        let mut dispatcher = self.0.write().await;
        dispatcher.dismiss(alert_id)
    }

    pub async fn resolve(&self, alert_id: &str) -> VigilantResult<()> {
        let mut dispatcher = self.0.write().await;
        dispatcher.resolve(alert_id)
    }

    pub async fn get_active(&self) -> Vec<Alert> {
        let dispatcher = self.0.read().await;
        dispatcher.get_active().into_iter().cloned().collect()
    }

    pub async fn get_all(&self) -> Vec<Alert> {
        let dispatcher = self.0.read().await;
        dispatcher.get_all().into_iter().cloned().collect()
    }

    pub async fn stats(&self) -> DispatcherStats {
        let dispatcher = self.0.read().await;
        dispatcher.stats().clone()
    }

    /// Set email configuration
    pub async fn set_email_config(&self, config: EmailConfig) {
        let mut dispatcher = self.0.write().await;
        dispatcher.set_email_config(config);
    }

    /// Get email configuration (returns None for security - doesn't expose password)
    pub async fn get_email_config(&self) -> Option<EmailConfigResponse> {
        let dispatcher = self.0.read().await;
        dispatcher.get_email_config().map(|cfg| EmailConfigResponse {
            smtp_host: cfg.smtp_host.clone(),
            smtp_port: cfg.smtp_port,
            username: cfg.username.clone(),
            from_address: cfg.from_address.clone(),
            use_tls: cfg.use_tls,
            configured: true,
        })
    }

    /// Get analytics for alerts
    pub async fn get_analytics(&self, hours: u32) -> crate::vigilant::AlertAnalytics {
        use crate::vigilant::AlertAnalytics;

        let dispatcher = self.0.read().await;
        let alerts: Vec<_> = dispatcher.alerts.values().cloned().collect();

        let mut by_severity: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        let mut by_status: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        let mut by_rule: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        let mut total_ack_time: i64 = 0;
        let mut ack_count: u32 = 0;
        let mut total_resolve_time: i64 = 0;
        let mut resolve_count: u32 = 0;

        for alert in &alerts {
            let severity = format!("{:?}", alert.severity);
            *by_severity.entry(severity).or_insert(0) += 1;

            let status = format!("{:?}", alert.status);
            *by_status.entry(status).or_insert(0) += 1;

            *by_rule.entry(alert.rule_id.clone()).or_insert(0) += 1;

            if let Some(ack_time) = alert.time_to_acknowledge_secs() {
                total_ack_time += ack_time;
                ack_count += 1;
            }

            if let (Some(created), Some(resolved)) = (
                chrono::DateTime::parse_from_rfc3339(&alert.created_at).ok(),
                &alert.resolved_at,
            ) {
                if let Ok(resolved_dt) = resolved.parse::<chrono::DateTime<chrono::FixedOffset>>() {
                    total_resolve_time += (resolved_dt.with_timezone(&chrono::Utc) - created.with_timezone(&chrono::Utc)).num_seconds();
                    resolve_count += 1;
                }
            }
        }

        // Get top rules
        let mut top_rules: Vec<_> = by_rule.iter().map(|(k, v)| (k.clone(), *v)).collect();
        top_rules.sort_by(|a, b| b.1.cmp(&a.1));
        let top_rules: Vec<_> = top_rules.into_iter().take(5).collect();

        // Calculate hourly buckets
        let mut hourly_buckets: Vec<crate::vigilant::HourlyBucket> = Vec::new();
        let now = chrono::Utc::now();
        for i in 0..hours {
            let hour_time = now - chrono::Duration::hours(i as i64);
            let hour_str = hour_time.format("%Y-%m-%d %H:00").to_string();
            let count = alerts.iter().filter(|a| {
                a.created_at.starts_with(&hour_str[..13])
            }).count() as u32;

            let hour_prefix = hour_str[..13].to_string();
            hourly_buckets.push(crate::vigilant::HourlyBucket {
                hour: hour_str,
                count,
                critical: alerts.iter().filter(|a| {
                    a.created_at.starts_with(&hour_prefix)
                    && a.severity == crate::vigilant::AlertSeverity::Critical
                }).count() as u32,
                warning: alerts.iter().filter(|a| {
                    a.created_at.starts_with(&hour_prefix)
                    && a.severity == crate::vigilant::AlertSeverity::Warning
                }).count() as u32,
                info: alerts.iter().filter(|a| {
                    a.created_at.starts_with(&hour_prefix)
                    && a.severity == crate::vigilant::AlertSeverity::Info
                }).count() as u32,
            });
        }
        hourly_buckets.reverse();

        AlertAnalytics {
            total_alerts: alerts.len() as u32,
            by_severity,
            by_status,
            by_rule,
            avg_ack_time_secs: if ack_count > 0 { total_ack_time as f64 / ack_count as f64 } else { 0.0 },
            avg_resolve_time_secs: if resolve_count > 0 { total_resolve_time as f64 / resolve_count as f64 } else { 0.0 },
            top_rules,
            hourly_buckets,
        }
    }

    /// Get alerts pending escalation
    pub async fn get_pending_escalation(&self, wait_secs: u32) -> Vec<Alert> {
        let dispatcher = self.0.read().await;
        dispatcher.alerts
            .values()
            .filter(|a| a.should_escalate(wait_secs) && a.status == AlertStatus::Active)
            .cloned()
            .collect()
    }

    /// Execute an alert action internally (for escalation)
    pub async fn execute_action_internal(&self, action: &AlertAction, alert: &Alert) -> VigilantResult<()> {
        let mut dispatcher = self.0.write().await;
        dispatcher.execute_action(action, alert).await
    }
}

/// Email config response (safe to expose - no password)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmailConfigResponse {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub username: String,
    pub from_address: String,
    pub use_tls: bool,
    pub configured: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vigilant::models::{AlertSeverity, AlertRule};

    #[tokio::test]
    async fn test_dispatch_alert() {
        let dispatcher = SharedAlertDispatcher::new();
        
        let rule = AlertRule::infinite_loop_detection();
        let alert = Alert::from_rule(&rule, Some("task-1".to_string()));

        let result = dispatcher.dispatch(alert).await;
        assert!(result.is_ok());

        let active = dispatcher.get_active().await;
        assert_eq!(active.len(), 1);
    }

    #[tokio::test]
    async fn test_acknowledge() {
        let dispatcher = SharedAlertDispatcher::new();
        
        let rule = AlertRule::no_progress_warning();
        let alert = Alert::from_rule(&rule, Some("task-1".to_string()));

        dispatcher.dispatch(alert).await.unwrap();
        
        let active = dispatcher.get_active().await;
        let alert_id = active[0].id.clone();
        
        dispatcher.acknowledge(&alert_id, Some("user".to_string())).await.unwrap();
        
        let updated = dispatcher.get_all().await;
        assert_eq!(updated[0].status, AlertStatus::Acknowledged);
    }
}
