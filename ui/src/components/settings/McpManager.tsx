import { useState, useEffect, useCallback, useRef } from 'react';
import { apiGet, apiPost, apiDelete } from '../../lib/api';
import { McpMarketplace } from './McpMarketplace';

interface McpServer {
  id: string;
  name: string;
  command: string;
  args: string[];
  env: Record<string, string>;
  enabled: boolean;
  status: string;
  last_error: string | null;
}

interface McpTool {
  id: string;
  server_id: string;
  name: string;
  description: string | null;
  input_schema: Record<string, unknown>;
}

interface CreateServerRequest {
  name: string;
  command: string;
  args?: string[];
  env?: Record<string, string>;
  enabled?: boolean;
}

const STATUS_CONFIG: Record<string, { color: string; bg: string; label: string; pulse: boolean }> = {
  connected: { color: 'text-green-500', bg: 'bg-green-500/10', label: 'Connected', pulse: false },
  connecting: { color: 'text-yellow-500', bg: 'bg-yellow-500/10', label: 'Connecting', pulse: true },
  disconnected: { color: 'text-[var(--color-text-muted)]', bg: 'bg-[var(--color-muted)]', label: 'Disconnected', pulse: false },
  error: { color: 'text-red-500', bg: 'bg-red-500/10', label: 'Error', pulse: false },
};

const POLL_INTERVAL_MS = 5000;

