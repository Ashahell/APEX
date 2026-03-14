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

// ============ Fast Mode (NEW) ============

export interface FastModeState {
  session_id: string;
  fast_enabled: boolean;
  fast_model?: string;
  fast_config?: Record<string, unknown>;
  toggles?: Record<string, boolean>;
}

export async function getSessionFastMode(sessionId: string): Promise<FastModeState> {
  const response = await apiGet(`/api/v1/llms/sessions/${sessionId}/fast-mode`);
  if (!response.ok) throw new Error('Failed to get fast mode');
  return response.json();
}

export async function setSessionFastMode(
  sessionId: string,
  fastEnabled: boolean,
  fastModel?: string,
  fastConfig?: Record<string, unknown>,
  toggles?: Record<string, boolean>
): Promise<FastModeState> {
  const response = await apiPut(`/api/v1/llms/sessions/${sessionId}/fast-mode`, {
    fast_enabled: fastEnabled,
    fast_model: fastModel,
    fast_config: fastConfig ? JSON.stringify(fastConfig) : undefined,
    toggles: toggles ? JSON.stringify(toggles) : undefined,
  });
  if (!response.ok) throw new Error('Failed to set fast mode');
  return response.json();
}

// ============ Provider Plugins (NEW) ============

export interface ProviderPlugin {
  id: string;
  provider_type: string;
  name: string;
  base_url: string;
  api_key?: string;
  default_model?: string;
  config?: string;
  enabled: boolean;
  priority: number;
}

export async function listProviderPlugins(): Promise<ProviderPlugin[]> {
  const response = await apiGet('/api/v1/llms/plugins');
  if (!response.ok) throw new Error('Failed to list provider plugins');
  return response.json();
}

export async function createProviderPlugin(data: {
  provider_type: string;
  name: string;
  base_url: string;
  api_key?: string;
  default_model?: string;
  config?: string;
}): Promise<ProviderPlugin> {
  const response = await apiPost('/api/v1/llms/plugins', data);
  if (!response.ok) throw new Error('Failed to create provider plugin');
  return response.json();
}

export async function updateProviderPlugin(
  id: string,
  data: Partial<ProviderPlugin>
): Promise<ProviderPlugin> {
  const response = await apiPut(`/api/v1/llms/plugins/${id}`, data);
  if (!response.ok) throw new Error('Failed to update provider plugin');
  return response.json();
}

export async function deleteProviderPlugin(id: string): Promise<void> {
  const response = await apiDelete(`/api/v1/llms/plugins/${id}`);
  if (!response.ok) throw new Error('Failed to delete provider plugin');
}

// ============ Model Fallbacks (NEW) ============

export interface ModelFallback {
  id: string;
  primary_model: string;
  fallback_model: string;
  provider?: string;
  priority: number;
}

export async function listModelFallbacks(primaryModel?: string): Promise<ModelFallback[]> {
  const params = primaryModel ? `?primary_model=${encodeURIComponent(primaryModel)}` : '';
  const response = await apiGet(`/api/v1/llms/fallbacks${params}`);
  if (!response.ok) throw new Error('Failed to list model fallbacks');
  return response.json();
}

export async function addModelFallback(data: {
  primary_model: string;
  fallback_model: string;
  provider?: string;
  priority?: number;
}): Promise<ModelFallback> {
  const response = await apiPost('/api/v1/llms/fallbacks', data);
  if (!response.ok) throw new Error('Failed to add model fallback');
  return response.json();
}

export async function deleteModelFallback(id: string): Promise<void> {
  const response = await apiDelete(`/api/v1/llms/fallbacks/${id}`);
  if (!response.ok) throw new Error('Failed to delete model fallback');
}

// ============ Session Control (NEW - Phase 3) ============

export interface SessionYieldLog {
  id: string;
  session_id: string;
  child_session_id: string;
  reason?: string;
  yield_payload?: string;
  created_at: string;
}

export interface SessionResumeHistory {
  id: string;
  session_id: string;
  resumed_from: string;
  resume_type: string;
  context_summary?: string;
  created_at: string;
}

export interface SessionCheckpoint {
  id: string;
  session_id: string;
  checkpoint_name: string;
  checkpoint_data: string;
  description?: string;
  created_at: string;
}

