import { useEffect, useState, useCallback } from 'react';
import { apiGet, apiPost } from '../../lib/api';

interface RegistryInfo {
  id: string;
  name: string;
  description?: string;
  tool_count?: number;
  created_at?: string;
}

interface Tool {
  name: string;
  description?: string;
  input_schema: any;
}

interface ValidationResult {
  valid: boolean;
  error?: string;
}

interface ServerTemplate {
  name: string;
  command: string;
  args: string[];
  description: string;
  icon?: string;
}

// Predefined MCP server templates from popular registries
const SERVER_TEMPLATES: ServerTemplate[] = [
  {
    name: 'filesystem',
    command: 'npx',
    args: ['-y', '@modelcontextprotocol/server-filesystem', '/path/to/files'],
    description: 'File system access - read and write files',
    icon: '📁',
  },
  {
    name: 'github',
    command: 'npx',
    args: ['-y', '@modelcontextprotocol/server-github'],
    description: 'GitHub integration - repos, issues, PRs',
    icon: '💻',
  },
  {
    name: 'slack',
    command: 'npx',
    args: ['-y', '@modelcontextprotocol/server-slack'],
    description: 'Slack integration - channels, messages',
    icon: '💬',
  },
  {
    name: 'postgres',
    command: 'npx',
    args: ['-y', '@modelcontextprotocol/server-postgres'],
    description: 'PostgreSQL database access',
    icon: '🐘',
  },
  {
    name: 'brave-search',
    command: 'npx',
    args: ['-y', '@modelcontextprotocol/server-brave-search'],
    description: 'Brave search API integration',
    icon: '🔍',
  },
  {
    name: 'fetch',
    command: 'npx',
    args: ['-y', '@modelcontextprotocol/server-fetch'],
    description: 'HTTP fetch / web scraping',
    icon: '🌐',
  },
];