export function McpManager() {
  const [servers, setServers] = useState<McpServer[]>([]);
  const [tools, setTools] = useState<McpTool[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAddForm, setShowAddForm] = useState(false);
  const [selectedServer, setSelectedServer] = useState<McpServer | null>(null);
  const [selectedTool, setSelectedTool] = useState<McpTool | null>(null);
  const [connecting, setConnecting] = useState<string | null>(null);
  const [callingTool, setCallingTool] = useState<string | null>(null);
  const [toolResult, setToolResult] = useState<string | null>(null);
  const [toolError, setToolError] = useState<string | null>(null);
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);
  const [isPolling, setIsPolling] = useState(false);
  
  const pollingRef = useRef<number | null>(null);
  const prevServerStatuses = useRef<Map<string, string>>(new Map());

  const [formData, setFormData] = useState<CreateServerRequest>({
    name: '',
    command: '',
    args: [],
    env: {},
    enabled: true,
  });

  const [toolArgs, setToolArgs] = useState<string>('{}');

  const loadServers = useCallback(async () => {
    try {
      const res = await apiGet('/api/v1/mcp/servers');
      if (res.ok) {
        const data = await res.json();
        setServers(data);
      }
    } catch (err) {
      console.error('Failed to load MCP servers:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  const loadTools = useCallback(async (serverId: string) => {
    try {
      const res = await apiGet(`/api/v1/mcp/servers/${serverId}/tools`);
      if (res.ok) {
        const data = await res.json();
        setTools(data);
      }
    } catch (err) {
      console.error('Failed to load MCP tools:', err);
    }
  }, []);

  useEffect(() => {
    loadServers();
  }, [loadServers]);

  useEffect(() => {
    if (selectedServer && selectedServer.status === 'connected') {
      loadTools(selectedServer.id);
    } else {
      setTools([]);
    }
  }, [selectedServer, loadTools]);

  useEffect(() => {
    const pollServers = async () => {
      try {
        const res = await apiGet('/api/v1/mcp/servers');
        if (res.ok) {
          const data: McpServer[] = await res.json();
          
          data.forEach(server => {
            const prevStatus = prevServerStatuses.current.get(server.id);
            if (prevStatus && prevStatus !== server.status) {
              console.log(`[MCP] Server "${server.name}" status changed: ${prevStatus} → ${server.status}`);
            }
            prevServerStatuses.current.set(server.id, server.status);
          });
          
          setServers(data);
          setLastUpdated(new Date());
          setIsPolling(true);
          
          if (selectedServer) {
            const updated = data.find(s => s.id === selectedServer.id);
            if (updated && updated.status !== selectedServer.status) {
              setSelectedServer(updated);
            }
          }
        }
      } catch (err) {
        console.error('Failed to poll MCP servers:', err);
      }
    };
    
    pollServers();
    
    pollingRef.current = window.setInterval(pollServers, POLL_INTERVAL_MS);
    
    return () => {
      if (pollingRef.current) {
        clearInterval(pollingRef.current);
        pollingRef.current = null;
      }
      setIsPolling(false);
    };
  }, [selectedServer]);

  const handleAdd = async () => {
    try {
      const res = await apiPost('/api/v1/mcp/servers', formData);
      if (res.ok) {
        loadServers();
        setShowAddForm(false);
        setFormData({ name: '', command: '', args: [], env: {}, enabled: true });
      }
    } catch (err) {
      console.error('Failed to add MCP server:', err);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Delete this MCP server?')) return;
    try {
      const res = await apiDelete(`/api/v1/mcp/servers/${id}`);
      if (res.ok) {
        if (selectedServer?.id === id) {
          setSelectedServer(null);
          setSelectedTool(null);
        }
        loadServers();
      }
    } catch (err) {
      console.error('Failed to delete MCP server:', err);
    }
  };

  const handleConnect = async (id: string) => {
    setConnecting(id);
    try {
      const res = await apiPost(`/api/v1/mcp/servers/${id}/connect`, {});
      if (res.ok) {
        loadServers();
        const server = servers.find(s => s.id === id);
        if (server) {
          setSelectedServer({ ...server, status: 'connected' });
        }
      } else {
        const data = await res.json();
        console.error('Connection failed:', data);
      }
    } catch (err) {
      console.error('Failed to connect MCP server:', err);
    } finally {
      setConnecting(null);
    }
  };

  const handleDisconnect = async (id: string) => {
    try {
      const res = await apiPost(`/api/v1/mcp/servers/${id}/disconnect`, {});
      if (res.ok) {
        loadServers();
        setTools([]);
        setSelectedTool(null);
      }
    } catch (err) {
      console.error('Failed to disconnect MCP server:', err);
    }
  };

  const handleCallTool = async () => {
    if (!selectedServer || !selectedTool) return;
    
    setCallingTool(selectedTool.id);
    setToolResult(null);
    setToolError(null);
    
    try {
      let args = {};
      try {
        args = JSON.parse(toolArgs);
      } catch {
        args = {};
      }
      
      const res = await apiPost(`/api/v1/mcp/servers/${selectedServer.id}/tools/${selectedTool.name}`, {
        arguments: args,
      });
      
      const data = await res.json();
      if (data.success) {
        setToolResult(data.content);
      } else {
        setToolError(data.error || 'Tool execution failed');
      }
    } catch (err) {
      setToolError(String(err));
    } finally {
      setCallingTool(null);
    }
  };

  const getStatusStyle = (status: string) => {
    return STATUS_CONFIG[status] || STATUS_CONFIG.disconnected;
  };

  const getStatusDot = (status: string) => {
    const style = getStatusStyle(status);
    return (
      <span className={`inline-flex items-center gap-1.5 px-2 py-1 rounded-full text-xs ${style.bg} ${style.color} border border-current/20`}>
        <span className={`w-1.5 h-1.5 rounded-full bg-current ${style.pulse ? 'animate-pulse' : ''}`} />
        {style.label}
      </span>
    );
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-[var(--color-text-muted)] flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="animate-spin">
            <line x1="12" y1="2" x2="12" y2="6"></line>
            <line x1="12" y1="18" x2="12" y2="22"></line>
            <line x1="4.93" y1="4.93" x2="7.76" y2="7.76"></line>
            <line x1="16.24" y1="16.24" x2="19.07" y2="19.07"></line>
            <line x1="2" y1="12" x2="6" y2="12"></line>
            <line x1="18" y1="12" x2="22" y2="12"></line>
            <line x1="4.93" y1="19.07" x2="7.76" y2="16.24"></line>
            <line x1="16.24" y1="7.76" x2="19.07" y2="4.93"></line>
          </svg>
          Loading MCP servers...
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      <McpMarketplace />
      {/* Header */}
      <div className="border-b border-[var(--color-border)] p-4 bg-[var(--color-panel)]">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-[#4248f1]/10 flex items-center justify-center">
              <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#4248f1" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <polyline points="16 18 22 12 16 6"></polyline>
                <polyline points="8 6 2 12 8 18"></polyline>
              </svg>
            </div>
            <div>
              <h2 className="text-xl font-semibold">MCP Servers</h2>
              <p className="text-sm text-[var(--color-text-muted)]">Model Context Protocol integrations</p>
            </div>
          </div>
          <div className="flex items-center gap-4">
            {lastUpdated && (
              <div className="flex items-center gap-2 text-xs text-[var(--color-text-muted)]">
                <span className={`w-2 h-2 rounded-full ${isPolling ? 'bg-green-500 animate-pulse' : 'bg-[var(--color-muted)]'}`} />
                <span>
                  {isPolling ? 'Live' : 'Paused'} • Updated {lastUpdated.toLocaleTimeString()}
                </span>
              </div>
            )}
            <button
              onClick={() => setShowAddForm(true)}
              className="px-4 py-2 bg-[#4248f1] text-white rounded-lg hover:bg-[#353bc5] transition-colors flex items-center gap-2"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <line x1="12" y1="5" x2="12" y2="19"></line>
                <line x1="5" y1="12" x2="19" y2="12"></line>
              </svg>
              Add Server
            </button>
          </div>
        </div>
      </div>

      {/* Add Form */}
      {showAddForm && (
        <div className="border-b border-[var(--color-border)] p-4 bg-[var(--color-muted)]/20">
          <h3 className="font-semibold mb-3 flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="12" y1="5" x2="12" y2="19"></line>
              <line x1="5" y1="12" x2="19" y2="12"></line>
            </svg>
            Add MCP Server
          </h3>
          <div className="space-y-3">
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="text-sm text-[var(--color-text-muted)] block mb-1.5">Name</label>
                <input
                  type="text"
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                  className="w-full px-3 py-2.5 border border-[var(--color-border)] rounded-lg bg-[var(--color-background)] text-[var(--color-text)]"
                  placeholder="my-mcp-server"
                />
              </div>
              <div>
                <label className="text-sm text-[var(--color-text-muted)] block mb-1.5">Command</label>
                <input
                  type="text"
                  value={formData.command}
                  onChange={(e) => setFormData({ ...formData, command: e.target.value })}
                  className="w-full px-3 py-2.5 border border-[var(--color-border)] rounded-lg bg-[var(--color-background)] text-[var(--color-text)] font-mono text-sm"
                  placeholder="npx"
                />
              </div>
            </div>
            <div>
              <label className="text-sm text-[var(--color-text-muted)] block mb-1.5">Arguments (space-separated)</label>
              <input
                type="text"
                value={formData.args?.join(' ') || ''}
                onChange={(e) => setFormData({ ...formData, args: e.target.value.split(' ').filter(Boolean) })}
                className="w-full px-3 py-2.5 border border-[var(--color-border)] rounded-lg bg-[var(--color-background)] text-[var(--color-text)] font-mono text-sm"
                placeholder="-y @modelcontextprotocol/server-filesystem /path/to/files"
              />
            </div>
            <div className="flex gap-2">
              <button
                onClick={handleAdd}
                disabled={!formData.name || !formData.command}
                className="px-4 py-2 bg-[#4248f1] text-white rounded-lg hover:bg-[#353bc5] disabled:opacity-50 transition-colors"
              >
                Add Server
              </button>
              <button
                onClick={() => setShowAddForm(false)}
                className="px-4 py-2 border border-[var(--color-border)] rounded-lg hover:bg-[var(--color-muted)] transition-colors"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}

      <div className="flex-1 flex overflow-hidden">
        {/* Servers List */}
        <div className="w-1/2 border-r border-[var(--color-border)] overflow-y-auto p-4">
          {servers.length === 0 ? (
            <div className="text-center py-12 text-[var(--color-text-muted)]">
              <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-[var(--color-muted)] flex items-center justify-center">
                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <polyline points="16 18 22 12 16 6"></polyline>
                  <polyline points="8 6 2 12 8 18"></polyline>
                </svg>
              </div>
              <p className="font-medium">No MCP servers configured</p>
              <p className="text-sm mt-1">Click "Add Server" to connect an MCP server</p>
            </div>
          ) : (
            <div className="space-y-2">
              {servers.map((server) => (
                <div
                  key={server.id}
                  className={`border border-[var(--color-border)] rounded-xl p-3 cursor-pointer transition-all ${
                    selectedServer?.id === server.id 
                      ? 'ring-2 ring-[#4248f1] bg-[#4248f1]/5' 
                      : 'hover:bg-[var(--color-muted)]/30'
                  }`}
                  onClick={() => setSelectedServer(server)}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <div className="p-1.5 bg-[var(--color-muted)] rounded">
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-[var(--color-text-muted)]">
                          <polyline points="5 12 12 5 19 12"></polyline>
                          <line x1="12" y1="5" x2="12" y2="19"></line>
                        </svg>
                      </div>
                      <div>
                        <h4 className="font-medium">{server.name}</h4>
                        <p className="text-xs text-[var(--color-text-muted)] font-mono">{server.command} {server.args?.join(' ')}</p>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      {getStatusDot(server.status)}
                    </div>
                  </div>
                  <div className="flex gap-2 mt-3">
                    {server.status === 'connected' ? (
                      <button
                        onClick={(e) => { e.stopPropagation(); handleDisconnect(server.id); }}
                        className="px-3 py-1.5 text-xs border border-[var(--color-border)] rounded-lg hover:bg-[var(--color-muted)]"
                      >
                        Disconnect
                      </button>
                    ) : (
                      <button
                        onClick={(e) => { e.stopPropagation(); handleConnect(server.id); }}
                        disabled={connecting === server.id}
                        className="px-3 py-1.5 text-xs bg-[#4248f1] text-white rounded-lg hover:bg-[#353bc5] disabled:opacity-50"
                      >
                        {connecting === server.id ? 'Connecting...' : 'Connect'}
                      </button>
                    )}
                    <button
                      onClick={(e) => { e.stopPropagation(); handleDelete(server.id); }}
                      className="px-3 py-1.5 text-xs text-red-500 hover:bg-red-500/10 rounded-lg"
                    >
                      Delete
                    </button>
                  </div>
                  {server.last_error && (
                    <p className="text-xs text-red-500 mt-2">Error: {server.last_error}</p>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Tools Panel */}
        <div className="w-1/2 overflow-y-auto p-4 bg-[var(--color-background)]">
          {!selectedServer ? (
            <div className="text-center py-12 text-[var(--color-text-muted)]">
              <p>Select a server to view tools</p>
            </div>
          ) : selectedServer.status !== 'connected' ? (
            <div className="text-center py-12 text-[var(--color-text-muted)]">
              <p>Connect to a server to view available tools</p>
            </div>
          ) : tools.length === 0 ? (
            <div className="text-center py-12 text-[var(--color-text-muted)]">
              <div className="w-12 h-12 mx-auto mb-3 rounded-full bg-[var(--color-muted)] flex items-center justify-center">
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <line x1="8" y1="6" x2="21" y2="6"></line>
                  <line x1="8" y1="12" x2="21" y2="12"></line>
                  <line x1="8" y1="18" x2="21" y2="18"></line>
                  <line x1="3" y1="6" x2="3.01" y2="6"></line>
                  <line x1="3" y1="12" x2="3.01" y2="12"></line>
                  <line x1="3" y1="18" x2="3.01" y2="18"></line>
                </svg>
              </div>
              <p className="font-medium">No tools available</p>
              <p className="text-sm mt-1">This server has no tools exposed</p>
            </div>
          ) : (
            <div className="space-y-3">
              <h3 className="font-semibold flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <line x1="8" y1="6" x2="21" y2="6"></line>
                  <line x1="8" y1="12" x2="21" y2="12"></line>
                  <line x1="8" y1="18" x2="21" y2="18"></line>
                  <line x1="3" y1="6" x2="3.01" y2="6"></line>
                  <line x1="3" y1="12" x2="3.01" y2="12"></line>
                  <line x1="3" y1="18" x2="3.01" y2="18"></line>
                </svg>
                Available Tools ({tools.length})
              </h3>
              {tools.map((tool) => (
                <div
                  key={tool.id}
                  className={`border border-[var(--color-border)] rounded-lg p-3 cursor-pointer transition-all ${
                    selectedTool?.id === tool.id
                      ? 'ring-2 ring-[#4248f1] bg-[#4248f1]/5'
                      : 'hover:bg-[var(--color-muted)]/30'
                  }`}
                  onClick={() => setSelectedTool(tool)}
                >
                  <div className="flex items-center gap-2">
                    <span className="px-2 py-0.5 rounded bg-[#4248f1]/10 text-[#4248f1] text-xs font-mono border border-[#4248f1]/20">
                      MCP
                    </span>
                    <h4 className="font-medium font-mono text-sm">{tool.name}</h4>
                  </div>
                  {tool.description && (
                    <p className="text-xs text-[var(--color-text-muted)] mt-1">{tool.description}</p>
                  )}
                </div>
              ))}

              {selectedTool && (
                <div className="border-t border-[var(--color-border)] pt-4 mt-4">
                  <h4 className="font-semibold mb-3 flex items-center gap-2">
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                      <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon>
                    </svg>
                    Execute Tool
                  </h4>
                  <div className="space-y-3">
                    <div>
                      <label className="text-sm text-[var(--color-text-muted)] block mb-1.5">Arguments (JSON)</label>
                      <textarea
                        value={toolArgs}
                        onChange={(e) => setToolArgs(e.target.value)}
                        className="w-full px-3 py-2 border border-[var(--color-border)] rounded-lg bg-[var(--color-background)] text-[var(--color-text)] font-mono text-sm h-24 resize-none"
                        placeholder='{"key": "value"}'
                      />
                    </div>
                    <button
                      onClick={handleCallTool}
                      disabled={callingTool !== null}
                      className="w-full px-4 py-2 bg-[#4248f1] text-white rounded-lg hover:bg-[#353bc5] disabled:opacity-50 flex items-center justify-center gap-2"
                    >
                      {callingTool ? (
                        <>
                          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="animate-spin">
                            <line x1="12" y1="2" x2="12" y2="6"></line>
                            <line x1="12" y1="18" x2="12" y2="22"></line>
                            <line x1="4.93" y1="4.93" x2="7.76" y2="7.76"></line>
                            <line x1="16.24" y1="16.24" x2="19.07" y2="19.07"></line>
                          </svg>
                          Executing...
                        </>
                      ) : (
                        <>
                          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <polygon points="5 3 19 12 5 21 5 3"></polygon>
                          </svg>
                          Execute
                        </>
                      )}
                    </button>

                    {toolResult && (
                      <div className="border border-green-500/20 rounded-lg p-3 bg-green-500/5">
                        <h5 className="text-sm font-medium text-green-500 mb-2 flex items-center gap-2">
                          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
                            <polyline points="22 4 12 14.01 9 11.01"></polyline>
                          </svg>
                          Result
                        </h5>
                        <pre className="text-xs font-mono whitespace-pre-wrap bg-[var(--color-background)] p-2 rounded border border-[var(--color-border)] overflow-x-auto">
                          {toolResult}
                        </pre>
                      </div>
                    )}

                    {toolError && (
                      <div className="border border-red-500/20 rounded-lg p-3 bg-red-500/5">
                        <h5 className="text-sm font-medium text-red-500 mb-2 flex items-center gap-2">
                          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <circle cx="12" cy="12" r="10"></circle>
                            <line x1="15" y1="9" x2="9" y2="15"></line>
                            <line x1="9" y1="9" x2="15" y2="15"></line>
                          </svg>
                          Error
                        </h5>
                        <pre className="text-xs font-mono whitespace-pre-wrap bg-[var(--color-background)] p-2 rounded border border-[var(--color-border)] overflow-x-auto">
                          {toolError}
                        </pre>
                      </div>
                    )}
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