export interface SessionAttachment {
  id: string;
  session_id: string;
  task_id?: string;
  file_name: string;
  file_type: string;
  file_size: number;
  encoding: string;
  uploaded_by: string;
  created_at: string;
}

export interface SessionState {
  session_id: string;
  state_data: Record<string, unknown>;
  checkpoint_id?: string;
  updated_at: string;
}

export async function yieldSession(
  sessionId: string,
  options?: {
    yieldPayload?: string;
    skipToolWork?: boolean;
    reason?: string;
  }
): Promise<{ yield_id: string; status: string; child_session_id: string }> {
  const response = await apiPost(`/api/v1/sessions/${sessionId}/yield`, {
    yield_payload: options?.yieldPayload,
    skip_tool_work: options?.skipToolWork,
    reason: options?.reason,
  });
  if (!response.ok) throw new Error('Failed to yield session');
  return response.json();
}

export async function getSessionYields(sessionId: string): Promise<SessionYieldLog[]> {
  const response = await apiGet(`/api/v1/sessions/${sessionId}/yields`);
  if (!response.ok) throw new Error('Failed to get session yields');
  return response.json();
}

export async function resumeSession(data: {
  resumeSessionId?: string;
  originalSessionId?: string;
  resumeType: string;
  contextSummary?: string;
}): Promise<{ session_id: string; resumed_from: string; context_loaded: boolean }> {
  const response = await apiPost('/api/v1/sessions/resume', {
    resume_session_id: data.resumeSessionId,
    original_session_id: data.originalSessionId,
    resume_type: data.resumeType,
    context_summary: data.contextSummary,
  });
  if (!response.ok) throw new Error('Failed to resume session');
  return response.json();
}

export async function getResumeHistory(sessionId: string): Promise<SessionResumeHistory[]> {
  const response = await apiGet(`/api/v1/sessions/${sessionId}/resume-history`);
  if (!response.ok) throw new Error('Failed to get resume history');
  return response.json();
}

// Session State
export async function getSessionState(sessionId: string): Promise<SessionState | null> {
  const response = await apiGet(`/api/v1/sessions/${sessionId}/state`);
  if (!response.ok) throw new Error('Failed to get session state');
  return response.json();
}

export async function saveSessionState(
  sessionId: string,
  stateData: Record<string, unknown>,
  checkpointId?: string
): Promise<SessionState> {
  const response = await apiPost(`/api/v1/sessions/${sessionId}/state`, {
    state_data: JSON.stringify(stateData),
    checkpoint_id: checkpointId,
  });
  if (!response.ok) throw new Error('Failed to save session state');
  return response.json();
}

export async function deleteSessionState(sessionId: string): Promise<void> {
  const response = await apiDelete(`/api/v1/sessions/${sessionId}/state`);
  if (!response.ok) throw new Error('Failed to delete session state');
}

// Checkpoints
export async function listCheckpoints(sessionId: string): Promise<SessionCheckpoint[]> {
  const response = await apiGet(`/api/v1/sessions/${sessionId}/checkpoints`);
  if (!response.ok) throw new Error('Failed to list checkpoints');
  return response.json();
}

export async function createCheckpoint(
  sessionId: string,
  checkpointName: string,
  checkpointData: string,
  description?: string
): Promise<SessionCheckpoint> {
  const response = await apiPost(`/api/v1/sessions/${sessionId}/checkpoints`, {
    checkpoint_name: checkpointName,
    checkpoint_data: checkpointData,
    description,
  });
  if (!response.ok) throw new Error('Failed to create checkpoint');
  return response.json();
}

export async function getCheckpoint(sessionId: string, checkpointId: string): Promise<SessionCheckpoint> {
  const response = await apiGet(`/api/v1/sessions/${sessionId}/checkpoints/${checkpointId}`);
  if (!response.ok) throw new Error('Failed to get checkpoint');
  return response.json();
}

export async function getCheckpointByName(sessionId: string, name: string): Promise<SessionCheckpoint> {
  const response = await apiGet(`/api/v1/sessions/${sessionId}/checkpoints/by-name/${name}`);
  if (!response.ok) throw new Error('Failed to get checkpoint by name');
  return response.json();
}

