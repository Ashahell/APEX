/**
 * ui/src/lib/ws.ts — WebSocket client with ticket-based auth for APEX streaming.
 *
 * Patch 14: Replaces SSE-based streaming with proper WebSocket + header auth.
 *
 * Flow:
 *   1. Fetch a short-lived ticket: GET /api/v1/stream/ticket?task_id=X
 *   2. Connect to WebSocket: WS /api/v1/stream/ws/:task_id?ticket=...
 *   3. Receive JSON ExecutionEvent messages, dispatch to Zustand store
 *
 * Events dispatched to Zustand store (same as SSEClient):
 *   Thought       → store.addExecutionStep(..., 'Thought')
 *   ToolCall      → store.addExecutionStep(..., 'ToolCall')
 *   ToolProgress  → store.addExecutionStep(..., 'ToolProgress')
 *   ToolResult    → store.addExecutionStep(..., 'ToolResult')
 *   ApprovalNeeded → store.addExecutionStep(..., 'ApprovalNeeded') + store.setPendingConfirmation()
 *   Error         → store.updateTask(..., 'failed') + store.addExecutionStep(..., 'Error')
 *   Complete      → store.updateTask(..., 'completed') + store.addExecutionStep(..., 'Complete')
 */

import { useAppStore } from '../stores/appStore';
import { apiGet } from './api';

const WS_BASE = 'ws://localhost:3000';

// ---------------------------------------------------------------------------
// WebSocket event types (matches backend execution_event_to_json)
// ---------------------------------------------------------------------------

export interface WsExecutionEvent {
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

export interface WsMetaEvent {
  type: 'connected' | 'error' | 'stream_closed' | 'pong';
  message?: string;
}

export type WsEvent = WsExecutionEvent | WsMetaEvent;

// ---------------------------------------------------------------------------
// WSClient class
// ---------------------------------------------------------------------------

export interface WSClientOptions {
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
  /** Ping interval in ms (default: 25000) */
  pingInterval?: number;
}

export class WSClient {
  private ws: WebSocket | null = null;
  private taskId: string;
  private options: Required<WSClientOptions>;
  private reconnectAttempts = 0;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private pingTimer: ReturnType<typeof setInterval> | null = null;
  private closed = false;
  private pingIntervalMs: number;

  constructor(taskId: string, options: WSClientOptions = {}) {
    this.taskId = taskId;
    this.pingIntervalMs = options.pingInterval ?? 25_000;
    this.options = {
      autoReconnect: options.autoReconnect ?? true,
      maxRetries: options.maxRetries ?? 5,
      reconnectDelay: options.reconnectDelay ?? 1000,
      onDone: options.onDone ?? (() => {}),
      onError: options.onError ?? (() => {}),
      onConnect: options.onConnect ?? (() => {}),
      pingInterval: this.pingIntervalMs,
    };
  }

  /** Fetch a signed ticket from the REST endpoint, then connect WebSocket. */
  async connect(): Promise<void> {
    if (this.closed) return;

    try {
      // Step 1: Get a ticket from the REST endpoint (HMAC-authenticated)
      const ticketRes = await apiGet(`/api/v1/stream/ticket?task_id=${encodeURIComponent(this.taskId)}`);
      if (!ticketRes.ok) {
        const text = await ticketRes.text();
        throw new Error(`Ticket request failed (${ticketRes.status}): ${text}`);
      }

      const ticket = await ticketRes.json() as {
        task_id: string;
        expires_at: number;
        nonce: string;
        signature: string;
      };

      // Step 2: Connect WebSocket with the ticket as a query param
      const wsUrl = `${WS_BASE}/api/v1/stream/ws/${encodeURIComponent(this.taskId)}?ticket=${encodeURIComponent(JSON.stringify(ticket))}`;
      this.ws = new WebSocket(wsUrl);

      this.ws.onopen = () => {
        this.reconnectAttempts = 0;
        this.options.onConnect(this.taskId);
        // Start heartbeat ping
        this.startPing();
      };

      this.ws.onmessage = (event: MessageEvent) => {
        try {
          const data = JSON.parse(event.data as string) as WsEvent;
          this.handleMessage(data);
        } catch {
          console.warn('[WS] Failed to parse message:', event.data);
        }
      };

      this.ws.onerror = () => {
        if (this.closed) return;
        this.options.onError(this.taskId, 'WebSocket connection error');
        this.scheduleReconnect();
      };

      this.ws.onclose = (event: CloseEvent) => {
        if (this.closed) return;
        this.stopPing();
        if (!event.wasClean && this.options.autoReconnect) {
          this.scheduleReconnect();
        } else if (event.wasClean) {
          // Clean close — resolve the stream
          this.options.onDone(this.taskId, 'cancelled');
        }
      };
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      this.options.onError(this.taskId, `WS connect failed: ${msg}`);
      this.scheduleReconnect();
    }
  }

