/**
 * APEX SSE Streaming Client
 *
 * Patch 11: EventSource-based SSE streaming client for consuming Hands and MCP
 * task events from the APEX router.
 *
 * Security posture:
 * - Streams are authenticated using a signed URL query parameter (MVP approach)
 *   because native EventSource cannot set custom headers.
 * - The HMAC signature is computed from: timestamp + "GET" + path + ""
 * - The router's SSE endpoint verifies the signature before establishing the stream.
 *
 * MVP scope: EventSource (SSE) only. WebSocket upgrade path in Patch 12+.
 *
 * Usage:
 * ```typescript
 * import { initSseStream } from './streaming';
 *
 * const client = initSseStream({
 *   path: '/api/v1/stream/hands/task-123',
 *   onConnected: () => console.log('Stream connected'),
 *   onThought: (data) => console.log('Thought:', data),
 *   onToolCall: (data) => console.log('Tool:', data),
 *   onToolResult: (data) => console.log('Result:', data),
 *   onComplete: (data) => console.log('Done:', data),
 *   onError: (err) => console.error('Stream error:', err),
 * });
 *
 * // Cleanup
 * client.close();
 * ```
 */

import { createHmac, randomUUID } from 'crypto';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** SSE event types emitted by the streaming endpoints. */
export type SseEventType =
  | 'connected'
  | 'thought'
  | 'tool_call'
  | 'tool_progress'
  | 'tool_result'
  | 'approval_needed'
  | 'error'
  | 'complete'
  | 'stream_closed';

/** Raw SSE event payload — all events include a `type` field. */
export interface SseEventPayload {
  type: SseEventType;
  [key: string]: unknown;
}

/** Parsed SSE event with raw EventSource data. */
export interface ParsedSseEvent {
  eventType: SseEventType;
  data: SseEventPayload;
  rawEvent: MessageEvent;
}

/** Handlers for each SSE event type. */
export interface SseStreamHandlers {
  /** Called when the SSE connection is established. */
  onConnected?: (data: { task_id: string; timestamp: string }) => void;
  /** Called for each thought event. */
  onThought?: (data: { step: number; content: string }) => void;
  /** Called for each tool call event. */
  onToolCall?: (data: { step: number; tool: string; input: Record<string, unknown> }) => void;
  /** Called for each tool progress event. */
  onToolProgress?: (data: { step: number; tool: string; output: string }) => void;
  /** Called for each tool result event. */
  onToolResult?: (data: { step: number; tool: string; success: boolean; output: string }) => void;
  /** Called when approval is needed for a T2/T3 action. */
  onApprovalNeeded?: (data: { step: number; tier: string; action: string; consequences: unknown }) => void;
  /** Called for errors. */
  onError?: (data: { message: string }) => void;
  /** Called when the stream completes successfully. */
  onComplete?: (data: { output: string; steps: number; tools_used: string[] }) => void;
  /** Called when the stream is closed by the server. */
  onStreamClosed?: (data: { task_id: string }) => void;
  /** Catch-all for any unhandled event types. */
  onMessage?: (event: ParsedSseEvent) => void;
}

/** Configuration for initSseStream. */
export interface SseStreamConfig extends SseStreamHandlers {
  /** SSE endpoint path (e.g. '/api/v1/stream/hands/task-123'). */
  path: string;
  /** APEX router base URL. Defaults to 'http://localhost:3000'. */
  baseUrl?: string;
  /** HMAC shared secret. Defaults to APEX_SHARED_SECRET env var. */
  sharedSecret?: string;
  /** Reconnect delay in ms. Defaults to 3000. */
  reconnectDelayMs?: number;
  /** Max reconnect attempts. Defaults to 5. -1 for infinite. */
  maxReconnects?: number;
  /** Called on connection open. */
  onOpen?: () => void;
  /** Called on connection close. */
  onClose?: () => void;
  /** Called when the underlying EventSource errors. */
  onSseError?: (err: Event) => void;
}

// ---------------------------------------------------------------------------
// HMAC URL signing (MVP)
// ---------------------------------------------------------------------------

/**
 * Build a signed SSE URL with HMAC query parameters.
 *
 * MVP approach: because native EventSource cannot set custom headers,
 * the HMAC signature is passed as query params. The router SSE endpoint
 * verifies the signature before establishing the stream.
 *
 * Production upgrade: use @microsoft/fetch-event-source to get header control,
 * or switch to WebSocket with an auth handshake message.
 */
function buildSignedUrl(config: SseStreamConfig): string {
  const base = config.baseUrl ?? 'http://localhost:3000';
  const secret = config.sharedSecret ?? process.env.APEX_SHARED_SECRET ?? 'dev-secret-change-in-production';
  const timestamp = Math.floor(Date.now() / 1000).toString();
  const nonce = randomUUID();

  // HMAC-SHA256(timestamp + "GET" + path + "")
  const payload = `${timestamp}GET${config.path}`;
  const signature = createHmac('sha256', secret).update(payload).digest('hex');

  const url = new URL(`${base}${config.path}`);
  url.searchParams.set('__timestamp', timestamp);
  url.searchParams.set('__nonce', nonce);
  url.searchParams.set('__signature', signature);

  return url.toString();
}

// ---------------------------------------------------------------------------
// SSE stream parser
// ---------------------------------------------------------------------------

/** Parse a raw MessageEvent from EventSource into a typed SseEventPayload. */
function parseSseEvent(raw: MessageEvent): SseEventPayload {
  try {
    return JSON.parse(raw.data) as SseEventPayload;
  } catch {
    // If the data is not JSON (e.g., a comment), return a minimal object
    return { type: 'unknown' as SseEventType, raw: raw.data };
  }
}

