import { useState, useEffect, useCallback, useRef } from 'react';

// ============================================================================
// Types - SSE Envelope and Payloads (mirrored from StreamingDashboard)
// ============================================================================

// Phase 1: Extended event types
export type StreamEventType = 
  | 'connected' 
  | 'disconnected' 
  | 'hands' 
  | 'mcp' 
  | 'task' 
  | 'stats' 
  | 'heartbeat' 
  | 'error'
  // Phase 1: Richer event types
  | 'session_start'
  | 'session_end'
  | 'checkpoint'
  | 'user_intervention';

export interface SseEnvelope<T> {
  type: StreamEventType;
  timestamp: number;
  trace_id?: string;
  payload: T;
}

interface UseStreamingOptions {
  /** Auto-connect on mount */
  autoConnect?: boolean;
  /** Reconnection delay in ms */
  reconnectDelay?: number;
  /** Maximum reconnection attempts */
  maxReconnectAttempts?: number;
  /** Callback on connect */
  onConnect?: () => void;
  /** Callback on disconnect */
  onDisconnect?: () => void;
  /** Callback on error */
  onError?: (error: Event) => void;
}

interface UseStreamingReturn {
  /** All collected events */
  events: SseEnvelope<unknown>[];
  /** Whether currently connected */
  connected: boolean;
  /** Last error */
  error: Event | null;
  /** Connect manually */
  connect: () => void;
  /** Disconnect manually */
  disconnect: () => void;
  /** Clear all events */
  clear: () => void;
}

/**
 * Hook to consume SSE streams from APEX streaming endpoints.
 * Supports browser-friendly authentication via query params (?sig=...&ts=...)
 * 
 * @param endpoint - The SSE endpoint path (e.g., '/stream/stats', '/stream/hands/:taskId')
 * @param options - Configuration options
 * @returns Streaming state and control functions
 */
export function useStreaming(
  endpoint: string,
  options: UseStreamingOptions = {}
): UseStreamingReturn {
  const {
    autoConnect = true,
    reconnectDelay = 3000,
    maxReconnectAttempts = 5,
    onConnect,
    onDisconnect,
    onError,
  } = options;

  const [events, setEvents] = useState<SseEnvelope<unknown>[]>([]);
  const [connected, setConnected] = useState(false);
  const [error, setError] = useState<Event | null>(null);

  const eventSourceRef = useRef<EventSource | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  // Parse the base URL from the current window location
  const getBaseUrl = useCallback(() => {
    if (typeof window === 'undefined') return 'http://localhost:3000';
    return `${window.location.protocol}//${window.location.host}`;
  }, []);

  // Generate signed URL for SSE (browser-friendly auth)
  // Calls backend endpoint to get a signed URL with HMAC query params
  const getSignedUrl = useCallback(async (path: string): Promise<string> => {
    const baseUrl = getBaseUrl();
    
    try {
      // Use api.ts helper for authenticated request
      const { apiGet } = await import('../lib/api');
      const response = await apiGet(`/api/v1/streams/sign?path=${encodeURIComponent(path)}`);
      
      if (!response.ok) {
        console.warn('Failed to get signed URL, falling back to unsigned:', response.status);
        return `${baseUrl}${path}`;
      }
      
      const data = await response.json();
      // The backend returns { url, signature, timestamp, expires_in }
      // The URL already includes the query params
      return `${baseUrl}${data.url}`;
    } catch (err) {
      console.warn('Error getting signed URL, falling back to unsigned:', err);
      return `${baseUrl}${path}`;
    }
  }, [getBaseUrl]);

  // Connect to SSE endpoint
  const connect = useCallback(async () => {
    // Clean up existing connection
    if (eventSourceRef.current) {
      eventSourceRef.current.close();
    }

    try {
      const url = await getSignedUrl(endpoint);
      const eventSource = new EventSource(url);

      eventSource.onopen = () => {
        setConnected(true);
        setError(null);
        reconnectAttemptsRef.current = 0;
        onConnect?.();
      };

      eventSource.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data) as SseEnvelope<unknown>;
          setEvents((prev) => [...prev, data]);
        } catch (parseError) {
          console.error('Failed to parse SSE message:', parseError);
        }
      };

      eventSource.onerror = (err) => {
        setConnected(false);
        setError(err);
        onError?.(err);

        // Attempt reconnection
        if (reconnectAttemptsRef.current < maxReconnectAttempts) {
          reconnectAttemptsRef.current += 1;
          reconnectTimeoutRef.current = setTimeout(() => {
            connect();
          }, reconnectDelay);
        }
      };

      eventSourceRef.current = eventSource;
    } catch (err) {
      setError(err as Event);
      setConnected(false);
    }
  }, [endpoint, getSignedUrl, reconnectDelay, maxReconnectAttempts, onConnect, onError]);

  // Disconnect from SSE endpoint
  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    if (eventSourceRef.current) {
      eventSourceRef.current.close();
      eventSourceRef.current = null;
    }

    setConnected(false);
    onDisconnect?.();
  }, [onDisconnect]);

  // Clear all events
  const clear = useCallback(() => {
    setEvents([]);
  }, []);

  // Auto-connect on mount
  useEffect(() => {
    if (autoConnect) {
      connect();
    }

    return () => {
      disconnect();
    };
  }, [autoConnect, connect, disconnect]);

  return {
    events,
    connected,
    error,
    connect,
    disconnect,
    clear,
  };
}

export default useStreaming;
