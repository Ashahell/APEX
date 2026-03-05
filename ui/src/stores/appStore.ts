import { create } from 'zustand';

export type ConnectionState = 'connected' | 'degraded' | 'disconnected';

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

interface AppState {
  messages: Message[];
  tasks: Task[];
  notifications: Notification[];
  isConnected: boolean;
  connectionState: ConnectionState;
  sessionCost: number;
  totalCost: number;
  pendingConfirmation: PendingConfirmation | null;
  
  addMessage: (message: Omit<Message, 'id' | 'timestamp'>) => void;
  addTask: (task: Omit<Task, 'id' | 'createdAt'>) => void;
  updateTask: (id: string, updates: Partial<Task>) => void;
  addNotification: (notification: Notification) => void;
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
  isConnected: false,
  connectionState: 'disconnected',
  sessionCost: 0,
  totalCost: 0,
  pendingConfirmation: null,

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

  setConnected: (connected) => set({ isConnected: connected }),

  setConnectionState: (connectionState) => set({ 
    connectionState,
    isConnected: connectionState === 'connected' 
  }),

  setSessionCost: (sessionCost) => set({ sessionCost }),

  setTotalCost: (totalCost) => set({ totalCost }),

  setPendingConfirmation: (pendingConfirmation) => set({ pendingConfirmation }),

  clearMessages: () => set({ messages: [] }),
}));
