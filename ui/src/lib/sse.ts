/**
 * ui/src/lib/sse.ts — SSE client with HMAC signing for APEX streaming endpoints.
 *
 * Connects to /api/v1/stream/task/{taskId} (also /hands/ and /mcp/ variants).
 * Uses query-param auth (HMAC-SHA256) because native EventSource cannot set headers.
 *
 * Events dispatched to Zustand store:
 *   Thought      → store.addExecutionStep(..., 'Thought')
 *   ToolCall     → store.addExecutionStep(..., 'ToolCall')
 *   ToolProgress → store.addExecutionStep(..., 'ToolProgress')
 *   ToolResult   → store.addExecutionStep(..., 'ToolResult')
 *   ApprovalNeeded → store.addExecutionStep(..., 'ApprovalNeeded') + store.setPendingConfirmation()
 *   Error        → store.updateTask(..., 'failed') + store.addExecutionStep(..., 'Error')
 *   Complete     → store.updateTask(..., 'completed') + store.addExecutionStep(..., 'Complete')
 */

import { useAppStore } from '../stores/appStore';

// Re-export event types so consumers can import them
export type {
  ExecutionStep,
} from '../stores/appStore';

const API_BASE = 'http://localhost:3000';
const SHARED_SECRET = 'dev-secret-change-in-production';

/** HMAC-SHA256 signature for a streaming request.
 *
 * Format: HMAC-SHA256(secret, "timestamp|GET|path|body")
 * Backend verify_request() uses the same format.
 */
async function computeStreamSignature(
  timestamp: number,
  method: string,
  path: string,
  body: string = ''
): Promise<string> {
  const encoder = new TextEncoder();
  const keyData = encoder.encode(SHARED_SECRET);
  const key = await globalThis.crypto.subtle.importKey(
    'raw',
    keyData,
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign']
  );
  // Format: timestamp|method|path|body (pipe-separated, same as backend)
  const message = `${timestamp}${method}${path}${body}`;
  const signatureBuffer = await globalThis.crypto.subtle.sign('HMAC', key, encoder.encode(message));
  return Array.from(new Uint8Array(signatureBuffer))
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
}

// ---------------------------------------------------------------------------
// SSE Event type (matches what backend sends)
// ---------------------------------------------------------------------------

export interface SseExecutionEvent {
  type: 'Thought' | 'ToolCall' | 'ToolProgress' | 'ToolResult' | 'ApprovalNeeded' | 'Error' | 'Complete';
  task_id: string;
  step: number;
  content?: string;
  tool?: string;
  input?: Record<string, unknown>;
  output?: string;
  success?: boolean;
  tier?: string;
  action?: string;
  message?: string;
  tools_used?: string[];
  consequences?: {
    files_read: string[];
    files_written: string[];
    commands_executed: string[];
    blast_radius: 'minimal' | 'limited' | 'extensive';
    summary: string;
  };
}

export interface SseMetaEvent {
  type: 'connected' | 'heartbeat' | 'error';
  message?: string;
}

export type SseEvent = SseExecutionEvent | SseMetaEvent;

// ---------------------------------------------------------------------------
// SSEClient class
// ---------------------------------------------------------------------------

export interface SSEClientOptions {
  /** URL path segment: "task", "hands", or "mcp" */
  pathSegment?: 'task' | 'hands' | 'mcp';
  /** Auto-reconnect on disconnect (default: true) */
  autoReconnect?: boolean;
  /** Max reconnect attempts (default: 5) */
  maxRetries?: number;
  /** Initial reconnect delay in ms (default: 1000) */
  reconnectDelay?: number;
  /** Called when the stream closes cleanly (complete or failed) */
  onDone?: (taskId: string, reason: 'complete' | 'failed' | 'error' | 'cancelled') => void;
  /** Called on any error */
  onError?: (taskId: string, error: string) => void;
  /** Called when connected */
  onConnect?: (taskId: string) => void;
}

export class SSEClient {
  private eventSource: EventSource | null = null;
  private taskId: string;
  private options: Required<SSEClientOptions>;
  private reconnectAttempts = 0;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private closed = false;

  constructor(taskId: string, options: SSEClientOptions = {}) {
    this.taskId = taskId;
    this.options = {
      pathSegment: options.pathSegment ?? 'task',
      autoReconnect: options.autoReconnect ?? true,
      maxRetries: options.maxRetries ?? 5,
      reconnectDelay: options.reconnectDelay ?? 1000,
      onDone: options.onDone ?? (() => {}),
      onError: options.onError ?? (() => {}),
      onConnect: options.onConnect ?? (() => {}),
    };
  }

  /** Build the SSE URL with HMAC auth query params. */
  private async buildSignedUrl(): Promise<string> {
    const path = `/api/v1/stream/${this.options.pathSegment}/${this.taskId}`;
    const timestamp = Math.floor(Date.now() / 1000);
    const signature = await computeStreamSignature(timestamp, 'GET', path);
    // nonce is optional but recommended for replay protection
    const nonce = crypto.randomUUID?.() ?? `${timestamp}-${Math.random().toString(36).slice(2)}`;
    return `${API_BASE}${path}?__timestamp=${timestamp}&__signature=${signature}&__nonce=${nonce}`;
  }

