import React, { useEffect, useRef, useState } from 'react';
import { SSEClient } from '../../lib/sse';

type Props = {
  taskId: string;
  streamEnabled?: boolean;
};

interface ScreenshotEvent {
  type: 'ScreenshotUpdate';
  screenshot_url?: string;
  screenshot_base64?: string;
  timestamp: number;
}

// Renders a base64 screenshot onto the canvas
const renderScreenshot = (canvas: HTMLCanvasElement | null, event: ScreenshotEvent) => {
  if (!canvas || !event.screenshot_base64) return;
  const ctx = canvas.getContext('2d');
  if (!ctx) return;
  const img = new Image();
  img.onload = () => {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
  };
  img.src = `data:image/png;base64,${event.screenshot_base64}`;
};

const ComputerUseViewer: React.FC<Props> = ({ taskId, streamEnabled = true }) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [status, setStatus] = useState<'idle' | 'connecting' | 'connected' | 'error'>('idle');
  const [lastUpdate, setLastUpdate] = useState<string>('Never');
  const [error, setError] = useState<string | null>(null);
  const [screenshotBase64, setScreenshotBase64] = useState<string | null>(null);
  const clientRef = useRef<SSEClient | null>(null);

  useEffect(() => {
    if (!streamEnabled || !taskId) {
      setStatus('idle');
      return;
    }

    setStatus('connecting');
    setError(null);
    setScreenshotBase64(null);

    const client = new SSEClient(taskId, {
      pathSegment: 'task',
      maxRetries: 3,
      onConnect: () => setStatus('connected'),
      onDone: () => setStatus('idle'),
      onError: (_tid, err) => {
        setError(err);
        setStatus('error');
      },
    });

    clientRef.current = client;

    // Intercept ScreenshotUpdate events directly from the EventSource
    // to render live screenshots on the canvas.
    // This bypasses the SSEClient's ExecutionEvent dispatch.
    const buildSignedUrl = async () => {
      const path = `/api/v1/stream/task/${taskId}`;
      const timestamp = Math.floor(Date.now() / 1000);
      const encoder = new TextEncoder();
      const keyData = encoder.encode('dev-secret-change-in-production');
      const key = await crypto.subtle.importKey('raw', keyData, { name: 'HMAC', hash: 'SHA-256' }, false, ['sign']);
      const message = `${timestamp}GET${path}`;
      const sigBuf = await crypto.subtle.sign('HMAC', key, encoder.encode(message));
      const sig = Array.from(new Uint8Array(sigBuf)).map(b => b.toString(16).padStart(2, '0')).join('');
      const nonce = `${timestamp}-${Math.random().toString(36).slice(2)}`;
      return `http://localhost:3000${path}?__timestamp=${timestamp}&__signature=${sig}&__nonce=${nonce}`;
    };

    let es: EventSource | null = null;

    buildSignedUrl().then((url) => {
      es = new EventSource(url);

      es.addEventListener('ScreenshotUpdate', (e: MessageEvent) => {
        try {
          const event = JSON.parse(e.data) as ScreenshotEvent;
          setScreenshotBase64(event.screenshot_base64 ?? null);
          setLastUpdate(new Date().toLocaleTimeString());
          if (canvasRef.current && event.screenshot_base64) {
            renderScreenshot(canvasRef.current, event);
          }
        } catch {
          // ignore parse errors
        }
      });

      es.onerror = () => {
        es?.close();
      };
    });

    return () => {
      client.close('cancelled');
      clientRef.current = null;
    };
  }, [taskId, streamEnabled]);

  const statusColor = {
    idle: '#6b7280',
    connecting: '#f59e0b',
    connected: '#10b981',
    error: '#ef4444',
  }[status];

  const statusText = {
    idle: 'Stream disabled',
    connecting: 'Connecting...',
    connected: 'Live stream',
    error: 'Stream error',
  }[status];

  return (
    <div
      style={{
        border: '1px solid var(--color-border, #374151)',
        borderRadius: 8,
        padding: 12,
        background: 'var(--color-panel, #1f2937)',
      }}
    >
      {/* Header */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          marginBottom: 8,
        }}
      >
        <div style={{ fontWeight: 600, fontSize: 13, color: 'var(--color-text, #f9fafb)' }}>
          Computer Use Viewer
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
          <span
            style={{
              display: 'inline-block',
              width: 8,
              height: 8,
              borderRadius: '50%',
              background: statusColor,
              ...(status === 'connected' ? { animation: 'pulse 2s infinite' } : {}),
            }}
          />
          <span style={{ fontSize: 11, color: statusColor }}>{statusText}</span>
        </div>
      </div>

      {/* Task info */}
      <div style={{ fontSize: 11, color: '#6b7280', marginBottom: 8 }}>
        Task: <code style={{ color: '#9ca3af' }}>{taskId.slice(0, 16)}…</code>
        {status === 'connected' && (
          <span className="ml-2 text-green-400">● streaming</span>
        )}
      </div>

      {/* Screenshot canvas */}
      <div
        style={{
          height: 240,
          background: '#111827',
          borderRadius: 6,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          position: 'relative',
          overflow: 'hidden',
        }}
      >
        <canvas
          ref={canvasRef}
          width={320}
          height={240}
          style={{
            width: '100%',
            height: '100%',
            objectFit: 'contain',
          }}
        />
        {/* Placeholder overlay */}
        {status !== 'connected' && (
          <div
            style={{
              position: 'absolute',
              inset: 0,
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              justifyContent: 'center',
              gap: 8,
              background: 'rgba(17, 24, 39, 0.8)',
            }}
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width={32}
              height={32}
              viewBox="0 0 24 24"
              fill="none"
              stroke="#6b7280"
              strokeWidth={1.5}
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <rect x={2} y={3} width={20} height={14} rx={2} ry={2} />
              <line x1={8} y1={21} x2={16} y2={21} />
              <line x1={12} y1={17} x2={12} y2={21} />
            </svg>
            <span style={{ fontSize: 12, color: '#6b7280' }}>
              {status === 'connecting' ? 'Connecting to stream...' :
               status === 'error' ? (error || 'Connection failed') :
               'Live screenshot stream'}
            </span>
          </div>
        )}
      </div>

      {/* Footer */}
      <div
        style={{
          marginTop: 6,
          fontSize: 10,
          color: '#6b7280',
          display: 'flex',
          justifyContent: 'space-between',
        }}
      >
        <span>Last update: {lastUpdate}</span>
        {screenshotBase64 && (
          <span className="text-green-400">Screenshot received ({Math.round(screenshotBase64.length * 0.75 / 1024)} KB)</span>
        )}
        {streamEnabled && (
          <span>
            Endpoint:{' '}
            <code style={{ color: '#9ca3af' }}>
              /api/v1/stream/task/{taskId.slice(0, 8)}
            </code>
          </span>
        )}
      </div>

      <style>{`
        @keyframes pulse {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.4; }
        }
      `}</style>
    </div>
  );
};

export default ComputerUseViewer;
