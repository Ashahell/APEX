import crypto from 'crypto';

export interface Config {
  baseURL?: string;
  sharedSecret: string;
  timeout?: number;
}

export interface TaskRequest {
  content: string;
  channel?: string;
  thread_id?: string;
  author?: string;
  max_steps?: number;
  budget_usd?: number;
  time_limit_secs?: number;
  project?: string;
  priority?: 'low' | 'medium' | 'high' | 'urgent';
  category?: string;
}

export interface TaskResponse {
  task_id: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  tier: 'instant' | 'shallow' | 'deep';
  capability_token?: string;
  instant_response?: string;
}

export interface TaskStatusResponse {
  task_id: string;
  status: string;
  content?: string;
  output?: string;
  error?: string;
  project?: string;
  priority?: string;
  category?: string;
  created_at?: string;
}

export interface TaskFilter {
  project?: string;
  status?: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  priority?: 'low' | 'medium' | 'high' | 'urgent';
  category?: string;
  limit?: number;
  offset?: number;
}

export interface Skill {
  name: string;
  version: string;
  tier: string;
  description?: string;
  healthy: boolean;
  last_health_check?: string;
}

export interface ExecuteSkillRequest {
  skill_name: string;
  input: Record<string, unknown>;
}

export interface Metrics {
  tasks: number;
  by_tier: Record<string, number>;
  by_status: Record<string, number>;
  total_cost_usd: number;
}

export interface HealthResponse {
  status: string;
}

function buildQueryString(params: Record<string, unknown>): string {
  const query = new URLSearchParams();
  Object.entries(params).forEach(([key, value]) => {
    if (value !== undefined && value !== null && value !== '') {
      query.append(key, String(value));
    }
  });
  return query.toString();
}

export class ApexClient {
  private baseURL: string;
  private sharedSecret: string;
  private timeout: number;

  constructor(config: Config) {
    this.baseURL = config.baseURL || 'http://localhost:3000';
    this.sharedSecret = config.sharedSecret;
    this.timeout = config.timeout || 30000;
  }

  private async signRequest(method: string, path: string, body: string): Promise<Headers> {
    const timestamp = Math.floor(Date.now() / 1000).toString();
    const message = timestamp + method + path + body;
    
    const hmac = crypto.createHmac('sha256', this.sharedSecret);
    hmac.update(message);
    const signature = hmac.digest('hex');
    
    return new Headers({
      'Content-Type': 'application/json',
      'X-APEX-Signature': signature,
      'X-APEX-Timestamp': timestamp,
    });
  }

  private async request<T>(
    method: string,
    path: string,
    body?: unknown
  ): Promise<T> {
    const bodyStr = body ? JSON.stringify(body) : '';
    const headers = await this.signRequest(method, path, bodyStr);
    
    const url = this.baseURL + path;
    const response = await fetch(url, {
      method,
      headers,
      body: bodyStr || undefined,
      signal: AbortSignal.timeout(this.timeout),
    });
    
    if (!response.ok) {
      const error = await response.text();
      throw new Error(`API error: ${response.status} - ${error}`);
    }
    
    return response.json();
  }

  async createTask(request: TaskRequest): Promise<TaskResponse> {
    return this.request<TaskResponse>('POST', '/api/v1/tasks', request);
  }

  async getTask(taskId: string): Promise<TaskStatusResponse> {
    return this.request<TaskStatusResponse>('GET', `/api/v1/tasks/${taskId}`);
  }

  async listTasks(filter?: TaskFilter): Promise<TaskStatusResponse[]> {
    const query = filter ? '?' + buildQueryString(filter) : '';
    return this.request<TaskStatusResponse[]>('GET', `/api/v1/tasks${query}`);
  }

  async cancelTask(taskId: string): Promise<TaskStatusResponse> {
    return this.request<TaskStatusResponse>('POST', `/api/v1/tasks/${taskId}/cancel`);
  }

  async updateTask(
    taskId: string,
    updates: { project?: string; priority?: string; category?: string; status?: string }
  ): Promise<TaskStatusResponse> {
    return this.request<TaskStatusResponse>('PUT', `/api/v1/tasks/${taskId}`, updates);
  }

  async listSkills(): Promise<Skill[]> {
    return this.request<Skill[]>('GET', '/api/v1/skills');
  }

  async getSkill(name: string): Promise<Skill> {
    return this.request<Skill>('GET', `/api/v1/skills/${name}`);
  }

  async executeSkill(request: ExecuteSkillRequest): Promise<unknown> {
    return this.request('POST', '/api/v1/skills/execute', request);
  }

  async getMetrics(): Promise<Metrics> {
    return this.request<Metrics>('GET', '/api/v1/metrics');
  }

  async healthCheck(): Promise<HealthResponse> {
    return this.request<HealthResponse>('GET', '/health');
  }

  async getFilterOptions(): Promise<{
    projects: string[];
    categories: string[];
    priorities: string[];
    statuses: string[];
  }> {
    return this.request('GET', '/api/v1/tasks/filter-options');
  }
}

export default ApexClient;
