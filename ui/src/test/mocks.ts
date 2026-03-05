import { vi } from 'vitest';

export interface MockNotification {
  id: string;
  notification_type: string;
  title: string;
  message: string;
  severity: string;
  read: boolean;
  created_at_ms: number;
  data?: Record<string, unknown>;
}

export interface MockWorkflow {
  id: string;
  name: string;
  description?: string;
  definition: string;
  category?: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface MockAdapter {
  name: string;
  adapter_type: string;
  enabled: boolean;
  config: Record<string, unknown>;
}

export interface MockWebhook {
  id: string;
  name: string;
  url: string;
  events: string[];
  enabled: boolean;
  created_at_ms: number;
  last_triggered_ms?: number;
  failure_count: number;
}

export interface MockFile {
  name: string;
  path: string;
  isDirectory: boolean;
  size?: number;
  modified?: number;
}

export interface MockTask {
  id: string;
  input_content: string;
  status: string;
  tier: string;
  priority?: string;
  category?: string;
  project?: string;
  created_at: string;
  updated_at: string;
  cost?: number;
}

const defaultNotifications: MockNotification[] = [
  {
    id: '1',
    notification_type: 'task_completed',
    title: 'Task Completed',
    message: 'Your task "Test task" has been completed',
    severity: 'info',
    read: false,
    created_at_ms: Date.now() - 60000,
  },
  {
    id: '2',
    notification_type: 'task_failed',
    title: 'Task Failed',
    message: 'Task "Analysis" failed due to timeout',
    severity: 'error',
    read: false,
    created_at_ms: Date.now() - 3600000,
  },
];

const defaultWorkflows: MockWorkflow[] = [
  {
    id: 'wf1',
    name: 'Daily Standup',
    description: 'Morning standup workflow',
    definition: '{"steps":[]}',
    category: 'meeting',
    is_active: true,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
];

const defaultAdapters: MockAdapter[] = [
  { name: 'slack', adapter_type: 'slack', enabled: true, config: { webhook_url: '' } },
  { name: 'telegram', adapter_type: 'telegram', enabled: false, config: { bot_token: '' } },
];

const defaultWebhooks: MockWebhook[] = [
  {
    id: 'wh1',
    name: 'Task Alerts',
    url: 'https://example.com/webhook',
    events: ['task.completed'],
    enabled: true,
    created_at_ms: Date.now() - 86400000,
    failure_count: 0,
  },
];

const defaultFiles: MockFile[] = [
  { name: 'src', path: '/src', isDirectory: true, modified: Date.now() - 1000 },
  { name: 'README.md', path: '/README.md', isDirectory: false, size: 1024, modified: Date.now() - 2000 },
  { name: 'package.json', path: '/package.json', isDirectory: false, size: 512, modified: Date.now() - 3000 },
];

const defaultTasks: MockTask[] = [
  {
    id: 'task1',
    input_content: 'Test task 1',
    status: 'pending',
    tier: 'instant',
    priority: 'high',
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: 'task2',
    input_content: 'Test task 2',
    status: 'running',
    tier: 'shallow',
    priority: 'medium',
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    cost: 0.05,
  },
];

export function createApiMock() {
  let notifications = [...defaultNotifications];
  let workflows = [...defaultWorkflows];
  let adapters = [...defaultAdapters];
  let webhooks = [...defaultWebhooks];
  let files = [...defaultFiles];
  let tasks = [...defaultTasks];

  return {
    notifications,
    workflows,
    adapters,
    webhooks,
    files,
    tasks,
    reset: () => {
      notifications = [...defaultNotifications];
      workflows = [...defaultWorkflows];
      adapters = [...defaultAdapters];
      webhooks = [...defaultWebhooks];
      files = [...defaultFiles];
      tasks = [...defaultTasks];
    },
  };
}

export function mockApiResponse(data: unknown, ok = true) {
  return {
    ok,
    json: async () => data,
  } as unknown as Response;
}

export function setupApiMocks(apiModule: typeof import('../lib/api')) {
  vi.spyOn(apiModule, 'apiGet').mockImplementation(async (path: string) => {
    if (path.includes('/api/v1/notifications')) {
      return mockApiResponse([]);
    }
    if (path.includes('/api/v1/workflows')) {
      return mockApiResponse([]);
    }
    if (path.includes('/api/v1/adapters')) {
      return mockApiResponse([]);
    }
    if (path.includes('/api/v1/webhooks')) {
      return mockApiResponse([]);
    }
    if (path.includes('/api/v1/files')) {
      return mockApiResponse([]);
    }
    if (path.includes('/api/v1/tasks')) {
      return mockApiResponse({ tasks: [], total: 0 });
    }
    return mockApiResponse({});
  });

  vi.spyOn(apiModule, 'apiPost').mockResolvedValue(mockApiResponse({ success: true }));
  vi.spyOn(apiModule, 'apiPut').mockResolvedValue(mockApiResponse({ success: true }));
  vi.spyOn(apiModule, 'apiDelete').mockResolvedValue(mockApiResponse({ success: true }));
}

export const mockData = {
  notifications: defaultNotifications,
  workflows: defaultWorkflows,
  adapters: defaultAdapters,
  webhooks: defaultWebhooks,
  files: defaultFiles,
  tasks: defaultTasks,
};
