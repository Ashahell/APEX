/// <reference types="vite/client" />

const API_BASE = (import.meta as unknown as { env: { VITE_API_URL?: string } }).env.VITE_API_URL || 'http://localhost:3000';
const SHARED_SECRET = (import.meta as unknown as { env: { VITE_APEX_SHARED_SECRET?: string } }).env.VITE_APEX_SHARED_SECRET || 'dev-secret-change-in-production';

async function computeSignature(method: string, path: string, body: string): Promise<{ signature: string; timestamp: number }> {
  const timestamp = Math.floor(Date.now() / 1000);
  
  const encoder = new TextEncoder();
  const keyData = encoder.encode(SHARED_SECRET);
  const key = await globalThis.crypto.subtle.importKey(
    'raw',
    keyData,
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign']
  );
  const message = `${timestamp}${method}${path}${body}`;
  const signatureBuffer = await globalThis.crypto.subtle.sign('HMAC', key, encoder.encode(message));
  const signature = Array.from(new Uint8Array(signatureBuffer))
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
  
  return { signature, timestamp };
}

export async function apiFetch(
  url: string,
  options: RequestInit = {}
): Promise<Response> {
  const method = options.method || 'GET';
  const urlPath = new URL(url, API_BASE).pathname;
  const body = typeof options.body === 'string' ? options.body : JSON.stringify(options.body || {});
  
  const { signature, timestamp } = await computeSignature(method, urlPath, body);

  const headers = new Headers(options.headers);
  headers.set('Content-Type', 'application/json');
  headers.set('X-APEX-Signature', signature);
  headers.set('X-APEX-Timestamp', timestamp.toString());

  return fetch(`${API_BASE}${url}`, { ...options, headers });
}

export async function apiGet(path: string): Promise<Response> {
  return apiFetch(path, { method: 'GET' });
}

export async function apiPost(path: string, body: unknown): Promise<Response> {
  return apiFetch(path, { method: 'POST', body: JSON.stringify(body) });
}

export async function apiPut(path: string, body: unknown): Promise<Response> {
  return apiFetch(path, { method: 'PUT', body: JSON.stringify(body) });
}

export async function apiDelete(path: string): Promise<Response> {
  return apiFetch(path, { method: 'DELETE' });
}

export interface Channel {
  id: string;
  name: string;
  description?: string;
  created_at: string;
  updated_at: string;
}

export interface DecisionJournalEntry {
  id: string;
  task_id?: string;
  title: string;
  context?: string;
  decision: string;
  rationale?: string;
  outcome?: string;
  tags?: string;
  created_at: string;
  updated_at: string;
}

export async function listChannels(): Promise<Channel[]> {
  const response = await apiGet('/api/v1/channels');
  if (!response.ok) throw new Error('Failed to list channels');
  return response.json();
}

export async function createChannel(name: string, description?: string): Promise<Channel> {
  const response = await apiPost('/api/v1/channels', { name, description });
  if (!response.ok) throw new Error('Failed to create channel');
  return response.json();
}

export async function updateChannel(id: string, name: string, description?: string): Promise<Channel> {
  const response = await apiPut(`/api/v1/channels/${id}`, { name, description });
  if (!response.ok) throw new Error('Failed to update channel');
  return response.json();
}

export async function deleteChannel(id: string): Promise<void> {
  const response = await apiDelete(`/api/v1/channels/${id}`);
  if (!response.ok) throw new Error('Failed to delete channel');
}

export async function listJournalEntries(limit = 50, offset = 0): Promise<DecisionJournalEntry[]> {
  const response = await apiGet(`/api/v1/journal?limit=${limit}&offset=${offset}`);
  if (!response.ok) throw new Error('Failed to list journal entries');
  return response.json();
}

export async function createJournalEntry(entry: Omit<DecisionJournalEntry, 'id' | 'created_at' | 'updated_at'>): Promise<DecisionJournalEntry> {
  const response = await apiPost('/api/v1/journal', entry);
  if (!response.ok) throw new Error('Failed to create journal entry');
  return response.json();
}

export async function updateJournalEntry(id: string, entry: Omit<DecisionJournalEntry, 'id' | 'created_at' | 'updated_at'>): Promise<DecisionJournalEntry> {
  const response = await apiPut(`/api/v1/journal/${id}`, entry);
  if (!response.ok) throw new Error('Failed to update journal entry');
  return response.json();
}

export async function deleteJournalEntry(id: string): Promise<void> {
  const response = await apiDelete(`/api/v1/journal/${id}`);
  if (!response.ok) throw new Error('Failed to delete journal entry');
}

export async function searchJournal(query: string, limit = 50): Promise<DecisionJournalEntry[]> {
  const response = await apiGet(`/api/v1/journal/search?q=${encodeURIComponent(query)}&limit=${limit}`);
  if (!response.ok) throw new Error('Failed to search journal');
  return response.json();
}

export interface Setting {
  key: string;
  value: string;
  encrypted: boolean;
}

export async function getSetting(key: string): Promise<Setting> {
  const response = await apiGet(`/api/v1/settings/${encodeURIComponent(key)}`);
  if (!response.ok) {
    if (response.status === 404) {
      throw new Error('Setting not found');
    }
    throw new Error('Failed to get setting');
  }
  return response.json();
}

export async function setSetting(key: string, value: string, encrypt = false): Promise<Setting> {
  const response = await apiPut(`/api/v1/settings/${encodeURIComponent(key)}`, { value, encrypt });
  if (!response.ok) throw new Error('Failed to set setting');
  return response.json();
}

export async function deleteSetting(key: string): Promise<void> {
  const response = await apiDelete(`/api/v1/settings/${encodeURIComponent(key)}`);
  if (!response.ok) throw new Error('Failed to delete setting');
}

export interface AuditEntry {
  id: number;
  prev_hash: string;
  hash: string;
  timestamp: string;
  action: string;
  entity_type: string;
  entity_id: string;
  details: string | null;
}

export interface AuditChainStatus {
  valid: boolean;
  total_entries: number;
}

export async function listAudit(limit = 50, offset = 0, entityType?: string, entityId?: string): Promise<AuditEntry[]> {
  let url = `/api/v1/audit?limit=${limit}&offset=${offset}`;
  if (entityType && entityId) {
    url = `/api/v1/audit/entity/${entityType}/${entityId}`;
  }
  const response = await apiGet(url);
  if (!response.ok) throw new Error('Failed to list audit');
  return response.json();
}

export async function getAuditChainStatus(): Promise<AuditChainStatus> {
  const response = await apiGet('/api/v1/audit/chain');
  if (!response.ok) throw new Error('Failed to get audit chain status');
  return response.json();
}

export async function createAudit(action: string, entityType: string, entityId: string, details?: string): Promise<AuditEntry> {
  const response = await apiPost('/api/v1/audit', { action, entity_type: entityType, entity_id: entityId, details });
  if (!response.ok) throw new Error('Failed to create audit');
  return response.json();
}

export { API_BASE };
