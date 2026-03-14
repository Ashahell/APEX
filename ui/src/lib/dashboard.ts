import { apiGet, apiPost, apiPut, apiDelete } from './api';

// Dashboard Layout
export interface DashboardLayout {
  id: string;
  layout_config: Record<string, unknown>;
  theme: string;
}

export async function getDashboardLayout(): Promise<DashboardLayout | null> {
  const res = await apiGet('/api/v1/dashboard/layout');
  if (!res.ok) return null;
  return res.json();
}

export async function updateDashboardLayout(
  layoutConfig: Record<string, unknown>,
  theme?: string
): Promise<DashboardLayout> {
  const res = await apiPut('/api/v1/dashboard/layout', {
    layout_config: JSON.stringify(layoutConfig),
    theme: theme || 'agentzero',
  });
  return res.json();
}

// Pinned Messages
export interface PinnedMessage {
  id: string;
  message_id: string;
  channel: string;
  thread_id?: string;
  task_id?: string;
  pinned_by: string;
  pin_note?: string;
  pinned_at: string;
}

export interface PinMessageRequest {
  message_id: string;
  channel: string;
  thread_id?: string;
  task_id?: string;
  pin_note?: string;
}

export async function listPins(channel?: string, limit = 50): Promise<PinnedMessage[]> {
  const params = new URLSearchParams({ limit: String(limit) });
  if (channel) params.set('channel', channel);
  
  const res = await apiGet(`/api/v1/dashboard/pins?${params}`);
  if (!res.ok) return [];
  return res.json();
}

export async function pinMessage(req: PinMessageRequest): Promise<PinnedMessage> {
  const res = await apiPost('/api/v1/dashboard/pins', req);
  return res.json();
}

export async function unpinMessage(id: string): Promise<void> {
  const res = await apiDelete(`/api/v1/dashboard/pins/${id}`);
  if (!res.ok) {
    throw new Error('Failed to unpin message');
  }
}

// Bookmarks
export interface Bookmark {
  id: string;
  message_id: string;
  channel: string;
  thread_id?: string;
  bookmark_note?: string;
  created_at: string;
}

export interface AddBookmarkRequest {
  message_id: string;
  channel: string;
  thread_id?: string;
  bookmark_note?: string;
}

export async function addBookmark(req: AddBookmarkRequest): Promise<Bookmark> {
  const res = await apiPost('/api/v1/dashboard/bookmarks', req);
  return res.json();
}

// Chat Search
export interface ChatMessage {
  id: string;
  message_id: string;
  channel: string;
  thread_id?: string;
  author: string;
  content: string;
  role: string;
  metadata?: Record<string, unknown>;
  created_at: string;
}

export interface SearchResult {
  messages: ChatMessage[];
  total: number;
}

export async function searchMessages(
  query: string,
  channel?: string,
  limit = 20,
  offset = 0
): Promise<SearchResult> {
  const params = new URLSearchParams({ q: query, limit: String(limit), offset: String(offset) });
  if (channel) params.set('channel', channel);
  
  const res = await apiGet(`/api/v1/dashboard/search?${params}`);
  return res.json();
}

// Session Management
export interface SessionMetadata {
  session_id: string;
  model?: string;
  thinking_level: string;
  verbose_level: string;
  fast_mode: boolean;
  send_policy: string;
  activation_mode: string;
  group_policy?: string;
  updated_at: string;
}

export interface UpdateSessionRequest {
  model?: string;
  thinking_level?: string;
  verbose_level?: string;
  fast_mode?: boolean;
  send_policy?: string;
  activation_mode?: string;
  group_policy?: string;
}

export async function listSessions(limit = 50): Promise<SessionMetadata[]> {
  const res = await apiGet(`/api/v1/dashboard/sessions?limit=${limit}`);
  if (!res.ok) return [];
  return res.json();
}

export async function getSession(sessionId: string): Promise<SessionMetadata> {
  const res = await apiGet(`/api/v1/dashboard/sessions/${sessionId}`);
  return res.json();
}

export async function updateSession(
  sessionId: string,
  req: UpdateSessionRequest
): Promise<SessionMetadata> {
  const res = await apiPut(`/api/v1/dashboard/sessions/${sessionId}`, req);
  return res.json();
}

// Command Palette
export interface CommandHistory {
  command: string;
  command_type: string;
  frequency: number;
  last_used: string;
}

export async function listCommands(
  commandType?: string,
  limit = 20
): Promise<CommandHistory[]> {
  const params = new URLSearchParams({ limit: String(limit) });
  if (commandType) params.set('command_type', commandType);
  
  const res = await apiGet(`/api/v1/dashboard/commands?${params}`);
  if (!res.ok) return [];
  return res.json();
}

export async function recordCommand(command: string, commandType: string): Promise<void> {
  const res = await apiPost('/api/v1/dashboard/commands', { command, command_type: commandType });
  if (!res.ok) {
    throw new Error('Failed to record command');
  }
}

// Export
export interface ExportRequest {
  session_id: string;
  export_format: 'json' | 'markdown' | 'pdf' | 'txt';
  export_range: 'all' | 'selection' | 'date_range';
  date_from?: string;
  date_to?: string;
}

export interface Export {
  id: string;
  session_id: string;
  export_format: string;
  export_range: string;
  status: 'pending' | 'completed' | 'failed';
  created_at: string;
}

export async function createExport(req: ExportRequest): Promise<Export> {
  const res = await apiPost('/api/v1/dashboard/export', req);
  return res.json();
}

export async function getExport(id: string): Promise<Export> {
  const res = await apiGet(`/api/v1/dashboard/export/${id}`);
  return res.json();
}