export async function deleteCheckpoint(sessionId: string, checkpointId: string): Promise<void> {
  const response = await apiDelete(`/api/v1/sessions/${sessionId}/checkpoints/${checkpointId}`);
  if (!response.ok) throw new Error('Failed to delete checkpoint');
}

// Attachments
export async function listAttachments(sessionId: string): Promise<SessionAttachment[]> {
  const response = await apiGet(`/api/v1/sessions/${sessionId}/attachments`);
  if (!response.ok) throw new Error('Failed to list attachments');
  return response.json();
}

export async function uploadAttachment(
  sessionId: string,
  data: {
    taskId?: string;
    fileName: string;
    fileType: string;
    fileSize: number;
    filePath: string;
    encoding?: string;
  }
): Promise<SessionAttachment> {
  const response = await apiPost(`/api/v1/sessions/${sessionId}/attachments`, {
    task_id: data.taskId,
    file_name: data.fileName,
    file_type: data.fileType,
    file_size: data.fileSize,
    file_path: data.filePath,
    encoding: data.encoding,
  });
  if (!response.ok) throw new Error('Failed to upload attachment');
  return response.json();
}

export async function getAttachment(sessionId: string, attachmentId: string): Promise<SessionAttachment> {
  const response = await apiGet(`/api/v1/sessions/${sessionId}/attachments/${attachmentId}`);
  if (!response.ok) throw new Error('Failed to get attachment');
  return response.json();
}

export async function deleteAttachment(sessionId: string, attachmentId: string): Promise<void> {
  const response = await apiDelete(`/api/v1/sessions/${sessionId}/attachments/${attachmentId}`);
  if (!response.ok) throw new Error('Failed to delete attachment');
}

// ============ PDF Tool (NEW - Phase 4) ============

export interface PdfDocument {
  id: string;
  file_name: string;
  file_hash: string;
  file_size: number;
  page_count: number | null;
  extracted_text: string | null;
  metadata: Record<string, unknown> | null;
  provider: string;
  model_used: string | null;
  created_at: string;
  expires_at: string | null;
}

export interface PdfExtractResponse {
  document_id: string;
  text: string;
  page_count: number;
  provider: string;
  model: string | null;
}

export interface PdfAnalyzeResponse {
  analysis: string;
  provider: string;
  model: string;
  tokens_used: number;
}

export async function listPdfDocuments(limit?: number, offset?: number): Promise<PdfDocument[]> {
  const params = new URLSearchParams();
  if (limit) params.set('limit', limit.toString());
  if (offset) params.set('offset', offset.toString());
  const query = params.toString();
  const response = await apiGet(`/api/v1/pdf/documents${query ? `?${query}` : ''}`);
  if (!response.ok) throw new Error('Failed to list PDF documents');
  return response.json();
}

export async function uploadPdf(file: File): Promise<{ document: PdfDocument; cached: boolean }> {
  const formData = new FormData();
  formData.append('file', file);
  
  const method = 'POST';
  const path = '/api/v1/pdf/upload';
  const { signature, timestamp } = await computeSignature(method, path, '');
  
  const response = await fetch(`${API_BASE}${path}`, {
    method,
    body: formData,
    headers: {
      'X-APEX-Signature': signature,
      'X-APEX-Timestamp': timestamp.toString(),
    },
  });
  
  if (!response.ok) throw new Error('Failed to upload PDF');
  return response.json();
}

export async function getPdfDocument(id: string): Promise<PdfDocument> {
  const response = await apiGet(`/api/v1/pdf/${id}`);
  if (!response.ok) throw new Error('Failed to get PDF document');
  return response.json();
}

export async function deletePdfDocument(id: string): Promise<void> {
  const response = await apiDelete(`/api/v1/pdf/${id}`);
  if (!response.ok) throw new Error('Failed to delete PDF document');
}

export async function extractPdfText(id: string, provider?: string): Promise<PdfExtractResponse> {
  const response = await apiPost(`/api/v1/pdf/${id}/extract`, { provider });
  if (!response.ok) throw new Error('Failed to extract PDF text');
  return response.json();
}

export async function analyzePdf(
  id: string,
  prompt: string,
  provider?: string
): Promise<PdfAnalyzeResponse> {
  const response = await apiPost(`/api/v1/pdf/${id}/analyze`, { prompt, provider });
  if (!response.ok) throw new Error('Failed to analyze PDF');
  return response.json();
}

