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
        <div className="text-muted-foreground">Loading configuration...</div>
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
            <h2 className="text-2xl font-bold">Configuration</h2>
            <p className="text-sm text-muted-foreground">
              Current runtime configuration
            </p>
          </div>
          <button
            onClick={() => setShowDetail(!showDetail)}
            className="px-4 py-2 rounded-lg border hover:bg-muted"
          >
            {showDetail ? 'Hide Details' : 'Show All Vars'}
          </button>
        </div>

        {summary?.validation_errors && summary.validation_errors.length > 0 && (
          <div className="border border-yellow-500/30 rounded-lg p-4 bg-yellow-500/10">
            <h3 className="font-semibold text-yellow-500 mb-2">Validation Warnings</h3>
            <div className="space-y-1">
              {summary.validation_errors.map((err, i) => (
                <div key={i} className="text-sm text-yellow-400">
                  <span className="font-mono">{err.field}</span>: {err.message}
                </div>
              ))}
            </div>
          </div>
        )}

        <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
          {categories.map((cat) => (
            <div key={cat.name} className="border rounded-lg p-4">
              <h3 className="font-semibold mb-2">{cat.name}</h3>
              <div className="space-y-1 text-sm">
                {cat.items.map((item) => (
                  <div key={item.label} className="flex justify-between">
                    <span className="text-muted-foreground">{item.label}:</span>
                    <span className="font-medium">{item.value}</span>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>

        {showDetail && detail && (
          <div className="border rounded-lg p-4">
            <h3 className="font-semibold mb-3">All Configuration Variables</h3>
            <div className="bg-muted rounded-lg p-3 max-h-96 overflow-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="text-left">
                    <th className="pb-2 font-medium">Variable</th>
                    <th className="pb-2 font-medium">Value</th>
                  </tr>
                </thead>
                <tbody className="divide-y">
                  {Object.entries(detail.config).map(([key, value]) => (
                    <tr key={key}>
                      <td className="py-1 font-mono text-blue-500">{key}</td>
                      <td className="py-1 font-mono text-muted-foreground">
                        {value}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}

        <div className="border rounded-lg p-4">
          <h3 className="font-semibold mb-2">Configuration Sources</h3>
          <div className="text-sm text-muted-foreground space-y-1">
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
