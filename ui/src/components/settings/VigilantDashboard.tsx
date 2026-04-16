import { useState, useEffect, useCallback } from 'react';
import { apiGet, apiPost, apiPut, apiDelete } from '../../lib/api';

// ============ Types ============

export type AlertSeverity = 'Info' | 'Warning' | 'Critical';

export type AlertStatus = 'Active' | 'Acknowledged' | 'Dismissed' | 'Resolved';

export interface AlertAction {
  type: 'Log' | 'Notify' | 'PauseTask' | 'CancelTask' | 'Webhook' | 'ExecuteCommand' | 'Email';
  url?: string;
  command?: string;
  to?: string;
  subject?: string;
}

export interface AlertType {
  type: 'InfiniteLoop' | 'NoProgress' | 'ResourceExhaustion' | 'TimeoutWarning' | 'PatternDetected' | 'ErrorSpike' | 'HighMemoryUsage' | 'LLMUnavailable' | 'ExecutionPoolExhausted' | 'AwaitingConfirmation';
  task_id?: string;
  iterations?: number;
  steps?: number;
  resource?: string;
  remaining_secs?: number;
  pattern?: string;
  error_count?: number;
  percentage?: number;
  wait_secs?: number;
}

export interface AlertRule {
  id: string;
  name: string;
  alert_type: AlertType;
  severity: AlertSeverity;
  cooldown_secs: number;
  actions: AlertAction[];
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface Alert {
  id: string;
  rule_id: string;
  alert_type: AlertType;
  severity: AlertSeverity;
  task_id?: string;
  message: string;
  payload?: string;
  status: AlertStatus;
  created_at: string;
  acknowledged_at?: string;
  acknowledged_by?: string;
  resolved_at?: string;
  escalation_level?: number;
  escalated_at?: string;
  last_escalation_at?: string;
}

export interface VigilantStats {
  total_alerts: number;
  active_alerts: number;
  by_severity: Record<string, number>;
  by_rule: Record<string, number>;
  acknowledged_today: number;
  resolved_today: number;
}

export interface AlertRuleCreate {
  name: string;
  alert_type: AlertType;
  severity: AlertSeverity;
  cooldown_secs: number;
  actions: AlertAction[];
}

// v1.9.0: Analytics types
export interface HourlyBucket {
  hour: string;
  count: number;
  critical: number;
  warning: number;
  info: number;
}

export interface AlertAnalytics {
  total_alerts: number;
  by_severity: Record<string, number>;
  by_status: Record<string, number>;
  by_rule: Record<string, number>;
  avg_ack_time_secs: number;
  avg_resolve_time_secs: number;
  top_rules: [string, number][];
  hourly_buckets: HourlyBucket[];
}

// v1.9.0: Escalation types
export interface EscalationLevel {
  level: number;
  wait_secs: number;
  actions: AlertAction[];
}

export interface EscalationConfig {
  enabled: boolean;
  max_level: number;
  levels: EscalationLevel[];
  default_wait_secs: number;
}

// v1.9.0: Pattern suggestion types
export interface AlertRuleSuggestion {
  pattern_type: string;
  suggested_name: string;
  suggested_severity: AlertSeverity;
  suggested_actions: AlertAction[];
  cooldown_secs: number;
  confidence: number;
  reason: string;
}

// ============ Component ============

export function VigilantDashboard() {
  const [rules, setRules] = useState<AlertRule[]>([]);
  const [alerts, setAlerts] = useState<Alert[]>([]);
  const [stats, setStats] = useState<VigilantStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [activeView, setActiveView] = useState<'alerts' | 'rules' | 'create' | 'analytics' | 'escalation' | 'patterns'>('alerts');
  const [severityFilter, setSeverityFilter] = useState<AlertSeverity | 'all'>('all');
  const [statusFilter, setStatusFilter] = useState<AlertStatus | 'all'>('all');
  
  // v1.9.0: Analytics state
  const [analytics, setAnalytics] = useState<AlertAnalytics | null>(null);
  const [analyticsHours, setAnalyticsHours] = useState(24);
  
  // v1.9.0: Escalation state
  const [escalationConfig, setEscalationConfig] = useState<EscalationConfig>({
    enabled: false,
    max_level: 3,
    levels: [
      { level: 1, wait_secs: 300, actions: [{ type: 'Notify' }, { type: 'Email', to: '', subject: 'Alert Unacknowledged' }] },
      { level: 2, wait_secs: 600, actions: [{ type: 'ExecuteCommand', command: 'echo ALERT' }] },
      { level: 3, wait_secs: 0, actions: [{ type: 'CancelTask' }] },
    ],
    default_wait_secs: 300,
  });
  const [pendingEscalation, setPendingEscalation] = useState<Alert[]>([]);
  
  // v1.9.0: Pattern suggestions state
  const [patternSuggestions, setPatternSuggestions] = useState<AlertRuleSuggestion[]>([]);
  const [patternLoading, setPatternLoading] = useState(false);
  
  // Create form state
  const [newRule, setNewRule] = useState<Partial<AlertRuleCreate>>({
    name: '',
    alert_type: { type: 'NoProgress' },
    severity: 'Warning',
    cooldown_secs: 60,
    actions: [{ type: 'Notify' }],
  });

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [rulesRes, alertsRes, statsRes] = await Promise.all([
        apiGet('/api/v1/vigilant/rules'),
        apiGet('/api/v1/vigilant/alerts?limit=50'),
        apiGet('/api/v1/vigilant/stats'),
      ]);
      
      if (rulesRes.ok) {
        const data = await rulesRes.json();
        setRules(Array.isArray(data) ? data : []);
      }
      if (alertsRes.ok) {
        const data = await alertsRes.json();
        setAlerts(Array.isArray(data) ? data : []);
      }
      if (statsRes.ok) {
        const data = await statsRes.json();
        setStats(data);
      }
    } catch (err) {
      console.error('Failed to load vigilant data:', err);
    } finally {
      setLoading(false);
    }
  }, []);
  
  // v1.9.0: Load analytics data
  const loadAnalytics = useCallback(async () => {
    try {
      const res = await apiGet(`/api/v1/vigilant/analytics?hours=${analyticsHours}`);
      if (res.ok) {
        const data = await res.json();
        setAnalytics(data.analytics);
      }
    } catch (err) {
      console.error('Failed to load analytics:', err);
    }
  }, [analyticsHours]);
  
  // v1.9.0: Load pending escalations
  const loadPendingEscalations = useCallback(async () => {
    try {
      const res = await apiGet('/api/v1/vigilant/escalation/pending?wait_secs=300');
      if (res.ok) {
        const data = await res.json();
        setPendingEscalation(data.alerts || []);
      }
    } catch (err) {
      console.error('Failed to load pending escalations:', err);
    }
  }, []);
  
  // v1.9.0: Process escalations
  const handleProcessEscalations = async () => {
    try {
      const res = await apiPost('/api/v1/vigilant/escalation/process', {
        wait_secs: 300,
        max_level: 3,
        escalation_actions: escalationConfig.levels.flatMap(l => l.actions),
      });
      if (res.ok) {
        loadPendingEscalations();
        loadData();
      }
    } catch (err) {
      console.error('Failed to process escalations:', err);
    }
  };
  
  // v1.9.0: Load pattern suggestions
  const loadPatternSuggestions = useCallback(async () => {
    setPatternLoading(true);
    try {
      const res = await apiGet('/api/v1/vigilant/patterns/suggestions');
      if (res.ok) {
        const data = await res.json();
        setPatternSuggestions(data.suggestions || []);
      }
    } catch (err) {
      console.error('Failed to load pattern suggestions:', err);
    } finally {
      setPatternLoading(false);
    }
  }, []);
  
  // v1.9.0: Create rule from pattern suggestion
  const handleCreateFromSuggestion = async (suggestion: AlertRuleSuggestion) => {
    try {
      const res = await apiPost('/api/v1/vigilant/patterns/create-rule', {
        pattern_type: suggestion.pattern_type,
        name: suggestion.suggested_name,
        severity: suggestion.suggested_severity,
        cooldown_secs: suggestion.cooldown_secs,
        actions: suggestion.suggested_actions,
      });
      if (res.ok) {
        loadData();
        setActiveView('rules');
      }
    } catch (err) {
      console.error('Failed to create rule from suggestion:', err);
    }
  };

  useEffect(() => {
    loadData();
  }, [loadData]);
  
  // v1.9.0: Load analytics when view changes
  useEffect(() => {
    if (activeView === 'analytics') {
      loadAnalytics();
    }
  }, [activeView, loadAnalytics]);
  
  // v1.9.0: Load pending escalations and pattern suggestions
  useEffect(() => {
    if (activeView === 'escalation') {
      loadPendingEscalations();
    }
    if (activeView === 'patterns') {
      loadPatternSuggestions();
    }
  }, [activeView, loadPendingEscalations, loadPatternSuggestions]);

  const handleCreateRule = async () => {
    if (!newRule.name || !newRule.alert_type) return;
    
    try {
      const res = await apiPost('/api/v1/vigilant/rules', {
        name: newRule.name,
        alert_type: newRule.alert_type,
        severity: newRule.severity,
        cooldown_secs: newRule.cooldown_secs,
        actions: newRule.actions,
      });
      
      if (res.ok) {
        setNewRule({
          name: '',
          alert_type: { type: 'NoProgress' },
          severity: 'Warning',
          cooldown_secs: 60,
          actions: [{ type: 'Notify' }],
        });
        setActiveView('rules');
        loadData();
      }
    } catch (err) {
      console.error('Failed to create rule:', err);
    }
  };

  const handleToggleRule = async (id: string, enabled: boolean) => {
    try {
      await apiPut(`/api/v1/vigilant/rules/${id}`, { enabled });
      loadData();
    } catch (err) {
      console.error('Failed to toggle rule:', err);
    }
  };

  const handleDeleteRule = async (id: string) => {
    try {
      await apiDelete(`/api/v1/vigilant/rules/${id}`);
      loadData();
    } catch (err) {
      console.error('Failed to delete rule:', err);
    }
  };

  const handleAcknowledgeAlert = async (id: string) => {
    try {
      await apiPost(`/api/v1/vigilant/alerts/${id}/acknowledge`, {});
      loadData();
    } catch (err) {
      console.error('Failed to acknowledge alert:', err);
    }
  };

  const handleDismissAlert = async (id: string) => {
    try {
      await apiPost(`/api/v1/vigilant/alerts/${id}/dismiss`, {});
      loadData();
    } catch (err) {
      console.error('Failed to dismiss alert:', err);
    }
  };

  const filteredAlerts = alerts.filter(a => {
    if (severityFilter !== 'all' && a.severity !== severityFilter) return false;
    if (statusFilter !== 'all' && a.status !== statusFilter) return false;
    return true;
  });

  const getSeverityColor = (severity: AlertSeverity) => {
    switch (severity) {
      case 'Critical': return 'bg-red-500/20 text-red-400 border-red-500/50';
      case 'Warning': return 'bg-yellow-500/20 text-yellow-400 border-yellow-500/50';
      case 'Info': return 'bg-blue-500/20 text-blue-400 border-blue-500/50';
    }
  };

  const getStatusColor = (status: AlertStatus) => {
    switch (status) {
      case 'Active': return 'bg-orange-500/20 text-orange-400';
      case 'Acknowledged': return 'bg-blue-500/20 text-blue-400';
      case 'Dismissed': return 'bg-gray-500/20 text-gray-400';
      case 'Resolved': return 'bg-green-500/20 text-green-400';
    }
  };

  const formatAlertType = (type: AlertType) => {
    const base = type.type.replace(/([A-Z])/g, ' $1').trim();
    const details = [];
    if (type.task_id) details.push(`task: ${type.task_id}`);
    if (type.iterations) details.push(`${type.iterations} iterations`);
    if (type.steps) details.push(`${type.steps} steps`);
    if (type.error_count) details.push(`${type.error_count} errors`);
    if (type.percentage) details.push(`${type.percentage}%`);
    if (type.pattern) details.push(`"${type.pattern}"`);
    return details.length > 0 ? `${base} (${details.join(', ')})` : base;
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-[var(--color-text-muted)]">Loading vigilant data...</div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h2 className="text-2xl font-semibold">Vigilant Mode</h2>
        <p className="text-[var(--color-text-muted)]">
          Alert rules, threshold monitoring, and automated responses
        </p>
      </div>

      {/* Stats Cards */}
      {stats && (
        <div className="grid grid-cols-5 gap-4">
          <div className="border rounded-lg p-4 bg-[var(--color-panel)]">
            <div className="text-2xl font-bold">{stats.total_alerts}</div>
            <div className="text-sm text-[var(--color-text-muted)]">Total Alerts</div>
          </div>
          <div className="border rounded-lg p-4 bg-[var(--color-panel)]">
            <div className="text-2xl font-bold text-orange-400">{stats.active_alerts}</div>
            <div className="text-sm text-[var(--color-text-muted)]">Active</div>
          </div>
          <div className="border rounded-lg p-4 bg-[var(--color-panel)]">
            <div className="text-2xl font-bold text-red-400">{stats.by_severity?.Critical || 0}</div>
            <div className="text-sm text-[var(--color-text-muted)]">Critical</div>
          </div>
          <div className="border rounded-lg p-4 bg-[var(--color-panel)]">
            <div className="text-2xl font-bold text-yellow-400">{stats.by_severity?.Warning || 0}</div>
            <div className="text-sm text-[var(--color-text-muted)]">Warning</div>
          </div>
          <div className="border rounded-lg p-4 bg-[var(--color-panel)]">
            <div className="text-2xl font-bold text-green-400">{stats.resolved_today}</div>
            <div className="text-sm text-[var(--color-text-muted)]">Resolved Today</div>
          </div>
        </div>
      )}

      {/* View Tabs */}
      <div className="flex flex-wrap gap-4 border-b">
        <button
          onClick={() => setActiveView('alerts')}
          className={`px-4 py-2 transition-colors ${
            activeView === 'alerts'
              ? 'border-b-2 border-[#00d4ff] text-[#00d4ff]'
              : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'
          }`}
        >
          Active Alerts
          {stats && stats.active_alerts > 0 && (
            <span className="ml-2 px-2 py-0.5 bg-orange-500/20 text-orange-400 rounded-full text-xs">
              {stats.active_alerts}
            </span>
          )}
        </button>
        <button
          onClick={() => setActiveView('rules')}
          className={`px-4 py-2 transition-colors ${
            activeView === 'rules'
              ? 'border-b-2 border-[#00d4ff] text-[#00d4ff]'
              : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'
          }`}
        >
          Alert Rules
        </button>
        <button
          onClick={() => setActiveView('create')}
          className={`px-4 py-2 transition-colors ${
            activeView === 'create'
              ? 'border-b-2 border-[#00d4ff] text-[#00d4ff]'
              : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'
          }`}
        >
          + New Rule
        </button>
        {/* v1.9.0: New tabs */}
        <button
          onClick={() => setActiveView('analytics')}
          className={`px-4 py-2 transition-colors ${
            activeView === 'analytics'
              ? 'border-b-2 border-[#00d4ff] text-[#00d4ff]'
              : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'
          }`}
        >
          📊 Analytics
        </button>
        <button
          onClick={() => setActiveView('escalation')}
          className={`px-4 py-2 transition-colors ${
            activeView === 'escalation'
              ? 'border-b-2 border-[#00d4ff] text-[#00d4ff]'
              : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'
          }`}
        >
          ⬆️ Escalation
        </button>
        <button
          onClick={() => setActiveView('patterns')}
          className={`px-4 py-2 transition-colors ${
            activeView === 'patterns'
              ? 'border-b-2 border-[#00d4ff] text-[#00d4ff]'
              : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'
          }`}
        >
          🔍 Pattern Suggestions
        </button>
      </div>

      {/* Alerts View */}
      {activeView === 'alerts' && (
        <div className="space-y-4">
          {/* Filters */}
          <div className="flex items-center gap-4">
            <select
              value={severityFilter}
              onChange={(e) => setSeverityFilter(e.target.value as AlertSeverity | 'all')}
              className="px-3 py-2 rounded border bg-[var(--color-panel)] border-[var(--color-border)]"
            >
              <option value="all">All Severities</option>
              <option value="Critical">Critical</option>
              <option value="Warning">Warning</option>
              <option value="Info">Info</option>
            </select>
            <select
              value={statusFilter}
              onChange={(e) => setStatusFilter(e.target.value as AlertStatus | 'all')}
              className="px-3 py-2 rounded border bg-[var(--color-panel)] border-[var(--color-border)]"
            >
              <option value="all">All Statuses</option>
              <option value="Active">Active</option>
              <option value="Acknowledged">Acknowledged</option>
              <option value="Dismissed">Dismissed</option>
              <option value="Resolved">Resolved</option>
            </select>
            <button
              onClick={loadData}
              className="px-3 py-2 bg-[var(--color-muted)] rounded hover:bg-[var(--color-muted)]/80 transition-colors text-sm"
            >
              Refresh
            </button>
          </div>
          
          <div className="space-y-2">
            {filteredAlerts.length === 0 ? (
              <div className="border rounded-lg p-8 text-center text-[var(--color-text-muted)]">
                No alerts match your filters
              </div>
            ) : (
              filteredAlerts.map((alert) => (
                <div
                  key={alert.id}
                  className="border rounded-lg p-4 bg-[var(--color-panel)]"
                >
                  <div className="flex items-start justify-between gap-4">
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-2">
                        <span className={`px-2 py-0.5 rounded text-xs border ${getSeverityColor(alert.severity)}`}>
                          {alert.severity}
                        </span>
                        <span className={`px-2 py-0.5 rounded text-xs ${getStatusColor(alert.status)}`}>
                          {alert.status}
                        </span>
                        {alert.task_id && (
                          <span className="font-mono text-xs text-[var(--color-text-muted)]">
                            {alert.task_id}
                          </span>
                        )}
                      </div>
                      <div className="text-sm mb-1">{alert.message}</div>
                      <div className="text-xs text-[var(--color-text-muted)]">
                        {formatAlertType(alert.alert_type)} • {new Date(alert.created_at).toLocaleString()}
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      {alert.status === 'Active' && (
                        <>
                          <button
                            onClick={() => handleAcknowledgeAlert(alert.id)}
                            className="px-3 py-1.5 bg-blue-500/20 text-blue-400 rounded hover:bg-blue-500/30 transition-colors text-sm"
                          >
                            Acknowledge
                          </button>
                          <button
                            onClick={() => handleDismissAlert(alert.id)}
                            className="px-3 py-1.5 bg-gray-500/20 text-gray-400 rounded hover:bg-gray-500/30 transition-colors text-sm"
                          >
                            Dismiss
                          </button>
                        </>
                      )}
                    </div>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      )}

      {/* Rules View */}
      {activeView === 'rules' && (
        <div className="space-y-4">
          {rules.length === 0 ? (
            <div className="border rounded-lg p-8 text-center text-[var(--color-text-muted)]">
              No alert rules configured. Create one to start monitoring.
            </div>
          ) : (
            <div className="space-y-2">
              {rules.map((rule) => (
                <div
                  key={rule.id}
                  className="border rounded-lg p-4 bg-[var(--color-panel)]"
                >
                  <div className="flex items-start justify-between gap-4">
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-2">
                        <span className={`px-2 py-0.5 rounded text-xs border ${getSeverityColor(rule.severity)}`}>
                          {rule.severity}
                        </span>
                        <span className={`px-2 py-0.5 rounded text-xs ${
                          rule.enabled ? 'bg-green-500/20 text-green-400' : 'bg-gray-500/20 text-gray-400'
                        }`}>
                          {rule.enabled ? 'Active' : 'Disabled'}
                        </span>
                        <h3 className="font-medium">{rule.name}</h3>
                      </div>
                      <div className="text-sm text-[var(--color-text-muted)] mb-2">
                        Type: <span className="text-[var(--color-text)]">{formatAlertType(rule.alert_type)}</span>
                      </div>
                      <div className="flex flex-wrap gap-2 text-xs">
                        <span className="text-[var(--color-text-muted)]">
                          Cooldown: <span className="text-[var(--color-text)]">{rule.cooldown_secs}s</span>
                        </span>
                        <span className="text-[var(--color-text-muted)]">
                          Actions: <span className="text-[var(--color-text)]">
                            {rule.actions.map(a => a.type).join(', ')}
                          </span>
                        </span>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <label className="relative inline-flex items-center cursor-pointer">
                        <input
                          type="checkbox"
                          checked={rule.enabled}
                          onChange={(e) => handleToggleRule(rule.id, e.target.checked)}
                          className="sr-only peer"
                        />
                        <div className="w-9 h-5 bg-[var(--color-muted)] rounded-full peer peer-checked:bg-[#00d4ff] after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:after:translate-x-full"></div>
                      </label>
                      <button
                        onClick={() => handleDeleteRule(rule.id)}
                        className="p-1.5 text-red-400 hover:bg-red-500/20 rounded transition-colors"
                        title="Delete rule"
                      >
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                          <path d="M3 6h18"></path>
                          <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"></path>
                          <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"></path>
                        </svg>
                      </button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Create Rule View */}
      {activeView === 'create' && (
        <div className="border rounded-lg p-6 bg-[var(--color-panel)] space-y-4">
          <h3 className="font-semibold">Create Alert Rule</h3>
          
          <div className="grid gap-4">
            <div>
              <label className="block text-sm font-medium mb-2">Name</label>
              <input
                type="text"
                value={newRule.name || ''}
                onChange={(e) => setNewRule({ ...newRule, name: e.target.value })}
                placeholder="High Error Rate Alert"
                className="w-full px-3 py-2 rounded border bg-[var(--color-bg)] border-[var(--color-border)]"
              />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium mb-2">Alert Type</label>
                <select
                  value={newRule.alert_type?.type || 'NoProgress'}
                  onChange={(e) => setNewRule({
                    ...newRule,
                    alert_type: { type: e.target.value as AlertRuleCreate['alert_type']['type'] }
                  })}
                  className="w-full px-3 py-2 rounded border bg-[var(--color-bg)] border-[var(--color-border)]"
                >
                  <option value="InfiniteLoop">Infinite Loop</option>
                  <option value="NoProgress">No Progress</option>
                  <option value="ResourceExhaustion">Resource Exhaustion</option>
                  <option value="TimeoutWarning">Timeout Warning</option>
                  <option value="PatternDetected">Pattern Detected</option>
                  <option value="ErrorSpike">Error Spike</option>
                  <option value="HighMemoryUsage">High Memory Usage</option>
                  <option value="LLMUnavailable">LLM Unavailable</option>
                  <option value="ExecutionPoolExhausted">Execution Pool Exhausted</option>
                  <option value="AwaitingConfirmation">Awaiting Confirmation</option>
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium mb-2">Severity</label>
                <select
                  value={newRule.severity || 'Warning'}
                  onChange={(e) => setNewRule({
                    ...newRule,
                    severity: e.target.value as AlertSeverity
                  })}
                  className="w-full px-3 py-2 rounded border bg-[var(--color-bg)] border-[var(--color-border)]"
                >
                  <option value="Info">Info</option>
                  <option value="Warning">Warning</option>
                  <option value="Critical">Critical</option>
                </select>
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">Cooldown (seconds)</label>
              <input
                type="number"
                value={newRule.cooldown_secs || 60}
                onChange={(e) => setNewRule({ ...newRule, cooldown_secs: parseInt(e.target.value) || 60 })}
                min={0}
                className="w-full px-3 py-2 rounded border bg-[var(--color-bg)] border-[var(--color-border)]"
              />
              <p className="text-xs text-[var(--color-text-muted)] mt-1">
                Minimum time between repeated alerts from this rule
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">Actions</label>
              <div className="space-y-2">
                {['Log', 'Notify', 'PauseTask', 'CancelTask'].map((actionType) => (
                  <label key={actionType} className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={newRule.actions?.some(a => a.type === actionType) || false}
                      onChange={(e) => {
                        const actions = newRule.actions || [];
                        if (e.target.checked) {
                          setNewRule({
                            ...newRule,
                            actions: [...actions.filter(a => a.type !== actionType), { type: actionType as AlertAction['type'] }]
                          });
                        } else {
                          setNewRule({
                            ...newRule,
                            actions: actions.filter(a => a.type !== actionType)
                          });
                        }
                      }}
                      className="rounded"
                    />
                    <span className="text-sm">{actionType}</span>
                  </label>
                ))}
                
                {/* Webhook action */}
                <div className="ml-6 mt-2 p-2 border rounded bg-[var(--color-muted)]/30">
                  <label className="flex items-center gap-2 cursor-pointer mb-2">
                    <input
                      type="checkbox"
                      checked={newRule.actions?.some(a => a.type === 'Webhook') || false}
                      onChange={(e) => {
                        const actions = (newRule.actions || []).filter(a => a.type !== 'Webhook');
                        if (e.target.checked) {
                          setNewRule({
                            ...newRule,
                            actions: [...actions, { type: 'Webhook' as const, url: '' }]
                          });
                        } else {
                          setNewRule({ ...newRule, actions });
                        }
                      }}
                      className="rounded"
                    />
                    <span className="text-sm font-medium">Webhook</span>
                  </label>
                  {newRule.actions?.some(a => a.type === 'Webhook') && (
                    <input
                      type="text"
                      placeholder="https://example.com/webhook"
                      value={newRule.actions?.find(a => a.type === 'Webhook')?.url || ''}
                      onChange={(e) => {
                        const actions = (newRule.actions || []).map(a => 
                          a.type === 'Webhook' ? { ...a, url: e.target.value } : a
                        );
                        setNewRule({ ...newRule, actions });
                      }}
                      className="w-full px-3 py-1.5 rounded border bg-[var(--color-bg)] border-[var(--color-border)] text-sm"
                    />
                  )}
                </div>
                
                {/* Email action */}
                <div className="ml-6 mt-2 p-2 border rounded bg-[var(--color-muted)]/30">
                  <label className="flex items-center gap-2 cursor-pointer mb-2">
                    <input
                      type="checkbox"
                      checked={newRule.actions?.some(a => a.type === 'Email') || false}
                      onChange={(e) => {
                        const actions = (newRule.actions || []).filter(a => a.type !== 'Email');
                        if (e.target.checked) {
                          setNewRule({
                            ...newRule,
                            actions: [...actions, { type: 'Email' as const, to: '', subject: '' }]
                          });
                        } else {
                          setNewRule({ ...newRule, actions });
                        }
                      }}
                      className="rounded"
                    />
                    <span className="text-sm font-medium">Email</span>
                  </label>
                  {newRule.actions?.some(a => a.type === 'Email') && (
                    <div className="space-y-2">
                      <input
                        type="email"
                        placeholder="alerts@example.com"
                        value={newRule.actions?.find(a => a.type === 'Email')?.to || ''}
                        onChange={(e) => {
                          const actions = (newRule.actions || []).map(a => 
                            a.type === 'Email' ? { ...a, to: e.target.value } : a
                          );
                          setNewRule({ ...newRule, actions });
                        }}
                        className="w-full px-3 py-1.5 rounded border bg-[var(--color-bg)] border-[var(--color-border)] text-sm"
                      />
                      <input
                        type="text"
                        placeholder="Subject (optional)"
                        value={newRule.actions?.find(a => a.type === 'Email')?.subject || ''}
                        onChange={(e) => {
                          const actions = (newRule.actions || []).map(a => 
                            a.type === 'Email' ? { ...a, subject: e.target.value } : a
                          );
                          setNewRule({ ...newRule, actions });
                        }}
                        className="w-full px-3 py-1.5 rounded border bg-[var(--color-bg)] border-[var(--color-border)] text-sm"
                      />
                    </div>
                  )}
                </div>
              </div>
            </div>
          </div>

          <div className="flex gap-3 pt-4">
            <button
              onClick={handleCreateRule}
              disabled={!newRule.name}
              className="px-4 py-2 bg-[#00d4ff] text-[#0f0f1a] rounded font-medium hover:opacity-90 disabled:opacity-50 transition-opacity"
            >
              Create Rule
            </button>
            <button
              onClick={() => setActiveView('rules')}
              className="px-4 py-2 bg-[var(--color-muted)] rounded hover:bg-[var(--color-muted)]/80 transition-colors"
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {/* v1.9.0: Analytics View */}
      {activeView === 'analytics' && (
        <div className="space-y-6">
          {/* Analytics Controls */}
          <div className="flex items-center gap-4">
            <select
              value={analyticsHours}
              onChange={(e) => setAnalyticsHours(parseInt(e.target.value))}
              className="px-3 py-2 rounded border bg-[var(--color-panel)] border-[var(--color-border)]"
            >
              <option value={6}>Last 6 hours</option>
              <option value={24}>Last 24 hours</option>
              <option value={48}>Last 48 hours</option>
              <option value={168}>Last 7 days</option>
            </select>
            <button
              onClick={loadAnalytics}
              className="px-3 py-2 bg-[var(--color-muted)] rounded hover:bg-[var(--color-muted)]/80 transition-colors text-sm"
            >
              Refresh
            </button>
          </div>

          {/* Analytics Summary */}
          {analytics && (
            <>
              <div className="grid grid-cols-4 gap-4">
                <div className="border rounded-lg p-4 bg-[var(--color-panel)]">
                  <div className="text-2xl font-bold">{analytics.total_alerts}</div>
                  <div className="text-sm text-[var(--color-text-muted)]">Total Alerts</div>
                </div>
                <div className="border rounded-lg p-4 bg-[var(--color-panel)]">
                  <div className="text-2xl font-bold">{Math.round(analytics.avg_ack_time_secs / 60)}m</div>
                  <div className="text-sm text-[var(--color-text-muted)]">Avg Ack Time</div>
                </div>
                <div className="border rounded-lg p-4 bg-[var(--color-panel)]">
                  <div className="text-2xl font-bold">{Math.round(analytics.avg_resolve_time_secs / 60)}m</div>
                  <div className="text-sm text-[var(--color-text-muted)]">Avg Resolve Time</div>
                </div>
                <div className="border rounded-lg p-4 bg-[var(--color-panel)]">
                  <div className="text-2xl font-bold">{analytics.top_rules?.length || 0}</div>
                  <div className="text-sm text-[var(--color-text-muted)]">Active Rules</div>
                </div>
              </div>

              {/* Hourly Chart */}
              <div className="border rounded-lg p-4 bg-[var(--color-panel)]">
                <h3 className="font-semibold mb-4">Alert Trend (Hourly)</h3>
                <div className="flex items-end gap-1 h-40">
                  {analytics.hourly_buckets?.map((bucket, idx) => {
                    const maxCount = Math.max(...(analytics.hourly_buckets?.map(b => b.count) || [1]));
                    const height = maxCount > 0 ? (bucket.count / maxCount) * 100 : 0;
                    return (
                      <div key={idx} className="flex-1 flex flex-col items-center group">
                        <div
                          className="w-full bg-[#00d4ff]/50 hover:bg-[#00d4ff] transition-colors rounded-t"
                          style={{ height: `${Math.max(height, 4)}%` }}
                          title={`${bucket.hour}: ${bucket.count} alerts`}
                        />
                        {bucket.critical > 0 && (
                          <div className="w-full bg-red-500/70 rounded-t" style={{ height: `${(bucket.critical / maxCount) * 100}%` }} />
                        )}
                      </div>
                    );
                  })}
                </div>
                <div className="flex justify-between mt-2 text-xs text-[var(--color-text-muted)]">
                  <span>Oldest</span>
                  <span>Most Recent</span>
                </div>
              </div>

              {/* Severity Breakdown */}
              <div className="grid grid-cols-3 gap-4">
                <div className="border rounded-lg p-4 bg-[var(--color-panel)]">
                  <h3 className="font-semibold mb-2">By Severity</h3>
                  <div className="space-y-2">
                    <div className="flex justify-between">
                      <span className="text-red-400">Critical</span>
                      <span>{analytics.by_severity?.Critical || 0}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-yellow-400">Warning</span>
                      <span>{analytics.by_severity?.Warning || 0}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-blue-400">Info</span>
                      <span>{analytics.by_severity?.Info || 0}</span>
                    </div>
                  </div>
                </div>
                <div className="border rounded-lg p-4 bg-[var(--color-panel)]">
                  <h3 className="font-semibold mb-2">By Status</h3>
                  <div className="space-y-2">
                    <div className="flex justify-between">
                      <span className="text-orange-400">Active</span>
                      <span>{analytics.by_status?.Active || 0}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-blue-400">Acknowledged</span>
                      <span>{analytics.by_status?.Acknowledged || 0}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-green-400">Resolved</span>
                      <span>{analytics.by_status?.Resolved || 0}</span>
                    </div>
                  </div>
                </div>
                <div className="border rounded-lg p-4 bg-[var(--color-panel)]">
                  <h3 className="font-semibold mb-2">Top Rules</h3>
                  <div className="space-y-2">
                    {analytics.top_rules?.slice(0, 3).map(([rule, count], idx) => (
                      <div key={idx} className="flex justify-between">
                        <span className="truncate mr-2">{rule}</span>
                        <span>{count}</span>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </>
          )}
        </div>
      )}

      {/* v1.9.0: Escalation View */}
      {activeView === 'escalation' && (
        <div className="space-y-6">
          {/* Escalation Config */}
          <div className="border rounded-lg p-4 bg-[var(--color-panel)]">
            <div className="flex items-center justify-between mb-4">
              <h3 className="font-semibold">Auto-Escalation Settings</h3>
              <label className="flex items-center gap-2">
                <span className="text-sm">Enable</span>
                <input
                  type="checkbox"
                  checked={escalationConfig.enabled}
                  onChange={(e) => setEscalationConfig({ ...escalationConfig, enabled: e.target.checked })}
                  className="rounded"
                />
              </label>
            </div>
            
            <div className="space-y-4">
              {escalationConfig.levels.map((level) => (
                <div key={level.level} className="border rounded p-4 bg-[var(--color-muted)]/30">
                  <div className="flex items-center gap-4 mb-2">
                    <span className="font-medium">Level {level.level}</span>
                    <span className="text-sm text-[var(--color-text-muted)]">
                      Wait: {level.wait_secs}s ({Math.round(level.wait_secs / 60)}m)
                    </span>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    {level.actions.map((action, idx) => (
                      <span key={idx} className="px-2 py-1 bg-[var(--color-panel)] rounded text-sm">
                        {action.type}
                        {action.type === 'Email' && action.to && `: ${action.to}`}
                        {action.type === 'Webhook' && action.url && `: ${action.url}`}
                        {action.type === 'ExecuteCommand' && action.command && `: ${action.command.substring(0, 20)}...`}
                      </span>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Pending Escalations */}
          <div className="border rounded-lg p-4 bg-[var(--color-panel)]">
            <div className="flex items-center justify-between mb-4">
              <h3 className="font-semibold">Pending Escalations</h3>
              <button
                onClick={handleProcessEscalations}
                disabled={pendingEscalation.length === 0}
                className="px-3 py-1.5 bg-orange-500/20 text-orange-400 rounded hover:bg-orange-500/30 disabled:opacity-50 transition-colors text-sm"
              >
                Process Now
              </button>
            </div>
            
            {pendingEscalation.length === 0 ? (
              <div className="text-center text-[var(--color-text-muted)] py-4">
                No alerts pending escalation
              </div>
            ) : (
              <div className="space-y-2">
                {pendingEscalation.map((alert) => (
                  <div key={alert.id} className="flex items-center justify-between p-3 border rounded bg-[var(--color-muted)]/20">
                    <div className="flex items-center gap-3">
                      <span className={`px-2 py-0.5 rounded text-xs ${getSeverityColor(alert.severity)}`}>
                        {alert.severity}
                      </span>
                      <span className="text-sm">{alert.message}</span>
                    </div>
                    <span className="text-sm text-[var(--color-text-muted)]">
                      Level {alert.escalation_level || 0}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      )}

      {/* v1.9.0: Pattern Suggestions View */}
      {activeView === 'patterns' && (
        <div className="space-y-4">
          <div className="flex items-center justify-between mb-4">
            <h3 className="font-semibold">Suggested Rules from Detected Patterns</h3>
            <button
              onClick={loadPatternSuggestions}
              className="px-3 py-1.5 bg-[var(--color-muted)] rounded hover:bg-[var(--color-muted)]/80 transition-colors text-sm"
            >
              Refresh
            </button>
          </div>
          
          {patternLoading ? (
            <div className="text-center py-8 text-[var(--color-text-muted)]">
              Loading pattern suggestions...
            </div>
          ) : patternSuggestions.length === 0 ? (
            <div className="border rounded-lg p-8 text-center text-[var(--color-text-muted)]">
              No pattern suggestions yet. Patterns are detected during task execution.
              Create rules manually or wait for more patterns to be detected.
            </div>
          ) : (
            <div className="space-y-3">
              {patternSuggestions.map((suggestion, idx) => (
                <div key={idx} className="border rounded-lg p-4 bg-[var(--color-panel)]">
                  <div className="flex items-start justify-between gap-4">
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-2">
                        <span className={`px-2 py-0.5 rounded text-xs border ${getSeverityColor(suggestion.suggested_severity)}`}>
                          {suggestion.suggested_severity}
                        </span>
                        <span className="px-2 py-0.5 rounded text-xs bg-green-500/20 text-green-400">
                          {suggestion.confidence}% confidence
                        </span>
                      </div>
                      <h4 className="font-medium mb-1">{suggestion.suggested_name}</h4>
                      <p className="text-sm text-[var(--color-text-muted)] mb-2">{suggestion.reason}</p>
                      <div className="flex flex-wrap gap-2 text-xs">
                        <span className="text-[var(--color-text-muted)]">Pattern: <span className="text-[var(--color-text)]">{suggestion.pattern_type}</span></span>
                        <span className="text-[var(--color-text-muted)]">Cooldown: <span className="text-[var(--color-text)]">{suggestion.cooldown_secs}s</span></span>
                        <span className="text-[var(--color-text-muted)]">Actions: <span className="text-[var(--color-text)]">{suggestion.suggested_actions.map(a => a.type).join(', ')}</span></span>
                      </div>
                    </div>
                    <button
                      onClick={() => handleCreateFromSuggestion(suggestion)}
                      className="px-3 py-1.5 bg-[#00d4ff]/20 text-[#00d4ff] rounded hover:bg-[#00d4ff]/30 transition-colors text-sm whitespace-nowrap"
                    >
                      Create Rule
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