export async function getPdfJobStatus(jobId: string): Promise<{
  id: string;
  document_id: string;
  status: string;
  error_message: string | null;
  started_at: string;
  completed_at: string | null;
}> {
  const response = await apiGet(`/api/v1/pdf/jobs/${jobId}`);
  if (!response.ok) throw new Error('Failed to get job status');
  return response.json();
}

// ============ Multimodal Memory (NEW - Phase 5) ============

export interface MultimodalConfig {
  image_indexing: boolean;
  audio_indexing: boolean;
  embedding_model: string;
  embedding_dim: number;
  enabled: boolean;
}

export interface MultimodalStats {
  total_embeddings: number;
  image_embeddings: number;
  audio_embeddings: number;
  text_embeddings: number;
  pending_jobs: number;
  processing_jobs: number;
}

export interface MultimodalEmbedding {
  id: string;
  memory_id: string;
  memory_type: string;
  modality: 'text' | 'image' | 'audio';
  mime_type: string | null;
  has_original_data: boolean;
  embedding_model: string;
  created_at: string;
}

export interface MultimodalSearchResult {
  memory_id: string;
  memory_type: string;
  modality: string;
  original_data: string | null;
  mime_type: string | null;
  score: number;
  created_at: string;
}

export async function getMultimodalConfig(): Promise<MultimodalConfig> {
  const response = await apiGet('/api/v1/memory/multimodal/config');
  if (!response.ok) throw new Error('Failed to get multimodal config');
  return response.json();
}

export async function updateMultimodalConfig(config: Partial<MultimodalConfig>): Promise<MultimodalConfig> {
  const response = await apiPut('/api/v1/memory/multimodal/config', config);
  if (!response.ok) throw new Error('Failed to update multimodal config');
  return response.json();
}

export async function getMultimodalStats(): Promise<MultimodalStats> {
  const response = await apiGet('/api/v1/memory/multimodal/stats');
  if (!response.ok) throw new Error('Failed to get multimodal stats');
  return response.json();
}

export async function listMultimodalEmbeddings(modality?: string, limit?: number): Promise<MultimodalEmbedding[]> {
  const params = new URLSearchParams();
  if (modality) params.set('modality', modality);
  if (limit) params.set('limit', limit.toString());
  const query = params.toString();
  const response = await apiGet(`/api/v1/memory/multimodal/embeddings${query ? `?${query}` : ''}`);
  if (!response.ok) throw new Error('Failed to list embeddings');
  return response.json();
}

export async function indexMemory(data: {
  memoryId: string;
  memoryType: string;
  modality: 'image' | 'audio';
  data: string;  // Base64
  mimeType: string;
}): Promise<{ job_id: string; status: string }> {
  const response = await apiPost('/api/v1/memory/multimodal/index', {
    memory_id: data.memoryId,
    memory_type: data.memoryType,
    modality: data.modality,
    data: data.data,
    mime_type: data.mimeType,
  });
  if (!response.ok) throw new Error('Failed to index memory');
  return response.json();
}

export async function searchMultimodal(query: {
  q?: string;
  modality?: string;
  limit?: number;
}): Promise<MultimodalSearchResult[]> {
  const params = new URLSearchParams();
  if (query.q) params.set('q', query.q);
  if (query.modality) params.set('modality', query.modality);
  if (query.limit) params.set('limit', query.limit.toString());
  const queryStr = params.toString();
  const response = await apiGet(`/api/v1/memory/multimodal/search${queryStr ? `?${queryStr}` : ''}`);
  if (!response.ok) throw new Error('Failed to search multimodal memory');
  return response.json();
}

// ============ Extended Channels (NEW - Phase 6) ============

export interface ExtendedChannelSettings {
  id: string;
  channel_type: string;
  channel_id: string;
  settings: Record<string, unknown>;
  is_enabled: boolean;
  created_at: string;
  updated_at: string;
  display_name: string;
  icon: string;
}

export interface ChannelTypeInfo {
  id: string;
  name: string;
  icon: string;
  has_credentials: boolean;
}

export interface ChannelTemplate {
  id: string;
  template_name: string;
  template_content: Record<string, unknown>;
  is_default: boolean;
  created_at: string;
}

