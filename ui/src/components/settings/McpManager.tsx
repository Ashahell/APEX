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
  connected: { color: 'text-green-400', bg: 'bg-green-500/20', label: 'Connected', pulse: false },
  connecting: { color: 'text-yellow-400', bg: 'bg-yellow-500/20', label: 'Connecting', pulse: true },
  disconnected: { color: 'text-gray-400', bg: 'bg-gray-500/20', label: 'Disconnected', pulse: false },
  error: { color: 'text-red-400', bg: 'bg-red-500/20', label: 'Error', pulse: false },
};

const POLL_INTERVAL_MS = 5000; // Poll every 5 seconds for status updates

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

  // Polling for real-time status updates
  useEffect(() => {
    const pollServers = async () => {
      try {
        const res = await apiGet('/api/v1/mcp/servers');
        if (res.ok) {
          const data: McpServer[] = await res.json();
          
          // Check for status changes and log
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
          
          // Update selected server if its status changed
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
    
    // Initial poll
    pollServers();
    
    // Set up polling interval
    pollingRef.current = window.setInterval(pollServers, POLL_INTERVAL_MS);
    
    // Cleanup on unmount
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
      <span className={`inline-flex items-center gap-1.5 px-2 py-1 rounded-full text-xs ${style.bg} ${style.color}`}>
        <span className={`w-1.5 h-1.5 rounded-full bg-current ${style.pulse ? 'animate-pulse' : ''}`} />
        {style.label}
      </span>
    );
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <span className="text-muted-foreground">Loading MCP servers...</span>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      <McpMarketplace />
      <div className="border-b p-4 bg-gradient-to-r from-cyan-950/50 to-blue-950/50">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-cyan-500/20 rounded-lg">
              <svg className="w-6 h-6 text-cyan-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
              </svg>
            </div>
            <div>
              <h2 className="text-2xl font-semibold">MCP Servers</h2>
              <p className="text-muted-foreground text-sm">Model Context Protocol integrations</p>
            </div>
          </div>
          <div className="flex items-center gap-4">
            {/* Real-time status indicator */}
            {lastUpdated && (
              <div className="flex items-center gap-2 text-xs text-muted-foreground">
                <span className={`w-2 h-2 rounded-full ${isPolling ? 'bg-green-400 animate-pulse' : 'bg-gray-400'}`} />
                <span>
                  {isPolling ? 'Live' : 'Paused'} • Updated {lastUpdated.toLocaleTimeString()}
                </span>
              </div>
            )}
            <button
              onClick={() => setShowAddForm(true)}
              className="px-4 py-2 bg-cyan-600 text-white rounded-lg hover:bg-cyan-700 transition-colors flex items-center gap-2"
            >
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
              </svg>
              Add Server
            </button>
          </div>
        </div>
      </div>

      {showAddForm && (
        <div className="border-b p-4 bg-muted/30">
          <h3 className="font-semibold mb-3 flex items-center gap-2">
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
            </svg>
            Add MCP Server
          </h3>
          <div className="space-y-3">
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="text-sm text-muted-foreground block mb-1">Name</label>
                <input
                  type="text"
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                  className="w-full px-3 py-2 border rounded-lg bg-background"
                  placeholder="my-mcp-server"
                />
              </div>
              <div>
                <label className="text-sm text-muted-foreground block mb-1">Command</label>
                <input
                  type="text"
                  value={formData.command}
                  onChange={(e) => setFormData({ ...formData, command: e.target.value })}
                  className="w-full px-3 py-2 border rounded-lg bg-background font-mono"
                  placeholder="npx"
                />
              </div>
            </div>
            <div>
              <label className="text-sm text-muted-foreground block mb-1">Arguments (space-separated)</label>
              <input
                type="text"
                value={formData.args?.join(' ') || ''}
                onChange={(e) => setFormData({ ...formData, args: e.target.value.split(' ').filter(Boolean) })}
                className="w-full px-3 py-2 border rounded-lg bg-background font-mono text-sm"
                placeholder="-y @modelcontextprotocol/server-filesystem /path/to/files"
              />
            </div>
            <div className="flex gap-2">
              <button
                onClick={handleAdd}
                disabled={!formData.name || !formData.command}
                className="px-4 py-2 bg-cyan-600 text-white rounded-lg hover:bg-cyan-700 disabled:opacity-50 transition-colors"
              >
                Add Server
              </button>
              <button
                onClick={() => setShowAddForm(false)}
                className="px-4 py-2 border rounded-lg hover:bg-muted transition-colors"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}

      <div className="flex-1 flex overflow-hidden">
        <div className="w-1/2 border-r overflow-y-auto p-4">
          {servers.length === 0 ? (
            <div className="text-center py-12 text-muted-foreground">
              <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-muted flex items-center justify-center">
                <svg className="w-8 h-8" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
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
                  className={`border rounded-lg p-3 cursor-pointer transition-all ${
                    selectedServer?.id === server.id 
                      ? 'ring-2 ring-cyan-500 bg-cyan-500/10' 
                      : 'hover:bg-muted/50'
                  }`}
                  onClick={() => setSelectedServer(server)}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <div className="p-1.5 bg-muted rounded">
                        <svg className="w-4 h-4 text-muted-foreground" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 12h14M12 5l7 7-7 7" />
                        </svg>
                      </div>
                      <div>
                        <h4 className="font-medium">{server.name}</h4>
                        <p className="text-xs text-muted-foreground font-mono">{server.command} {server.args?.join(' ')}</p>
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
                        className="px-3 py-1 text-xs border rounded hover:bg-muted"
                      >
                        Disconnect
                      </button>
                    ) : (
                      <button
                        onClick={(e) => { e.stopPropagation(); handleConnect(server.id); }}
                        disabled={connecting === server.id}
                        className="px-3 py-1 text-xs bg-cyan-600 text-white rounded hover:bg-cyan-700 disabled:opacity-50"
                      >
                        {connecting === server.id ? 'Connecting...' : 'Connect'}
                      </button>
                    )}
                    <button
                      onClick={(e) => { e.stopPropagation(); handleDelete(server.id); }}
                      className="px-3 py-1 text-xs text-red-500 hover:bg-red-50 rounded"
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

        <div className="w-1/2 overflow-y-auto p-4">
          {!selectedServer ? (
            <div className="text-center py-12 text-muted-foreground">
              <p>Select a server to view tools</p>
            </div>
          ) : selectedServer.status !== 'connected' ? (
            <div className="text-center py-12 text-muted-foreground">
              <p>Connect to a server to view available tools</p>
            </div>
          ) : tools.length === 0 ? (
            <div className="text-center py-12 text-muted-foreground">
              <div className="w-12 h-12 mx-auto mb-3 rounded-full bg-muted flex items-center justify-center">
                <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 10h16M4 14h16M4 18h16" />
                </svg>
              </div>
              <p className="font-medium">No tools available</p>
              <p className="text-sm mt-1">This server has no tools exposed</p>
            </div>
          ) : (
            <div className="space-y-3">
              <h3 className="font-semibold flex items-center gap-2">
                <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 10h16M4 14h16M4 18h16" />
                </svg>
                Available Tools ({tools.length})
              </h3>
              {tools.map((tool) => (
                <div
                  key={tool.id}
                  className={`border rounded-lg p-3 cursor-pointer transition-all ${
                    selectedTool?.id === tool.id
                      ? 'ring-2 ring-cyan-500 bg-cyan-500/10'
                      : 'hover:bg-muted/50'
                  }`}
                  onClick={() => setSelectedTool(tool)}
                >
                  <div className="flex items-center gap-2">
                    <span className="px-2 py-0.5 rounded bg-cyan-500/20 text-cyan-400 text-xs font-mono">
                      MCP
                    </span>
                    <h4 className="font-medium font-mono text-sm">{tool.name}</h4>
                  </div>
                  {tool.description && (
                    <p className="text-xs text-muted-foreground mt-1">{tool.description}</p>
                  )}
                </div>
              ))}

              {selectedTool && (
                <div className="border-t pt-4 mt-4">
                  <h4 className="font-semibold mb-3 flex items-center gap-2">
                    <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                    </svg>
                    Execute Tool
                  </h4>
                  <div className="space-y-3">
                    <div>
                      <label className="text-sm text-muted-foreground block mb-1">Arguments (JSON)</label>
                      <textarea
                        value={toolArgs}
                        onChange={(e) => setToolArgs(e.target.value)}
                        className="w-full px-3 py-2 border rounded-lg bg-background font-mono text-sm h-24 resize-none"
                        placeholder='{"key": "value"}'
                      />
                    </div>
                    <button
                      onClick={handleCallTool}
                      disabled={callingTool !== null}
                      className="w-full px-4 py-2 bg-cyan-600 text-white rounded-lg hover:bg-cyan-700 disabled:opacity-50 flex items-center justify-center gap-2"
                    >
                      {callingTool ? (
                        <>
                          <svg className="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                          </svg>
                          Executing...
                        </>
                      ) : (
                        <>
                          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                          </svg>
                          Execute
                        </>
                      )}
                    </button>

                    {toolResult && (
                      <div className="border rounded-lg p-3 bg-green-500/10">
                        <h5 className="text-sm font-medium text-green-400 mb-2">Result</h5>
                        <pre className="text-xs font-mono whitespace-pre-wrap bg-background p-2 rounded overflow-x-auto">
                          {toolResult}
                        </pre>
                      </div>
                    )}

                    {toolError && (
                      <div className="border rounded-lg p-3 bg-red-500/10">
                        <h5 className="text-sm font-medium text-red-400 mb-2">Error</h5>
                        <pre className="text-xs font-mono whitespace-pre-wrap bg-background p-2 rounded overflow-x-auto">
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