  /** Start the SSE connection. */
  async connect(): Promise<void> {
    if (this.closed) return;
    const url = await this.buildSignedUrl();

    try {
      this.eventSource = new EventSource(url);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      this.options.onError(this.taskId, `Failed to create EventSource: ${msg}`);
      this.scheduleReconnect();
      return;
    }

    this.eventSource.onopen = () => {
      this.reconnectAttempts = 0;
      this.options.onConnect(this.taskId);
    };

    // Wire up SSE event handlers
    this.wireEvents();

    this.eventSource.onerror = () => {
      if (this.closed) return;
      // EventSource auto-reconnects on error; we just schedule fallback if needed
      if (this.eventSource?.readyState === EventSource.CLOSED) {
        this.scheduleReconnect();
      }
    };
  }

  /** Wire all SSE event type handlers. */
  private wireEvents(): void {
    if (!this.eventSource) return;

    const store = useAppStore.getState();

    // Execution events
    const eventTypes: SseExecutionEvent['type'][] = [
      'Thought', 'ToolCall', 'ToolProgress', 'ToolResult',
      'ApprovalNeeded', 'Error', 'Complete',
    ];

    for (const type of eventTypes) {
      this.eventSource.addEventListener(type, (e: MessageEvent) => {
        try {
          const event = JSON.parse(e.data) as SseExecutionEvent;
          this.dispatchEvent(event, store);
        } catch {
          console.warn(`[SSE] Failed to parse ${type} event:`, e.data);
        }
      });
    }

    // Generic message handler as fallback
    this.eventSource.onmessage = (e: MessageEvent) => {
      try {
        const raw = JSON.parse(e.data) as SseEvent;
        if (raw.type === 'connected' || raw.type === 'heartbeat') {
          // connection confirmed, no-op
        } else if (raw.type === 'error') {
          this.options.onError(this.taskId, raw.message ?? 'Unknown SSE error');
        } else {
          this.dispatchEvent(raw as SseExecutionEvent, store);
        }
      } catch {
        // ignore non-JSON messages (e.g., keepalive from SSE)
      }
    };
  }

  /** Dispatch an SSE event to the Zustand store. */
  private dispatchEvent(event: SseExecutionEvent, store: typeof useAppStore.prototype): void {
    switch (event.type) {
      case 'Thought':
        store.addExecutionStep(this.taskId, {
          type: 'Thought',
          step: event.step || 0,
          content: event.content || '',
        });
        break;

      case 'ToolCall':
        store.addExecutionStep(this.taskId, {
          type: 'ToolCall',
          step: event.step || 0,
          tool: event.tool || '',
          input: event.input || {},
        });
        break;

      case 'ToolProgress':
        store.addExecutionStep(this.taskId, {
          type: 'ToolProgress',
          step: event.step || 0,
          tool: event.tool || '',
          content: event.content,
          output: event.output,
        });
        break;

      case 'ToolResult':
        store.addExecutionStep(this.taskId, {
          type: 'ToolResult',
          step: event.step || 0,
          tool: event.tool || '',
          success: event.success ?? true,
          output: event.output || '',
        });
        break;

      case 'ApprovalNeeded':
        store.addExecutionStep(this.taskId, {
          type: 'ApprovalNeeded',
          step: event.step || 0,
          content: event.message || event.action || '',
          output: event.tier,
        });
        if (event.tier && event.action) {
          store.setPendingConfirmation({
            taskId: this.taskId,
            tier: event.tier as 'T1' | 'T2' | 'T3',
            action: event.action,
            skillName: event.tool,
            consequences: event.consequences,
          });
        }
        break;

      case 'Error':
        store.updateTask(this.taskId, { status: 'failed', error: event.message });
        store.addExecutionStep(this.taskId, {
          type: 'Error',
          step: event.step || 0,
          content: event.message || 'Execution error',
        });
        this.options.onDone(this.taskId, 'error');
        break;

      case 'Complete':
        store.updateTask(this.taskId, { status: 'completed', output: event.output });
        store.addExecutionStep(this.taskId, {
          type: 'Complete',
          step: event.step || 0,
          content: event.content,
          output: event.output,
        });
        this.options.onDone(this.taskId, 'complete');
        break;
    }
  }

  /** Schedule a reconnect attempt with exponential backoff. */
  private scheduleReconnect(): void {
    if (this.closed) return;
    if (this.reconnectAttempts >= this.options.maxRetries) {
      this.options.onError(this.taskId, 'Max reconnect attempts reached');
      this.options.onDone(this.taskId, 'error');
      return;
    }
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
    const delay = this.options.reconnectDelay * Math.pow(2, this.reconnectAttempts);
    this.reconnectAttempts++;
    this.reconnectTimer = setTimeout(() => this.connect(), delay);
  }