export interface ChannelWebhook {
  id: string;
  channel_id: string;
  webhook_url: string;
  events: string[];
  is_enabled: boolean;
  created_at: string;
}

export async function listExtendedChannels(): Promise<ExtendedChannelSettings[]> {
  const response = await apiGet('/api/v1/channels/extended/list');
  if (!response.ok) throw new Error('Failed to list extended channels');
  return response.json();
}

export async function listChannelTypes(): Promise<ChannelTypeInfo[]> {
  const response = await apiGet('/api/v1/channels/extended/types');
  if (!response.ok) throw new Error('Failed to list channel types');
  return response.json();
}

export async function createExtendedChannel(data: {
  channel_type: string;
  channel_id: string;
  settings: Record<string, unknown>;
  credentials?: Record<string, unknown>;
}): Promise<ExtendedChannelSettings> {
  const response = await apiPost('/api/v1/channels/extended', data);
  if (!response.ok) throw new Error('Failed to create channel');
  return response.json();
}

export async function getExtendedChannel(id: string): Promise<ExtendedChannelSettings> {
  const response = await apiGet(`/api/v1/channels/extended/${id}`);
  if (!response.ok) throw new Error('Failed to get channel');
  return response.json();
}

export async function updateExtendedChannel(
  id: string,
  data: {
    settings?: Record<string, unknown>;
    credentials?: Record<string, unknown>;
  }
): Promise<ExtendedChannelSettings> {
  const response = await apiPut(`/api/v1/channels/extended/${id}`, data);
  if (!response.ok) throw new Error('Failed to update channel');
  return response.json();
}

export async function toggleExtendedChannel(id: string, enabled: boolean): Promise<ExtendedChannelSettings> {
  const response = await apiPut(`/api/v1/channels/extended/${id}/toggle`, { enabled });
  if (!response.ok) throw new Error('Failed to toggle channel');
  return response.json();
}

export async function deleteExtendedChannel(id: string): Promise<void> {
  const response = await apiDelete(`/api/v1/channels/extended/${id}`);
  if (!response.ok) throw new Error('Failed to delete channel');
}

// Templates
export async function listChannelTemplates(channelType: string): Promise<ChannelTemplate[]> {
  const response = await apiGet(`/api/v1/channels/extended/templates/${channelType}`);
  if (!response.ok) throw new Error('Failed to list templates');
  return response.json();
}

export async function createChannelTemplate(data: {
  channel_type: string;
  template_name: string;
  template_content: Record<string, unknown>;
}): Promise<{ id: string; created_at: string }> {
  const response = await apiPost('/api/v1/channels/extended/templates', data);
  if (!response.ok) throw new Error('Failed to create template');
  return response.json();
}

export async function deleteChannelTemplate(id: string): Promise<void> {
  const response = await apiDelete(`/api/v1/channels/extended/templates/${id}`);
  if (!response.ok) throw new Error('Failed to delete template');
}

// Webhooks
export async function listChannelWebhooks(channelType: string): Promise<ChannelWebhook[]> {
  const response = await apiGet(`/api/v1/channels/extended/webhooks/${channelType}`);
  if (!response.ok) throw new Error('Failed to list webhooks');
  return response.json();
}

export async function createChannelWebhook(data: {
  channel_type: string;
  channel_id: string;
  webhook_url: string;
  events: string[];
}): Promise<{ id: string; created_at: string }> {
  const response = await apiPost('/api/v1/channels/extended/webhooks', data);
  if (!response.ok) throw new Error('Failed to create webhook');
  return response.json();
}

export async function toggleChannelWebhook(id: string, enabled: boolean): Promise<void> {
  const response = await apiPut(`/api/v1/channels/extended/webhooks/${id}/toggle`, { enabled });
  if (!response.ok) throw new Error('Failed to toggle webhook');
}

export async function deleteChannelWebhook(id: string): Promise<void> {
  const response = await apiDelete(`/api/v1/channels/extended/webhooks/${id}`);
  if (!response.ok) throw new Error('Failed to delete webhook');
}

// ============ Secrets (Phase 7) ============

export interface SecretRef {
  id: string;
  ref_key: string;
  secret_name: string;
  env_var: string | null;
  description: string | null;
  targets: string[];
  category: string;
  category_label: string;
  category_icon: string;
  is_predefined: boolean;
  created_at: string;
  updated_at: string;
}