export function McpMarketplace() {
  const [newRegistryName, setNewRegistryName] = useState('');
  const [validationResult, setValidationResult] = useState<ValidationResult | null>(null);
  const [validating, setValidating] = useState(false);
  const [registries, setRegistries] = useState<RegistryInfo[]>([]);
  const [selectedRid, setSelectedRid] = useState<string | null>(null);
  const [tools, setTools] = useState<Tool[]>([]);
  const [loading, setLoading] = useState(true);
  const [creating, setCreating] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [showInstallModal, setShowInstallModal] = useState(false);
  const [selectedTemplate, setSelectedTemplate] = useState<ServerTemplate | null>(null);
  const [installing, setInstalling] = useState(false);
  
  // Filter templates based on search
  const filteredTemplates = SERVER_TEMPLATES.filter(t => 
    t.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    t.description.toLowerCase().includes(searchQuery.toLowerCase())
  );

  useEffect(() => {
    fetchRegistries();
  }, []);

  const fetchRegistries = async () => {
    setLoading(true);
    try {
      const res = await apiGet('/api/v1/mcp/registries');
      const data = await res.json();
      setRegistries(data);
      if (data.length > 0 && !selectedRid) {
        setSelectedRid(data[0].id);
      }
    } catch (e) {
      console.error('Failed to load MCP registries', e);
    } finally {
      setLoading(false);
    }
  };

  const validateRegistryName = useCallback(async (name: string) => {
    if (!name.trim()) {
      setValidationResult(null);
      return;
    }
    
    setValidating(true);
    try {
      const res = await apiPost('/api/v1/mcp/registries/validate', { name: name.trim() });
      const data = await res.json();
      setValidationResult(data);
    } catch (e) {
      console.error('Validation error', e);
      setValidationResult({ valid: false, error: 'Validation failed' });
    } finally {
      setValidating(false);
    }
  }, []);

  useEffect(() => {
    const timer = setTimeout(() => {
      validateRegistryName(newRegistryName);
    }, 300);
    return () => clearTimeout(timer);
  }, [newRegistryName, validateRegistryName]);

  useEffect(() => {
    if (selectedRid) {
      loadTools(selectedRid);
    }
  }, [selectedRid]);

  const createRegistry = async () => {
    if (!newRegistryName.trim()) return;
    if (validationResult && !validationResult.valid) return;
    
    setCreating(true);
    try {
      const res = await apiPost('/api/v1/mcp/registries', { name: newRegistryName.trim() });
      if (res.ok) {
        await fetchRegistries();
        setNewRegistryName('');
        setValidationResult(null);
      } else {
        const error = await res.json();
        console.error('Failed to create registry', error);
      }
    } catch (e) {
      console.error('Failed to create registry', e);
    } finally {
      setCreating(false);
    }
  };

  const loadTools = async (rid: string) => {
    try {
      const res = await apiGet(`/api/v1/mcp/registries/${rid}/tools`);
      const data = await res.json();
      setTools(data);
    } catch (e) {
      console.error('Failed to load tools for registry', e);
    }
  };

  const handleInstallTemplate = async (template: ServerTemplate) => {
    setSelectedTemplate(template);
    setShowInstallModal(true);
  };

  const confirmInstall = async () => {
    if (!selectedTemplate) return;
    
    setInstalling(true);
    try {
      const res = await apiPost('/api/v1/mcp/servers', {
        name: selectedTemplate.name,
        command: selectedTemplate.command,
        args: selectedTemplate.args,
        enabled: true,
      });
      if (res.ok) {
        setShowInstallModal(false);
        setSelectedTemplate(null);
      } else {
        const error = await res.json();
        console.error('Failed to install MCP server', error);
      }
    } catch (e) {
      console.error('Failed to install MCP server', e);
    } finally {
      setInstalling(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && newRegistryName.trim()) {
      createRegistry();
    }
  };

  return (
    <div className="flex h-full flex-col gap-4 p-4">
      {/* Header */}
      <div className="flex items-center justify-between pb-2 border-b">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-purple-500/20 rounded-lg">
            <svg className="w-6 h-6 text-purple-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
            </svg>
          </div>
          <div>
            <h2 className="text-xl font-semibold">MCP Marketplace</h2>
            <p className="text-muted-foreground text-sm">Discover and install MCP server templates</p>
          </div>
        </div>
      </div>

      <div className="flex gap-4 flex-1 overflow-hidden">
        {/* Left: Server Templates */}
        <div className="w-1/2 flex flex-col gap-4">
          {/* Search */}
          <div className="relative">
            <input
              placeholder="Search MCP servers..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full px-4 py-2 pl-10 border rounded-lg bg-background"
            />
            <svg className="absolute left-3 top-2.5 w-4 h-4 text-muted-foreground" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </div>

          {/* Templates Grid */}
          <div className="flex-1 overflow-y-auto grid grid-cols-2 gap-3 pr-2">
            {filteredTemplates.map((template) => (
              <div
                key={template.name}
                className="border rounded-lg p-3 bg-card hover:bg-muted/50 cursor-pointer transition-colors group"
                onClick={() => handleInstallTemplate(template)}
              >
                <div className="flex items-start gap-2">
                  <span className="text-xl">{template.icon}</span>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center justify-between">
                      <h4 className="font-medium text-sm truncate">{template.name}</h4>
                      <button className="opacity-0 group-hover:opacity-100 px-2 py-0.5 text-xs bg-purple-600 text-white rounded hover:bg-purple-700 transition-opacity">
                        Install
                      </button>
                    </div>
                    <p className="text-xs text-muted-foreground mt-1 line-clamp-2">{template.description}</p>
                    <code className="text-xs text-muted-foreground font-mono mt-1 block truncate">
                      {template.command} {template.args.slice(0, 2).join(' ')}...
                    </code>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Right: Registries & Tools */}
        <div className="w-1/2 flex flex-col gap-4">
          {/* Registry creation */}
          <div className="border rounded-lg p-3 bg-card">
            <h3 className="text-sm font-semibold mb-2">Create Custom Registry</h3>
            <div className="flex gap-2">
              <input
                placeholder="New registry name"
                value={newRegistryName}
                onChange={(e) => setNewRegistryName(e.target.value)}
                onKeyDown={handleKeyDown}
                className="flex-1 px-3 py-2 border rounded bg-background"
              />
              <button
                onClick={createRegistry}
                disabled={!newRegistryName.trim() || (validationResult && !validationResult.valid) || creating || validating}
                className="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
              >
                {creating ? 'Creating...' : 'Create'}
              </button>
            </div>
          </div>

          {/* Registries List */}
          <div className="border rounded-lg p-3 bg-card flex-1 flex flex-col overflow-hidden">
            <h3 className="text-sm font-semibold mb-2">Registries</h3>
            {loading ? (
              <div className="text-muted-foreground text-sm">Loading...</div>
            ) : (
              <div className="flex-1 overflow-y-auto space-y-2">
                {registries.map(r => (
                  <div
                    key={r.id}
                    className={`p-2 rounded cursor-pointer flex items-center justify-between group ${
                      selectedRid === r.id ? 'bg-muted' : 'hover:bg-muted'
                    }`}
                    onClick={() => setSelectedRid(r.id)}
                  >
                    <div>
                      <div className="text-sm">{r.name}</div>
                      <div className="text-xs text-muted-foreground truncate max-w-xs">{r.id}</div>
                    </div>
                    <button
                      onClick={(e) => { e.stopPropagation(); /* delete logic could go here */ }}
                      className="opacity-0 group-hover:opacity-100 px-2 py-1 text-xs text-red-500 hover:bg-red-50 rounded"
                    >
                      Delete
                    </button>
                  </div>
                ))}
                {registries.length === 0 && (
                  <div className="text-sm text-muted-foreground text-center py-4">
                    No registries created yet
                  </div>
                )}
              </div>
            )}
          </div>

          {/* Tools List */}
          <div className="border rounded-lg p-3 bg-card flex-1 flex flex-col overflow-hidden">
            <div className="flex items-center justify-between mb-2">
              <h3 className="text-sm font-semibold">Tools in Registry</h3>
            </div>
            {selectedRid ? (
              <div className="flex-1 overflow-y-auto space-y-2">
                {tools.length === 0 ? (
                  <div className="text-sm text-muted-foreground text-center py-4">
                    No tools in this registry
                  </div>
                ) : (
                  tools.map(t => (
                    <div key={t.name} className="border rounded-lg p-2 flex flex-col">
                      <strong className="text-sm">{t.name}</strong>
                      {t.description && (
                        <span className="text-xs text-muted-foreground">{t.description}</span>
                      )}
                    </div>
                  ))
                )}
              </div>
            ) : (
              <div className="text-sm text-muted-foreground text-center py-4">
                Select a registry to view tools
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Install Modal */}
      {showInstallModal && selectedTemplate && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={() => setShowInstallModal(false)}>
          <div className="bg-card border rounded-lg p-6 w-96 shadow-xl" onClick={e => e.stopPropagation()}>
            <h3 className="text-lg font-semibold mb-2">Install MCP Server</h3>
            <p className="text-sm text-muted-foreground mb-4">
              This will install <strong>{selectedTemplate.name}</strong> as an MCP server.
            </p>
            <div className="bg-muted p-3 rounded mb-4 font-mono text-sm">
              <div className="text-xs text-muted-foreground mb-1">Command:</div>
              <div>{selectedTemplate.command}</div>
              <div className="text-xs text-muted-foreground mt-2 mb-1">Arguments:</div>
              <div>{selectedTemplate.args.join(' ')}</div>
            </div>
            <div className="flex gap-2 justify-end">
              <button
                onClick={() => setShowInstallModal(false)}
                className="px-4 py-2 border rounded hover:bg-muted"
              >
                Cancel
              </button>
              <button
                onClick={confirmInstall}
                disabled={installing}
                className="px-4 py-2 bg-purple-600 text-white rounded hover:bg-purple-700 disabled:opacity-50 flex items-center gap-2"
              >
                {installing ? (
                  <>
                    <svg className="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                      <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                      <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                    </svg>
                    Installing...
                  </>
                ) : (
                  'Install Server'
                )}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