/** Map an SSE event type string to the handler key. */
function toHandlerKey(eventType: SseEventType): keyof SseStreamHandlers | null {
  const map: Record<SseEventType, keyof SseStreamHandlers | null> = {
    connected: 'onConnected',
    thought: 'onThought',
    tool_call: 'onToolCall',
    tool_progress: 'onToolProgress',
    tool_result: 'onToolResult',
    approval_needed: 'onApprovalNeeded',
    error: 'onError',
    complete: 'onComplete',
    stream_closed: 'onStreamClosed',
  };
  return map[eventType] ?? 'onMessage';
}

// ---------------------------------------------------------------------------
// EventSource lifecycle
// ---------------------------------------------------------------------------

/** Holds the active EventSource and reconnect state. */
export interface SseStreamClient {
  /** The underlying EventSource instance. */
  source: EventSource;
  /** Close the stream and stop reconnecting. */
  close: () => void;
  /** Returns true if the stream is currently open. */
  isOpen: () => boolean;
}

/** Initialize an SSE stream with the given configuration. */
export function initSseStream(config: SseStreamConfig): SseStreamClient {
  const url = buildSignedUrl(config);
  const reconnectDelay = config.reconnectDelayMs ?? 3000;
  const maxReconnects = config.maxReconnects ?? 5;
  let reconnectCount = 0;
  let closed = false;

  // Create the EventSource
  const source = new EventSource(url);

  // Map event type names (as used in SSE .event()) to handler names
  const HANDLER_MAP: Record<string, keyof SseStreamHandlers> = {
    connected: 'onConnected',
    thought: 'onThought',
    tool_call: 'onToolCall',
    tool_progress: 'onToolProgress',
    tool_result: 'onToolResult',
    approval_needed: 'onApprovalNeeded',
    error: 'onError',
    complete: 'onComplete',
    stream_closed: 'onStreamClosed',
  };

  // Attach event listeners for each known event type
  for (const [eventType, handlerKey] of Object.entries(HANDLER_MAP)) {
    const handler = config[handlerKey];
    if (handler && handlerKey !== 'onMessage') {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      source.addEventListener(eventType, (raw: any) => {
        const parsed = parseSseEvent(raw as MessageEvent);
        // Double-cast through unknown to satisfy TypeScript's strict union analysis
        const typedHandler = handler as unknown as (data: SseEventPayload) => void;
        typedHandler(parsed);
      });
    }
  }

  // Catch-all for any unhandled event types
  source.addEventListener('message', (raw: MessageEvent) => {
    if (config.onMessage) {
      const eventType = parseSseEvent(raw).type as SseEventType;
      config.onMessage({ eventType, data: parseSseEvent(raw), rawEvent: raw });
    }
  });

  // Connection lifecycle
  source.onopen = () => {
    reconnectCount = 0;
    config.onOpen?.();
  };

  source.onerror = (err: Event) => {
    config.onSseError?.(err);

    if (closed) return;

    // Attempt reconnect if within limits
    if (maxReconnects === -1 || reconnectCount < maxReconnects) {
      reconnectCount++;
      console.warn(`[SSE] Connection lost, reconnecting in ${reconnectDelay}ms (attempt ${reconnectCount})...`);
      setTimeout(() => {
        if (!closed) {
          // Re-create the EventSource with a fresh signed URL (signature is time-bound)
          source.close();
          initSseStream({ ...config, maxReconnects, reconnectDelayMs: reconnectDelay });
        }
      }, reconnectDelay);
    } else {
      console.error(`[SSE] Max reconnect attempts (${maxReconnects}) reached. Giving up.`);
      config.onSseError?.(err);
    }
  };

  // Expose close method
  const close = () => {
    closed = true;
    source.close();
    config.onClose?.();
  };

  return {
    source,
    close,
    isOpen: () => source.readyState === EventSource.OPEN,
  };
}

// ---------------------------------------------------------------------------
// Convenience helpers
// ---------------------------------------------------------------------------

/**
 * Subscribe to a Hands task stream.
 * Shorthand for `initSseStream({ path: `/api/v1/stream/hands/${taskId}`, ...handlers })`.
 */
export function streamHandsTask(
  taskId: string,
  handlers: SseStreamHandlers,
  options?: Partial<Omit<SseStreamConfig, 'path' | keyof SseStreamHandlers>>
): SseStreamClient {
  return initSseStream({ path: `/api/v1/stream/hands/${taskId}`, ...handlers, ...options });
}

/**
 * Subscribe to an MCP task stream.
 * Shorthand for `initSseStream({ path: `/api/v1/stream/mcp/${taskId}`, ...handlers })`.
 */
export function streamMcpTask(
  taskId: string,
  handlers: SseStreamHandlers,
  options?: Partial<Omit<SseStreamConfig, 'path' | keyof SseStreamHandlers>>
): SseStreamClient {
  return initSseStream({ path: `/api/v1/stream/mcp/${taskId}`, ...handlers, ...options });
}

/**
 * Subscribe to a generic task stream.
 * Shorthand for `initSseStream({ path: `/api/v1/stream/task/${taskId}`, ...handlers })`.
 */
export function streamTask(
  taskId: string,
  handlers: SseStreamHandlers,
  options?: Partial<Omit<SseStreamConfig, 'path' | keyof SseStreamHandlers>>
): SseStreamClient {
  return initSseStream({ path: `/api/v1/stream/task/${taskId}`, ...handlers, ...options });
}
