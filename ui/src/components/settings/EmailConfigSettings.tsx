import { useState, useEffect } from 'react';
import { apiGet, apiPost, apiDelete } from '../../lib/api';

interface EmailConfigState {
  smtp_host: string;
  smtp_port: number;
  username: string;
  password: string;
  from_address: string;
  use_tls: boolean;
}

export function EmailConfigSettings() {
  const [config, setConfig] = useState<EmailConfigState>({
    smtp_host: 'smtp.gmail.com',
    smtp_port: 587,
    username: '',
    password: '',
    from_address: '',
    use_tls: true,
  });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [testing, setTesting] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);
  const [testEmail, setTestEmail] = useState('');
  const [showPassword, setShowPassword] = useState(false);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    setLoading(true);
    try {
      const res = await apiGet('/api/v1/vigilant/email/config');
      if (res.ok) {
        const data = await res.json();
        if (data.email && data.email.configured) {
          setConfig({
            smtp_host: data.email.smtp_host || 'smtp.gmail.com',
            smtp_port: data.email.smtp_port || 587,
            username: data.email.username || '',
            password: '', // Don't show existing password
            from_address: data.email.from_address || '',
            use_tls: data.email.use_tls ?? true,
          });
        }
      }
    } catch (e) {
      console.error('Failed to load email config:', e);
    } finally {
      setLoading(false);
    }
  };

  const saveConfig = async () => {
    setSaving(true);
    setMessage(null);
    try {
      const res = await apiPost('/api/v1/vigilant/email/config', {
        smtp_host: config.smtp_host,
        smtp_port: config.smtp_port,
        username: config.username,
        password: config.password || undefined, // Only send if changed
        from_address: config.from_address,
        use_tls: config.use_tls,
      });
      if (res.ok) {
        setMessage({ type: 'success', text: 'Email configuration saved successfully!' });
        setConfig(prev => ({ ...prev, password: '' })); // Clear password after save
      } else {
        const data = await res.json();
        setMessage({ type: 'error', text: data.message || 'Failed to save configuration' });
      }
    } catch (e) {
      setMessage({ type: 'error', text: 'Failed to save configuration' });
    } finally {
      setSaving(false);
    }
  };

  const testConnection = async () => {
    if (!testEmail) {
      setMessage({ type: 'error', text: 'Please enter a test email address' });
      return;
    }
    setTesting(true);
    setMessage(null);
    try {
      // First save config
      await saveConfig();
      // Then test
      const res = await apiPost('/api/v1/vigilant/email/test', {
        test_email: testEmail,
      });
      const data = await res.json();
      if (res.ok) {
        setMessage({ type: 'success', text: data.message || 'Test email queued!' });
      } else {
        setMessage({ type: 'error', text: data[1] || 'Failed to send test email' });
      }
    } catch (e) {
      setMessage({ type: 'error', text: 'Failed to send test email' });
    } finally {
      setTesting(false);
    }
  };

  const deleteConfig = async () => {
    if (!confirm('Are you sure you want to delete the email configuration?')) return;
    try {
      const res = await apiDelete('/api/v1/vigilant/email/config');
      if (res.ok) {
        setConfig({
          smtp_host: 'smtp.gmail.com',
          smtp_port: 587,
          username: '',
          password: '',
          from_address: '',
          use_tls: true,
        });
        setMessage({ type: 'success', text: 'Email configuration deleted' });
      }
    } catch (e) {
      setMessage({ type: 'error', text: 'Failed to delete configuration' });
    }
  };

  if (loading) {
    return <div className="text-[var(--color-text-muted)]">Loading email configuration...</div>;
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-semibold">Email Configuration</h2>
        <p className="text-[var(--color-text-muted)] mt-1">
          Configure SMTP settings for alert email notifications
        </p>
      </div>

      {message && (
        <div className={`p-3 rounded ${message.type === 'success' ? 'bg-green-500/20 text-green-400 border border-green-500/30' : 'bg-red-500/20 text-red-400 border border-red-500/30'}`}>
          {message.text}
        </div>
      )}

      <div className="border rounded-lg p-6 bg-[var(--color-panel)] space-y-4">
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-sm font-medium mb-2">SMTP Host</label>
            <input
              type="text"
              value={config.smtp_host}
              onChange={(e) => setConfig({ ...config, smtp_host: e.target.value })}
              placeholder="smtp.gmail.com"
              className="w-full px-3 py-2 rounded border bg-[var(--color-bg)] border-[var(--color-border)]"
            />
          </div>
          <div>
            <label className="block text-sm font-medium mb-2">SMTP Port</label>
            <input
              type="number"
              value={config.smtp_port}
              onChange={(e) => setConfig({ ...config, smtp_port: parseInt(e.target.value) || 587 })}
              placeholder="587"
              className="w-full px-3 py-2 rounded border bg-[var(--color-bg)] border-[var(--color-border)]"
            />
          </div>
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-sm font-medium mb-2">Username / Email</label>
            <input
              type="text"
              value={config.username}
              onChange={(e) => setConfig({ ...config, username: e.target.value })}
              placeholder="your-email@gmail.com"
              className="w-full px-3 py-2 rounded border bg-[var(--color-bg)] border-[var(--color-border)]"
            />
          </div>
          <div>
            <label className="block text-sm font-medium mb-2">From Address</label>
            <input
              type="email"
              value={config.from_address}
              onChange={(e) => setConfig({ ...config, from_address: e.target.value })}
              placeholder="alerts@yourdomain.com"
              className="w-full px-3 py-2 rounded border bg-[var(--color-bg)] border-[var(--color-border)]"
            />
          </div>
        </div>

        <div>
          <label className="block text-sm font-medium mb-2">Password / App Password</label>
          <div className="relative">
            <input
              type={showPassword ? 'text' : 'password'}
              value={config.password}
              onChange={(e) => setConfig({ ...config, password: e.target.value })}
              placeholder="•••••••• (leave empty to keep existing)"
              className="w-full px-3 py-2 pr-10 rounded border bg-[var(--color-bg)] border-[var(--color-border)]"
            />
            <button
              type="button"
              onClick={() => setShowPassword(!showPassword)}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
            >
              {showPassword ? (
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path>
                  <line x1="1" y1="1" x2="23" y2="23"></line>
                </svg>
              ) : (
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path>
                  <circle cx="12" cy="12" r="3"></circle>
                </svg>
              )}
            </button>
          </div>
          <p className="text-xs text-[var(--color-text-muted)] mt-1">
            For Gmail, use an App Password instead of your regular password
          </p>
        </div>

        <div className="flex items-center gap-3">
          <label className="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={config.use_tls}
              onChange={(e) => setConfig({ ...config, use_tls: e.target.checked })}
              className="sr-only peer"
            />
            <div className="w-9 h-5 bg-[var(--color-muted)] rounded-full peer peer-checked:bg-[#00d4ff] after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:after:translate-x-full"></div>
          </label>
          <div>
            <span className="font-medium">Use TLS</span>
            <p className="text-xs text-[var(--color-text-muted)]">Enable TLS encryption (recommended)</p>
          </div>
        </div>

        <div className="border-t pt-4 flex flex-wrap gap-3">
          <button
            onClick={saveConfig}
            disabled={saving}
            className="px-4 py-2 bg-[#00d4ff] text-[#0f0f1a] rounded font-medium hover:opacity-90 disabled:opacity-50 transition-opacity"
          >
            {saving ? 'Saving...' : 'Save Configuration'}
          </button>
          <button
            onClick={deleteConfig}
            className="px-4 py-2 bg-red-500/20 text-red-400 rounded font-medium hover:bg-red-500/30 transition-colors"
          >
            Delete Configuration
          </button>
        </div>
      </div>

      {/* Test Email Section */}
      <div className="border rounded-lg p-6 bg-[var(--color-panel)] space-y-4">
        <h3 className="font-semibold flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"></path>
            <polyline points="22,6 12,13 2,6"></polyline>
          </svg>
          Test Email
        </h3>
        <p className="text-sm text-[var(--color-text-muted)]">
          Send a test email to verify your configuration works correctly.
        </p>
        <div className="flex gap-3">
          <input
            type="email"
            value={testEmail}
            onChange={(e) => setTestEmail(e.target.value)}
            placeholder="test@example.com"
            className="flex-1 px-3 py-2 rounded border bg-[var(--color-bg)] border-[var(--color-border)]"
          />
          <button
            onClick={testConnection}
            disabled={testing}
            className="px-4 py-2 bg-green-500/20 text-green-400 rounded font-medium hover:bg-green-500/30 transition-colors disabled:opacity-50"
          >
            {testing ? 'Sending...' : 'Send Test Email'}
          </button>
        </div>
      </div>

      {/* Help Section */}
      <div className="border rounded-lg p-4 bg-[var(--color-panel)]">
        <h4 className="font-medium mb-2 flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="12" cy="12" r="10"></circle>
            <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path>
            <line x1="12" y1="17" x2="12.01" y2="17"></line>
          </svg>
          Setup Tips
        </h4>
        <ul className="text-sm text-[var(--color-text-muted)] space-y-2 ml-6 list-disc">
          <li><strong>Gmail:</strong> Enable 2FA, then create an App Password at myaccount.google.com/security</li>
          <li><strong>Outlook/Hotmail:</strong> Use your regular password with SMTP (port 587)</li>
          <li><strong>Custom SMTP:</strong> Check with your email provider for correct settings</li>
          <li><strong>Security:</strong> Passwords are stored securely and never displayed after saving</li>
        </ul>
      </div>
    </div>
  );
}
