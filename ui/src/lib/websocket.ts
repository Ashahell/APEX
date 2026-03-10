import { useAppStore } from '../stores/appStore';

export type ConnectionState = 'connected' | 'degraded' | 'disconnected';

export interface TaskUpdate {
  taskId: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  output?: string;
  error?: string;
  cost?: number;
}

// Execution event types from backend
export interface ExecutionEvent {
  type: 'Thought' | 'ToolCall' | 'ToolProgress' | 'ToolResult' | 'ApprovalNeeded' | 'Error' | 'Complete';
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
}

class WebSocketClient {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;
  private baseUrl: string;
  private pollingInterval: number | null = null;

  constructor() {
    this.baseUrl = import.meta.env.VITE_WS_URL?.replace('http', 'ws') || 'ws://localhost:3000';
  }

  connect(): void {
    const store = useAppStore.getState();
    store.setConnectionState('disconnected');

    try {
      this.ws = new WebSocket(`${this.baseUrl}/api/v1/ws`);
      
      this.ws.onopen = () => {
        this.reconnectAttempts = 0;
        store.setConnectionState('connected');
        this.stopPolling();
      };
      
      this.ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          this.handleMessage(data);
        } catch (e) {
          console.error('Failed to parse WebSocket message:', e);
        }
      };
      
      this.ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        this.reconnectAttempts++;
        if (this.reconnectAttempts >= 2) {
          store.setConnectionState('degraded');
          this.startPolling();
        }
      };
      
      this.ws.onclose = () => {
        this.reconnectAttempts++;
        if (this.reconnectAttempts < this.maxReconnectAttempts) {
          setTimeout(() => this.connect(), this.reconnectDelay * this.reconnectAttempts);
        } else {
          store.setConnectionState('disconnected');
          this.startPolling();
        }
      };
    } catch (error) {
      console.error('Failed to create WebSocket:', error);
      store.setConnectionState('disconnected');
      this.startPolling();
    }
  }

  private handleMessage(data: Record<string, unknown>): void {
    const store = useAppStore.getState();
    const msgType = data.type as string;
    
    switch (msgType) {
      case 'task_update':
      case 'task_created': {
        const taskId = (data.task_id || data.taskId) as string;
        const status = data.status as TaskUpdate['status'];
        if (taskId) {
          store.updateTask(taskId, {
            status,
            output: data.output as string | undefined,
            error: data.error as string | undefined,
          });
        }
        break;
      }
      // Handle execution stream events (Thought, ToolCall, ToolResult, etc.)
      case 'Thought': {
        const event = data as unknown as ExecutionEvent;
        const taskId = (data.task_id || data.taskId || event.step?.toString()) as string;
        if (taskId) {
          store.addExecutionStep(taskId, {
            type: 'Thought',
            step: event.step || 0,
            content: event.content || '',
          });
        }
        break;
      }
      case 'ToolCall': {
        const event = data as unknown as ExecutionEvent;
        const taskId = (data.task_id || data.taskId || event.step?.toString()) as string;
        if (taskId) {
          store.addExecutionStep(taskId, {
            type: 'ToolCall',
            step: event.step || 0,
            tool: event.tool || '',
            input: event.input || {},
          });
        }
        break;
      }
      case 'ToolResult': {
        const event = data as unknown as ExecutionEvent;
        const taskId = (data.task_id || data.taskId || event.step?.toString()) as string;
        if (taskId) {
          store.addExecutionStep(taskId, {
            type: 'ToolResult',
            step: event.step || 0,
            tool: event.tool || '',
            success: event.success ?? true,
            output: event.output || '',
          });
        }
        break;
      }
      case 'Complete': {
        const event = data as unknown as ExecutionEvent;
        const taskId = (data.task_id || data.taskId) as string;
        if (taskId) {
          store.updateTask(taskId, {
            status: 'completed',
            output: event.output,
          });
        }
        break;
      }
      case 'Error': {
        const event = data as unknown as ExecutionEvent;
        const taskId = (data.task_id || data.taskId) as string;
        if (taskId) {
          store.updateTask(taskId, {
            status: 'failed',
            error: event.message,
          });
        }
        break;
      }
      case 'notification': {
        const notification = data as unknown as Parameters<typeof store.addNotification>[0];
        store.addNotification(notification);
        break;
      }
      case 'metrics': {
        store.setSessionCost((data.sessionCost as number) || 0);
        store.setTotalCost((data.totalCost as number) || 0);
        break;
      }
    }
  }

  private startPolling(): void {
    if (this.pollingInterval) return;
    
    const store = useAppStore.getState();
    
    this.pollingInterval = window.setInterval(async () => {
      try {
        const response = await fetch('http://localhost:3000/api/v1/tasks?status=running&limit=10');
        if (response.ok) {
          const tasks = await response.json();
          tasks.forEach((task: TaskUpdate) => {
            store.updateTask(task.taskId, {
              status: task.status,
              output: task.output,
              error: task.error,
            });
          });
        }
      } catch (e) {
        console.error('Polling error:', e);
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
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  sendMessage(message: string): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(message);
    }
  }
}

export const wsClient = new WebSocketClient();