  /** Handle an incoming WebSocket message. */
  private handleMessage(event: WsEvent): void {
    switch (event.type) {
      case 'connected':
        // Handshake confirmed
        break;

      case 'pong':
        // Heartbeat response — no-op
        break;

      case 'error':
        this.options.onError(this.taskId, event.message ?? 'Unknown WS error');
        break;

      case 'stream_closed':
        this.options.onDone(this.taskId, 'complete');
        break;

      case 'Thought':
      case 'ToolCall':
      case 'ToolProgress':
      case 'ToolResult':
      case 'ApprovalNeeded':
      case 'Error':
      case 'Complete':
        this.dispatchEvent(event as WsExecutionEvent);
        break;

      default:
        console.warn('[WS] Unknown event type:', (event as WsEvent).type);
    }
  }

  /** Dispatch a WsExecutionEvent to the Zustand store. */
  private dispatchEvent(event: WsExecutionEvent): void {
    const store = useAppStore.getState();
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

  /** Send a ping frame for heartbeat. */
  private startPing(): void {
    this.stopPing();
    this.pingTimer = setInterval(() => {
      if (this.ws?.readyState === WebSocket.OPEN) {
        this.ws.send(JSON.stringify({ type: 'ping' }));
      }
    }, this.pingIntervalMs);
  }

  private stopPing(): void {
    if (this.pingTimer !== null) {
      clearInterval(this.pingTimer);
      this.pingTimer = null;
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

  /** Close the WebSocket connection. */
  close(reason: 'complete' | 'failed' | 'cancelled' = 'cancelled'): void {
    this.closed = true;
    this.stopPing();
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    if (this.ws) {
      this.ws.close(1000, reason);
      this.ws = null;
    }
    this.options.onDone(this.taskId, reason);
  }

  /** Check if the connection is currently open. */
  get isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  get isClosed(): boolean {
    return this.closed;
  }
}

// ---------------------------------------------------------------------------
// React hook for WebSocket streaming
// ---------------------------------------------------------------------------

import { useEffect, useRef, useCallback, useState } from 'react';

export interface UseWsStreamOptions extends Omit<WSClientOptions, 'pathSegment'> {
  /** Auto-connect when taskId is set (default: true) */
  autoConnect?: boolean;
}

export interface UseWsStreamReturn {
  /** The WSClient instance (null before connect) */
  client: WSClient | null;
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
 * useWsStream — React hook for WebSocket streaming of a task.
 *
 * Usage:
 *   const { client, isConnected, error, disconnect } = useWsStream(taskId);
 *
 * The hook automatically dispatches incoming events to the Zustand store.
 * Pass `autoConnect={false}` to manually control connection.
 */
export function useWsStream(
  taskId: string | null,
  options: UseWsStreamOptions = {}
): UseWsStreamReturn {
  const {
    autoConnect = true,
    onDone,
    onError,
    onConnect,
    autoReconnect,
    maxRetries,
    reconnectDelay,
    pingInterval,
  } = options;

  const clientRef = useRef<WSClient | null>(null);
  const [isDone, setIsDone] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const connect = useCallback(() => {
    if (!taskId || isDone) return;
    if (clientRef.current) {
      clientRef.current.close('cancelled');
    }

    const client = new WSClient(taskId, {
      autoReconnect: autoReconnect ?? true,
      maxRetries: maxRetries ?? 5,
      reconnectDelay: reconnectDelay ?? 1000,
      pingInterval: pingInterval,
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
  }, [taskId, isDone, autoReconnect, maxRetries, reconnectDelay, pingInterval, onDone, onError, onConnect]);

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

export interface StreamTaskOptions extends WSClientOptions {
  /** Timeout in ms (default: no timeout) */
  timeoutMs?: number;
}

/**
 * streamTask — Connect to a task WebSocket stream and resolve when complete/failed.
 *
 * Usage:
 *   const result = await streamTask('task-123', { timeoutMs: 60_000 });
 *   // result.status: 'completed' | 'failed' | 'timeout' | 'cancelled'
 */
export function streamTask(
  taskId: string,
  options: StreamTaskOptions = {}
): Promise<{ status: 'completed' | 'failed' | 'timeout' | 'cancelled'; error?: string }> {
  return new Promise((resolve) => {
    const client = new WSClient(taskId, {
      ...options,
      onDone: (_tid, reason) => {
        resolve({
          status: reason === 'complete' ? 'completed' :
                  reason === 'failed' ? 'failed' :
                  reason === 'cancelled' ? 'cancelled' : 'failed',
        });
      },
      onError: (_tid, err) => {
        resolve({ status: 'failed', error: err });
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
