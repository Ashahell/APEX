import { useState, useEffect } from 'react';
import { apiGet } from '../../lib/api';

interface SandboxConfig {
  memory_limit_mb: number;
  timeout_secs: number;
}

interface RuntimeConfig {
  sandbox: SandboxConfig;
  config_source: string;
}

export function RuntimeSettings() {
  const [config, setConfig] = useState<RuntimeConfig | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    setLoading(true);
    try {
      const res = await apiGet('/api/v1/config/summary');
      if (res.ok) {
        const data = await res.json();
        setConfig({
          sandbox: {
            memory_limit_mb: 512, // From environment default
            timeout_secs: 30,     // From environment default
          },
          config_source: data.config_source || 'default',
        });
      }
    } catch (err) {
      console.error('Failed to load runtime config:', err);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-[var(--color-text-muted)]">Loading runtime settings...</div>
      </div>
    );
  }

  return (
    <>
      <div className="mb-6">
        <h2 className="text-2xl font-semibold">Runtime Settings</h2>
        <p className="text-muted-foreground">
          Configure dynamic tool execution and sandbox behavior
        </p>
      </div>

      <div className="space-y-6">
        {/* Sandbox Configuration */}
        <section className="border rounded-lg p-4">
          <h3 className="font-semibold mb-4">Python Sandbox</h3>
          <p className="text-sm text-muted-foreground mb-4">
            Configure the secure sandbox for executing dynamically generated Python tools.
          </p>

          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium mb-2">
                Memory Limit (MB)
              </label>
              <div className="flex items-center gap-4">
                <input
                  type="number"
                  value={config?.sandbox.memory_limit_mb || 512}
                  className="w-32 px-3 py-2 rounded border bg-[var(--color-panel)]"
                  disabled
                />
                <span className="text-sm text-muted-foreground">
                  Max memory per tool execution (Unix only)
                </span>
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">
                Execution Timeout (seconds)
              </label>
              <div className="flex items-center gap-4">
                <input
                  type="number"
                  value={config?.sandbox.timeout_secs || 30}
                  className="w-32 px-3 py-2 rounded border bg-[var(--color-panel)]"
                  disabled
                />
                <span className="text-sm text-muted-foreground">
                  Max execution time per tool
                </span>
              </div>
            </div>
          </div>

          <div className="mt-4 p-3 bg-[var(--color-panel)] rounded text-sm">
            <p className="font-medium mb-2">Environment Variables:</p>
            <code className="block text-xs bg-[var(--color-muted)] p-2 rounded">
              APEX_SANDBOX_MEMORY_MB={config?.sandbox.memory_limit_mb || 512}
            </code>
            <code className="block text-xs bg-[var(--color-muted)] p-2 rounded mt-1">
              APEX_SANDBOX_TIMEOUT_SECS={config?.sandbox.timeout_secs || 30}
            </code>
            <p className="mt-2 text-muted-foreground">
              Source: {config?.config_source || 'default'}
            </p>
          </div>
        </section>

        {/* Platform Notes */}
        <section className="border rounded-lg p-4">
          <h3 className="font-semibold mb-4">Platform Notes</h3>
          <div className="space-y-2 text-sm text-muted-foreground">
            <p>
              <strong>Linux/macOS:</strong> Memory limits are enforced using{' '}
              <code className="bg-[var(--color-muted)] px-1 rounded">resource.setrlimit()</code>.
              The process will receive MemoryError or segfault if limit is exceeded.
            </p>
            <p>
              <strong>Windows:</strong> Memory limits are not supported natively. Timeout enforcement
              remains active.
            </p>
          </div>
        </section>

        {/* Storage Info */}
        <section className="border rounded-lg p-4">
          <h3 className="font-semibold mb-4">Configuration Storage</h3>
          <div className="space-y-2 text-sm text-muted-foreground">
            <p>
              Settings are loaded from these sources (in order of precedence):
            </p>
            <ol className="list-decimal list-inside space-y-1 ml-4">
              <li>
                <strong>Environment variables</strong> - Set before starting APEX
                <code className="block text-xs bg-[var(--color-muted)] p-2 rounded mt-1">
                  APEX_SANDBOX_MEMORY_MB=512
                </code>
              </li>
              <li>
                <strong>YAML config file</strong> - <code className="bg-[var(--color-muted)] px-1 rounded">apex.yaml</code>
                <pre className="bg-[var(--color-muted)] p-2 rounded mt-1 text-xs overflow-x-auto">
{`execution:
  sandbox:
    memory_limit_mb: 512
    timeout_secs: 30`}
                </pre>
              </li>
              <li>
                <strong>Default values</strong> - 512MB memory, 30s timeout
              </li>
            </ol>
          </div>
        </section>
      </div>
    </>
  );
}
