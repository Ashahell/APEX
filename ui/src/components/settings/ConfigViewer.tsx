import { useState, useEffect } from 'react';
import { apiGet } from '../../lib/api';

interface ConfigSummary {
  server: {
    host: string;
    port: number;
  };
  auth_enabled: boolean;
  database_type: string;
  nats_enabled: boolean;
  use_llm: boolean;
  execution_backend: string;
  heartbeat_enabled: boolean;
  config_source: string;
  validation_errors: Array<{
    field: string;
    message: string;
  }>;
}

interface ConfigDetail {
  config: Record<string, string>;
  source: string;
}

export function ConfigViewer() {
  const [summary, setSummary] = useState<ConfigSummary | null>(null);
  const [detail, setDetail] = useState<ConfigDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [showDetail, setShowDetail] = useState(false);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    setLoading(true);
    try {
      const summaryRes = await apiGet('/api/v1/config/summary');
      if (summaryRes.ok) {
        const summaryData = await summaryRes.json();
        setSummary(summaryData);
      }
      
      const detailRes = await apiGet('/api/v1/config');
      if (detailRes.ok) {
        const detailData = await detailRes.json();
        setDetail(detailData);
      }
    } catch (err) {
      console.error('Failed to load config:', err);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-[var(--color-text-muted)]">Loading configuration...</div>
      </div>
    );
  }

  const categories = [
    {
      name: 'Server',
      items: [
        { label: 'Host', value: summary?.server.host || 'N/A' },
        { label: 'Port', value: summary?.server.port?.toString() || 'N/A' },
      ],
    },
    {
      name: 'Authentication',
      items: [
        { label: 'Enabled', value: summary?.auth_enabled ? 'Yes' : 'No' },
        { label: 'Source', value: summary?.config_source || 'N/A' },
      ],
    },
    {
      name: 'Execution',
      items: [
        { label: 'Backend', value: summary?.execution_backend || 'N/A' },
        { label: 'LLM Enabled', value: summary?.use_llm ? 'Yes' : 'No' },
      ],
    },
    {
      name: 'Database',
      items: [
        { label: 'Type', value: summary?.database_type || 'N/A' },
      ],
    },
    {
      name: 'Messaging',
      items: [
        { label: 'NATS Enabled', value: summary?.nats_enabled ? 'Yes' : 'No' },
      ],
    },
    {
      name: 'Autonomy',
      items: [
        { label: 'Heartbeat Enabled', value: summary?.heartbeat_enabled ? 'Yes' : 'No' },
      ],
    },
  ];

  return (
    <div className="h-full overflow-auto p-4">
      <div className="max-w-4xl mx-auto space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold" style={{ color: '#4248f1' }}>Configuration</h2>
            <p className="text-sm text-[var(--color-text-muted)]">
              Current runtime configuration
            </p>
          </div>
          <button
            onClick={() => setShowDetail(!showDetail)}
            className="px-4 py-2 rounded-xl border border-border hover:bg-[#4248f1]/10 transition-colors flex items-center gap-2"
          >
            {showDetail ? (
              <>
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                </svg>
                Hide Details
              </>
            ) : (
              <>
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
                Show All Vars
              </>
            )}
          </button>
        </div>

        {summary?.validation_errors && summary.validation_errors.length > 0 && (
          <div className="border border-[#4248f1]/30 rounded-xl p-4 bg-[#4248f1]/10">
            <h3 className="font-semibold mb-2" style={{ color: '#4248f1' }}>Validation Warnings</h3>
            <div className="space-y-1">
              {summary.validation_errors.map((err, i) => (
                <div key={i} className="text-sm" style={{ color: '#818cf8' }}>
                  <span className="font-mono">{err.field}</span>: {err.message}
                </div>
              ))}
            </div>
          </div>
        )}

        <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
          {categories.map((cat) => (
            <div key={cat.name} className="border border-border rounded-xl p-4 bg-[var(--color-panel)] hover:border-[#4248f1]/30 transition-colors">
              <h3 className="font-semibold mb-2" style={{ color: '#4248f1' }}>{cat.name}</h3>
              <div className="space-y-1 text-sm">
                {cat.items.map((item) => (
                  <div key={item.label} className="flex justify-between">
                    <span className="text-[var(--color-text-muted)]">{item.label}:</span>
                    <span className="font-medium">{item.value}</span>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>

        {showDetail && detail && (
          <div className="border border-border rounded-xl p-4 bg-[var(--color-panel)]">
            <h3 className="font-semibold mb-3" style={{ color: '#4248f1' }}>All Configuration Variables</h3>
            <div className="bg-[var(--color-background)] rounded-xl p-3 max-h-96 overflow-auto border border-border">
              <table className="w-full text-sm">
                <thead>
                  <tr className="text-left">
                    <th className="pb-2 font-medium">Variable</th>
                    <th className="pb-2 font-medium">Value</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border">
                  {Object.entries(detail.config).map(([key, value]) => (
                    <tr key={key}>
                      <td className="py-1 font-mono" style={{ color: '#4248f1' }}>{key}</td>
                      <td className="py-1 font-mono text-[var(--color-text-muted)]">
                        {value}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}

        <div className="border border-border rounded-xl p-4 bg-[var(--color-panel)]">
          <h3 className="font-semibold mb-2" style={{ color: '#4248f1' }}>Configuration Sources</h3>
          <div className="text-sm text-[var(--color-text-muted)] space-y-1">
            <p>Configuration is loaded in the following priority order:</p>
            <ol className="list-decimal list-inside space-y-1 mt-2">
              <li>Environment variables (highest priority)</li>
              <li>apex.yaml / apex.yml file</li>
              <li>Default values (lowest priority)</li>
            </ol>
          </div>
        </div>
      </div>
    </div>
  );
}
