import { useState, useEffect } from 'react';
import { apiGet, apiPost } from '../../lib/api';

export function SecretsManager() {
  const [variables, setVariables] = useState('');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      const res = await apiGet('/api/v1/settings/secrets_variables');
      if (res.ok) {
        const data = await res.json();
        if (data.value) {
          setVariables(data.value);
        }
      }
    } catch (err) {
      console.error('Failed to load secrets:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      await apiPost('/api/v1/settings/secrets_variables', {
        value: variables,
        encrypt: false,
      });
    } catch (err) {
      console.error('Failed to save secrets:', err);
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return <div className="p-4">Loading secrets...</div>;
  }

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold">Secrets Management</h3>
        <p className="text-sm text-muted-foreground">
          Manage secrets and credentials. Variables are visible to LLMs and chat history.
          Use the Preferences tab for encrypted storage.
        </p>
      </div>

      <div className="space-y-4 max-w-2xl">
        <div className="field field-full">
          <div className="field-label">
            <div className="field-title">Variables Store</div>
            <div className="field-description">
              Store non-sensitive variables in .env format, one KEY=VALUE per line.
              Use comments starting with # to add descriptions.
            </div>
          </div>
          <div className="field-control">
            <textarea
              value={variables}
              onChange={(e) => setVariables(e.target.value)}
              placeholder="# Email configuration&#10;EMAIL_IMAP_SERVER=&quot;imap.gmail.com&quot;&#10;EMAIL_SMTP_SERVER=&quot;smtp.gmail.com&quot;&#10;&#10;# Database&#10;DB_HOST=&quot;localhost&quot;"
              className="w-full px-3 py-2 bg-background border rounded-md font-mono text-sm"
              rows={12}
            />
          </div>
        </div>

        <div className="flex gap-3 pt-4">
          <button
            onClick={handleSave}
            disabled={saving}
            className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:opacity-90 disabled:opacity-50"
          >
            {saving ? 'Saving...' : 'Save Variables'}
          </button>
        </div>

        <div className="bg-muted/50 p-4 rounded-lg">
          <h4 className="font-medium mb-2">Note</h4>
          <p className="text-sm text-muted-foreground">
            For sensitive data like API keys and passwords, use the Preferences tab which provides
            encrypted storage. Variables stored here are visible in plain text.
          </p>
        </div>
      </div>
    </div>
  );
}
