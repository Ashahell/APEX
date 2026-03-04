import { io, Socket } from 'socket.io-client';
import { useAppStore } from '../stores/appStore';

export type ConnectionState = 'connected' | 'degraded' | 'disconnected';

export interface TaskUpdate {
  taskId: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  output?: string;
  error?: string;
  step?: {
    type: 'GEN' | 'USE' | 'EXE' | 'WWW' | 'SUB' | 'MEM' | 'AUD';
    name: string;
    input?: Record<string, unknown>;
    output?: string;
  };
  cost?: number;
}

export interface ExecutionEvent {
  type: 'Thought' | 'ToolCall' | 'ToolProgress' | 'ToolResult' | 'ApprovalNeeded' | 'Error' | 'Complete';
  data: {
    step?: number;
    content?: string;
    tool?: string;
    input?: Record<string, unknown>;
    output?: string;
    success?: boolean;
    tier?: string;
    action?: string;
    consequences?: {
      files_read: string[];
      files_written: string[];
      commands_executed: string[];
      blast_radius: 'minimal' | 'limited' | 'extensive';
      summary: string;
    };
    message?: string;
    steps?: number;
    tools_used?: string[];
  };
}

export interface ServerMessage {
  type: 'task_update' | 'task_created' | 'metrics' | 'execution_event' | 'error';
  payload: unknown;
}

class WebSocketClient {
  private socket: Socket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;
  private pollingInterval: number | null = null;
  private baseUrl: string;

  constructor() {
    this.baseUrl = import.meta.env.VITE_WS_URL || 'http://localhost:3000';
  }

  connect(): void {
    const store = useAppStore.getState();
    store.setConnectionState('disconnected');

    try {
      this.socket = io(this.baseUrl, {
        path: '/api/v1/ws',
        transports: ['websocket', 'polling'],
        reconnection: true,
        reconnectionAttempts: this.maxReconnectAttempts,
        reconnectionDelay: this.reconnectDelay,
        reconnectionDelayMax: 30000,
        timeout: 10000,
      });

      this.socket.on('connect', () => {
        console.log('WebSocket connected');
        this.reconnectAttempts = 0;
        store.setConnectionState('connected');
        this.stopPolling();
      });

      this.socket.on('disconnect', (reason) => {
        console.log('WebSocket disconnected:', reason);
        store.setConnectionState('disconnected');
        if (reason === 'io server disconnect') {
          this.socket?.connect();
        }
      });

      this.socket.on('connect_error', (error) => {
        console.error('WebSocket connection error:', error);
        this.reconnectAttempts++;
        if (this.reconnectAttempts >= 2) {
          store.setConnectionState('degraded');
          this.startPolling();
        }
      });

      this.socket.on('task_update', (data: TaskUpdate) => {
        this.handleTaskUpdate(data);
      });

      this.socket.on('task_created', (data: TaskUpdate) => {
        this.handleTaskUpdate(data);
      });

      this.socket.on('metrics', (data: { totalCost: number; sessionCost: number }) => {
        store.setSessionCost(data.sessionCost);
        store.setTotalCost(data.totalCost);
      });

      this.socket.on('execution_event', (event: ExecutionEvent) => {
        this.handleExecutionEvent(event);
      });

      this.socket.on('error', (error: Error) => {
        console.error('WebSocket error:', error);
        store.setConnectionState('degraded');
      });

    } catch (error) {
      console.error('Failed to create WebSocket:', error);
      store.setConnectionState('disconnected');
      this.startPolling();
    }
  }

  private handleTaskUpdate(data: TaskUpdate): void {
    const store = useAppStore.getState();
    
    if (data.status === 'completed' || data.status === 'failed') {
      store.updateTask(data.taskId, {
        status: data.status,
        output: data.output,
        error: data.error,
        completedAt: new Date(),
      });
    } else {
      store.updateTask(data.taskId, {
        status: data.status,
        output: data.output,
      });
    }
  }

  private handleExecutionEvent(event: ExecutionEvent): void {
    const store = useAppStore.getState();
    
    if (event.type === 'ApprovalNeeded') {
      const data = event.data;
      const tier = (data.tier as 'T1' | 'T2' | 'T3') || 'T2';
      
      store.setPendingConfirmation({
        taskId: store.pendingConfirmation?.taskId || 'pending',
        tier,
        action: data.action || 'Unknown action',
        consequences: data.consequences,
      });
    }
    
    if (event.type === 'Thought' || event.type === 'ToolCall' || event.type === 'ToolResult') {
      console.log('Execution event:', event.type, event.data);
    }
  }

  private startPolling(): void {
    if (this.pollingInterval) return;
    
    console.log('Starting polling fallback');
    const store = useAppStore.getState();
    
    this.pollingInterval = window.setInterval(async () => {
      try {
        const response = await fetch(`${this.baseUrl}/api/v1/metrics`);
        if (response.ok) {
          const data = await response.json();
          store.setSessionCost(data.total_cost_usd || 0);
        }
      } catch {
        // Polling failed
      }
    }, 5000);
  }

  private stopPolling(): void {
    if (this.pollingInterval) {
      clearInterval(this.pollingInterval);
      this.pollingInterval = null;
    }
  }

  disconnect(): void {
    this.stopPolling();
    if (this.socket) {
      this.socket.disconnect();
      this.socket = null;
    }
  }

  isConnected(): boolean {
    return this.socket?.connected ?? false;
  }

  on(event: string, callback: (data: unknown) => void): void {
    this.socket?.on(event, callback);
  }

  off(event: string, callback?: (data: unknown) => void): void {
    if (callback) {
      this.socket?.off(event, callback);
    } else {
      this.socket?.off(event);
    }
  }
}

export const wsClient = new WebSocketClient();
