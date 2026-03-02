import { useEffect, useRef, useCallback } from 'react';
import { useAppStore } from '../stores/appStore';

const WS_URL = `ws://${window.location.host}/ws`;

export function useWebSocket() {
  const wsRef = useRef<WebSocket | null>(null);
  const { setConnected, addMessage, addTask } = useAppStore();

  const connect = useCallback(() => {
    try {
      if (wsRef.current?.readyState === WebSocket.OPEN) return;

      const ws = new WebSocket(WS_URL);
      
      ws.onopen = () => {
        console.log('WebSocket connected');
        setConnected(true);
      };

      ws.onclose = () => {
        console.log('WebSocket disconnected, using local mode');
        setConnected(false);
      };

      ws.onerror = () => {
        console.log('WebSocket unavailable, using local mode');
        setConnected(false);
      };

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          
          if (data.type === 'message') {
            addMessage({
              role: data.role,
              content: data.content,
              attachments: data.attachments,
            });
          } else if (data.type === 'task') {
            addTask({
              status: data.status,
              tier: data.tier,
              input: data.input,
              output: data.output,
              error: data.error,
            });
          }
        } catch {
          console.error('Failed to parse WebSocket message');
        }
      };

      wsRef.current = ws;
    } catch {
      console.log('WebSocket unavailable, using local mode');
      setConnected(false);
    }
  }, [setConnected, addMessage, addTask]);

  const send = useCallback((data: unknown) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(data));
    }
  }, []);

  const disconnect = useCallback(() => {
    wsRef.current?.close();
    wsRef.current = null;
  }, []);

  useEffect(() => {
    connect();
    return () => disconnect();
  }, [connect, disconnect]);

  return { send, connect, disconnect };
}