export interface SecretRotationLog {
  id: string;
  secret_name: string;
  rotated_at: string;
  rotated_by: string | null;
  status: string;
  error_message: string | null;
  old_value_hash: string | null;
  new_value_hash: string | null;
}

export interface SecretAccessLog {
  id: string;
  secret_ref_id: string;
  accessed_at: string;
  access_type: string;
  accessed_by: string | null;
  success: number;
  error_message: string | null;
}

// List all secrets
export async function listSecrets(): Promise<SecretRef[]> {
  const response = await apiGet('/api/v1/secrets');
  if (!response.ok) throw new Error('Failed to list secrets');
  return response.json();
}

// Get secret by ID
export async function getSecret(id: string): Promise<SecretRef> {
  const response = await apiGet(`/api/v1/secrets/${id}`);
  if (!response.ok) throw new Error('Failed to get secret');
  return response.json();
}

// Update secret description
export async function updateSecret(id: string, description?: string): Promise<SecretRef> {
  const response = await apiPut(`/api/v1/secrets/${id}`, { description });
  if (!response.ok) throw new Error('Failed to update secret');
  return response.json();
}

// Delete secret (only custom secrets)
export async function deleteSecret(id: string): Promise<{ deleted: boolean }> {
  const response = await apiDelete(`/api/v1/secrets/${id}`);
  if (!response.ok) throw new Error('Failed to delete secret');
  return response.json();
}

// List categories
export async function listSecretCategories(): Promise<string[]> {
  const response = await apiGet('/api/v1/secrets/categories');
  if (!response.ok) throw new Error('Failed to list categories');
  return response.json();
}

// Get secrets by category
export async function getSecretsByCategory(category: string): Promise<SecretRef[]> {
  const response = await apiGet(`/api/v1/secrets/category/${category}`);
  if (!response.ok) throw new Error('Failed to get secrets by category');
  return response.json();
}

// Get rotation history
export async function getSecretRotationHistory(secretName: string, limit = 50): Promise<SecretRotationLog[]> {
  const response = await apiGet(`/api/v1/secrets/rotation/${secretName}?limit=${limit}`);
  if (!response.ok) throw new Error('Failed to get rotation history');
  return response.json();
}

// Get recent rotations
export async function getRecentSecretRotations(limit = 20): Promise<SecretRotationLog[]> {
  const response = await apiGet(`/api/v1/secrets/rotation/recent?limit=${limit}`);
  if (!response.ok) throw new Error('Failed to get recent rotations');
  return response.json();
}

// Get access history
export async function getSecretAccessHistory(secretRefId: string, limit = 50): Promise<SecretAccessLog[]> {
  const response = await apiGet(`/api/v1/secrets/access/${secretRefId}?limit=${limit}`);
  if (!response.ok) throw new Error('Failed to get access history');
  return response.json();
}

// Get recent accesses
export async function getRecentSecretAccesses(limit = 20): Promise<SecretAccessLog[]> {
  const response = await apiGet(`/api/v1/secrets/access/recent?limit=${limit}`);
  if (!response.ok) throw new Error('Failed to get recent accesses');
  return response.json();
}

// Get failed accesses
export async function getFailedSecretAccesses(limit = 20): Promise<SecretAccessLog[]> {
  const response = await apiGet(`/api/v1/secrets/access/failed?limit=${limit}`);
  if (!response.ok) throw new Error('Failed to get failed accesses');
  return response.json();
}

// List predefined secret IDs
export async function listPredefinedSecrets(): Promise<string[]> {
  const response = await apiGet('/api/v1/secrets/predefined');
  if (!response.ok) throw new Error('Failed to list predefined secrets');
  return response.json();
}

// ============ Slack Block Templates (Phase 8) ============

export interface SlackBlockTemplate {
  id: string;
  name: string;
  template: string;
  description: string | null;
  created_at: string;
}

// List all Slack templates
export async function listSlackTemplates(): Promise<SlackBlockTemplate[]> {
  const response = await apiGet('/api/v1/slack/templates');
  if (!response.ok) throw new Error('Failed to list templates');
  return response.json();
}

