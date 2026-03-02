import { create } from 'zustand';

export interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: Date;
  attachments?: string[];
}

export interface Task {
  id: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  tier: 'instant' | 'shallow' | 'deep';
  input: string;
  output?: string;
  error?: string;
  createdAt: Date;
  completedAt?: Date;
}

interface AppState {
  messages: Message[];
  tasks: Task[];
  isConnected: boolean;
  
  addMessage: (message: Omit<Message, 'id' | 'timestamp'>) => void;
  addTask: (task: Omit<Task, 'id' | 'createdAt'>) => void;
  updateTask: (id: string, updates: Partial<Task>) => void;
  setConnected: (connected: boolean) => void;
  clearMessages: () => void;
}

export const useAppStore = create<AppState>((set) => ({
  messages: [],
  tasks: [],
  isConnected: false,

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

  setConnected: (connected) => set({ isConnected: connected }),

  clearMessages: () => set({ messages: [] }),
}));
