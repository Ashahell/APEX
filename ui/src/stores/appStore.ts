import { create } from 'zustand';

export type ConnectionState = 'connected' | 'degraded' | 'disconnected';

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface Toast {
  id: string;
  type: ToastType;
  message: string;
  duration?: number;
}

export interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: Date;
  attachments?: string[];
}

export interface Task {
  id: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  tier: 'instant' | 'shallow' | 'deep';
  input: string;
  output?: string;
  error?: string;
  createdAt: Date;
  completedAt?: Date;
  cost?: number;
  skillName?: string;
}

export interface PendingConfirmation {
  taskId: string;
  tier: 'T1' | 'T2' | 'T3';
  action: string;
  skillName?: string;
  consequences?: {
    files_read: string[];
    files_written: string[];
    commands_executed: string[];
    blast_radius: 'minimal' | 'limited' | 'extensive';
    summary: string;
  };
}

export interface Notification {
  id: string;
  notification_type: string;
  title: string;
  message: string;
  severity: string;
  read: boolean;
  created_at_ms: number;
  data?: Record<string, unknown>;
}

export interface ExecutionStep {
  id: string;
  taskId: string;
  type: 'Thought' | 'ToolCall' | 'ToolProgress' | 'ToolResult' | 'ApprovalNeeded' | 'Error' | 'Complete';
  step: number;
  content?: string;
  tool?: string;
  input?: Record<string, unknown>;
  output?: string;
  success?: boolean;
  timestamp: Date;
}

interface AppState {
  messages: Message[];
  tasks: Task[];
  notifications: Notification[];
  executionSteps: ExecutionStep[];
  isConnected: boolean;
  connectionState: ConnectionState;
  sessionCost: number;
  totalCost: number;
  pendingConfirmation: PendingConfirmation | null;
  messageQueue: string[];
  isProcessingQueue: boolean;
  toasts: Toast[];
  
  addMessage: (message: Omit<Message, 'id' | 'timestamp'>) => void;
  addToast: (toast: Omit<Toast, 'id'>) => void;
  removeToast: (id: string) => void;
  addToMessageQueue: (message: string) => void;
  removeFromMessageQueue: (index: number) => void;
  clearMessageQueue: () => void;
  setIsProcessingQueue: (processing: boolean) => void;
  addTask: (task: Omit<Task, 'id' | 'createdAt'>) => void;
  updateTask: (id: string, updates: Partial<Task>) => void;
  addNotification: (notification: Notification) => void;
  addExecutionStep: (taskId: string, step: Omit<ExecutionStep, 'id' | 'taskId' | 'timestamp'>) => void;
  clearExecutionSteps: (taskId: string) => void;
  setConnected: (connected: boolean) => void;
  setConnectionState: (state: ConnectionState) => void;
  setSessionCost: (cost: number) => void;
  setTotalCost: (cost: number) => void;
  setPendingConfirmation: (confirmation: PendingConfirmation | null) => void;
  clearMessages: () => void;
}

export const useAppStore = create<AppState>((set) => ({
  messages: [],
  tasks: [],
  notifications: [],
  executionSteps: [],
  isConnected: false,
  connectionState: 'disconnected',
  sessionCost: 0,
  totalCost: 0,
  pendingConfirmation: null,
  messageQueue: [],
  isProcessingQueue: false,
  toasts: [],

  addMessage: (message) =>
    set((state) => ({
      messages: [
        ...state.messages,
        {
          ...message,
          id: crypto.randomUUID(),
          timestamp: new Date(),
        },
      ],
    })),

  addToMessageQueue: (message) =>
    set((state) => ({
      messageQueue: [...state.messageQueue, message],
    })),

  removeFromMessageQueue: (index) =>
    set((state) => ({
      messageQueue: state.messageQueue.filter((_, i) => i !== index),
    })),

  clearMessageQueue: () => set({ messageQueue: [] }),

  setIsProcessingQueue: (isProcessingQueue) => set({ isProcessingQueue }),

  addTask: (task) =>
    set((state) => ({
      tasks: [
        ...state.tasks,
        {
          ...task,
          ...task,
          id: crypto.randomUUID(),
          createdAt: new Date(),
        },
      ],
    })),

  updateTask: (id, updates) =>
    set((state) => ({
      tasks: state.tasks.map((t) =>
        t.id === id ? { ...t, ...updates } : t
      ),
    })),

  addNotification: (notification) =>
    set((state) => ({
      notifications: [notification, ...state.notifications].slice(0, 50),
    })),

  addExecutionStep: (taskId, step) =>
    set((state) => ({
      executionSteps: [
        ...state.executionSteps,
        {
          ...step,
          id: crypto.randomUUID(),
          taskId,
          timestamp: new Date(),
        },
      ],
    })),

  clearExecutionSteps: (taskId) =>
    set((state) => ({
      executionSteps: state.executionSteps.filter((s) => s.taskId !== taskId),
    })),

  setConnected: (connected) => set({ isConnected: connected }),

  setConnectionState: (connectionState) => set({ 
    connectionState,
    isConnected: connectionState === 'connected' 
  }),

  setSessionCost: (sessionCost) => set({ sessionCost }),

  setTotalCost: (totalCost) => set({ totalCost }),

  setPendingConfirmation: (pendingConfirmation) => set({ pendingConfirmation }),

  clearMessages: () => set({ messages: [] }),

  addToast: (toast) =>
    set((state) => ({
      toasts: [
        ...state.toasts,
        { ...toast, id: crypto.randomUUID() },
      ],
    })),

  removeToast: (id) =>
    set((state) => ({
      toasts: state.toasts.filter((t) => t.id !== id),
    })),
}));