// Get template by ID
export async function getSlackTemplate(id: string): Promise<SlackBlockTemplate> {
  const response = await apiGet(`/api/v1/slack/templates/${id}`);
  if (!response.ok) throw new Error('Failed to get template');
  return response.json();
}

// Create template
export async function createSlackTemplate(data: {
  name: string;
  template: string;
  description?: string;
}): Promise<SlackBlockTemplate> {
  const response = await apiPost('/api/v1/slack/templates', data);
  if (!response.ok) throw new Error('Failed to create template');
  return response.json();
}

// Update template
export async function updateSlackTemplate(id: string, data: {
  name?: string;
  template?: string;
  description?: string;
}): Promise<SlackBlockTemplate> {
  const response = await apiPut(`/api/v1/slack/templates/${id}`, data);
  if (!response.ok) throw new Error('Failed to update template');
  return response.json();
}

// Delete template
export async function deleteSlackTemplate(id: string): Promise<{ deleted: boolean }> {
  const response = await apiDelete(`/api/v1/slack/templates/${id}`);
  if (!response.ok) throw new Error('Failed to delete template');
  return response.json();
}

// Render template with variables
export async function renderSlackTemplate(id: string, variables: Record<string, unknown>): Promise<{ rendered: string }> {
  const response = await apiPost(`/api/v1/slack/templates/${id}/render`, { variables });
  if (!response.ok) throw new Error('Failed to render template');
  return response.json();
}

// ============ Execution Patterns / Death Spiral Detection (Phase 9) ============

export interface ExecutionPattern {
  id: string;
  task_id: string;
  pattern_type: string;
  severity: string;
  tool_calls: string[] | null;
  file_ops: string[] | null;
  error_count: number;
  details: Record<string, unknown> | null;
  detected_at: string;
}

export interface PatternAlertTemplate {
  id: string;
  pattern_type: string;
  title: string;
  description: string | null;
  severity: string;
  remediation: string | null;
  created_at: string;
}

export interface PatternStats {
  by_severity: { severity: string; count: number }[];
  by_type: { pattern_type: string; count: number }[];
  total: number;
}

// List recent patterns
export async function listExecutionPatterns(limit = 50): Promise<ExecutionPattern[]> {
  const response = await apiGet(`/api/v1/patterns?limit=${limit}`);
  if (!response.ok) throw new Error('Failed to list patterns');
  return response.json();
}

// Get patterns by task
export async function getPatternsByTask(taskId: string): Promise<ExecutionPattern[]> {
  const response = await apiGet(`/api/v1/patterns/task/${taskId}`);
  if (!response.ok) throw new Error('Failed to get patterns');
  return response.json();
}

// Get patterns by type
export async function getPatternsByType(patternType: string): Promise<ExecutionPattern[]> {
  const response = await apiGet(`/api/v1/patterns/type/${patternType}`);
  if (!response.ok) throw new Error('Failed to get patterns');
  return response.json();
}

// Get patterns by severity
export async function getPatternsBySeverity(severity: string): Promise<ExecutionPattern[]> {
  const response = await apiGet(`/api/v1/patterns/severity/${severity}`);
  if (!response.ok) throw new Error('Failed to get patterns');
  return response.json();
}

// Get pattern statistics
export async function getPatternStats(): Promise<PatternStats> {
  const response = await apiGet('/api/v1/patterns/stats');
  if (!response.ok) throw new Error('Failed to get pattern stats');
  return response.json();
}

// Delete patterns by task
export async function deletePatternsByTask(taskId: string): Promise<{ deleted: boolean }> {
  const response = await apiDelete(`/api/v1/patterns/task/${taskId}`);
  if (!response.ok) throw new Error('Failed to delete patterns');
  return response.json();
}

// List pattern alert templates
export async function listPatternTemplates(): Promise<PatternAlertTemplate[]> {
  const response = await apiGet('/api/v1/patterns/templates');
  if (!response.ok) throw new Error('Failed to list templates');
  return response.json();
}

// Get template by pattern type
export async function getPatternTemplate(patternType: string): Promise<PatternAlertTemplate> {
  const response = await apiGet(`/api/v1/patterns/templates/${patternType}`);
  if (!response.ok) throw new Error('Failed to get template');
  return response.json();
}

export { API_BASE };