  /** Close the SSE connection. */
  close(reason: 'complete' | 'failed' | 'cancelled' = 'cancelled'): void {
    this.closed = true;
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    if (this.eventSource) {
      this.eventSource.close();
      this.eventSource = null;
    }
    this.options.onDone(this.taskId, reason);
  }

  /** Check if the connection is currently open. */
  get isConnected(): boolean {
    return this.eventSource?.readyState === EventSource.OPEN;
  }

  get isClosed(): boolean {
    return this.closed;
  }
}

// ---------------------------------------------------------------------------
// React hook for SSE streaming
// ---------------------------------------------------------------------------

import { useEffect, useRef, useCallback, useState } from 'react';

export interface UseTaskStreamOptions extends Omit<SSEClientOptions, 'pathSegment'> {
  /** Auto-connect when taskId is set (default: true) */
  autoConnect?: boolean;
  /** Path segment: "task", "hands", or "mcp" (default: "task") */
  pathSegment?: 'task' | 'hands' | 'mcp';
}

export interface UseTaskStreamReturn {
  /** The SSEClient instance (null before connect) */
  client: SSEClient | null;
  /** Whether we are currently connected */
  isConnected: boolean;
  /** Whether the stream has closed (complete/failed/error/cancelled) */
  isDone: boolean;
  /** Error message if any */
  error: string | null;
  /** Manually connect to the stream */
  connect: () => void;
  /** Manually disconnect from the stream */
  disconnect: () => void;
}

/**
 * useTaskStream — React hook for SSE streaming of a task.
 *
 * Usage:
 *   const { client, isConnected, error, disconnect } = useTaskStream(taskId);
 *
 * The hook automatically dispatches incoming events to the Zustand store.
 * Pass `autoConnect={false}` to manually control connection.
 */
export function useTaskStream(
  taskId: string | null,
  options: UseTaskStreamOptions = {}
): UseTaskStreamReturn {
  const {
    autoConnect = true,
    pathSegment = 'task',
    onDone,
    onError,
    onConnect,
    autoReconnect,
    maxRetries,
    reconnectDelay,
  } = options;

  const clientRef = useRef<SSEClient | null>(null);
  const [isDone, setIsDone] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const connect = useCallback(() => {
    if (!taskId || isDone) return;
    if (clientRef.current) {
      clientRef.current.close('cancelled');
    }

    const client = new SSEClient(taskId, {
      autoReconnect: autoReconnect ?? true,
      maxRetries: maxRetries ?? 5,
      reconnectDelay: reconnectDelay ?? 1000,
      pathSegment,
      onDone: (_tid, reason) => {
        setIsDone(true);
        onDone?.(_tid, reason);
      },
      onError: (_tid, err) => {
        setError(err);
        onError?.(_tid, err);
      },
      onConnect: (tid) => {
        onConnect?.(tid);
      },
    });

    clientRef.current = client;
    client.connect();
  }, [taskId, isDone, pathSegment, autoReconnect, maxRetries, reconnectDelay, onDone, onError, onConnect]);

  const disconnect = useCallback(() => {
    clientRef.current?.close('cancelled');
    clientRef.current = null;
  }, []);

  // Auto-connect when taskId is set
  useEffect(() => {
    if (autoConnect && taskId && !isDone) {
      connect();
    }
    return () => {
      clientRef.current?.close('cancelled');
    };
  }, [autoConnect, taskId, isDone, connect]);

  return {
    client: clientRef.current,
    isConnected: clientRef.current?.isConnected ?? false,
    isDone,
    error,
    connect,
    disconnect,
  };
}

// ---------------------------------------------------------------------------
// Convenience: stream a task and return a promise that resolves on completion
// ---------------------------------------------------------------------------

export interface StreamTaskOptions extends SSEClientOptions {
  /** Timeout in ms (default: no timeout) */
  timeoutMs?: number;
}

/**
 * streamTask — Connect to a task SSE stream and resolve when complete/failed.
 *
 * Usage:
 *   const result = await streamTask('task-123', { timeoutMs: 60_000 });
 *   // result.status: 'completed' | 'failed' | 'timeout'
 */
export function streamTask(
  taskId: string,
  options: StreamTaskOptions = {}
): Promise<{ status: 'completed' | 'failed' | 'timeout' | 'cancelled'; error?: string }> {
  return new Promise((resolve) => {
    const client = new SSEClient(taskId, {
      ...options,
      onDone: (_tid, reason) => {
        resolve({
          status: reason === 'complete' ? 'completed' :
                  reason === 'failed' ? 'failed' :
                  reason === 'cancelled' ? 'cancelled' : 'failed',
        });
      },
      onError: (_tid, error) => {
        resolve({ status: 'failed', error });
      },
    });

    client.connect();

    if (options.timeoutMs) {
      setTimeout(() => {
        if (!client.isClosed) {
          client.close('cancelled');
          resolve({ status: 'timeout' });
        }
      }, options.timeoutMs);
    }
  });
}
