import { FastifyInstance, FastifyRequest, FastifyReply } from 'fastify';

export interface RestAdapterConfig {
  port?: number;
}

export interface CreateTaskRequest {
  content: string;
  channel?: string;
  thread_id?: string;
  author?: string;
  max_steps?: number;
  budget_usd?: number;
  time_limit_secs?: number;
}

export interface TaskResponse {
  task_id: string;
  status: string;
  tier: string;
  capability_token?: string;
  instant_response?: string;
}

export interface MessageHistoryEntry {
  id: string;
  task_id: string | null;
  channel: string;
  thread_id: string | null;
  author: string;
  content: string;
  role: string;
  created_at: string;
}

export class RestAdapter {
  private app: FastifyInstance | null = null;
  private routerUrl: string;
  private port: number;

  constructor(routerUrl: string = 'http://localhost:3000', port: number = 3001) {
    this.routerUrl = routerUrl;
    this.port = port;
  }

  async start(): Promise<void> {
    const { fastify } = await import('fastify');
    this.app = fastify({ logger: true });

    this.app.get('/health', async () => ({ status: 'ok' }));

    this.app.post<{ Body: CreateTaskRequest }>(
      '/api/tasks',
      async (request: FastifyRequest<{ Body: CreateTaskRequest }>, reply: FastifyReply) => {
        const response = await fetch(`${this.routerUrl}/api/v1/tasks`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(request.body),
        });

        if (!response.ok) {
          reply.code(response.status);
          return { error: response.statusText };
        }

        return response.json();
      }
    );

    this.app.get<{ Params: { task_id: string } }>(
      '/api/tasks/:task_id',
      async (request: FastifyRequest<{ Params: { task_id: string } }>, reply: FastifyReply) => {
        const { task_id } = request.params;
        const response = await fetch(`${this.routerUrl}/api/v1/tasks/${task_id}`);

        if (!response.ok) {
          reply.code(response.status);
          return { error: response.statusText };
        }

        return response.json();
      }
    );

    this.app.get<{ Querystring: { limit?: string; offset?: string } }>(
      '/api/tasks',
      async (request: FastifyRequest<{ Querystring: { limit?: string; offset?: string } }>, reply: FastifyReply) => {
        const limit = request.query.limit || '100';
        const offset = request.query.offset || '0';
        const response = await fetch(`${this.routerUrl}/api/v1/tasks?limit=${limit}&offset=${offset}`);

        if (!response.ok) {
          reply.code(response.status);
          return { error: response.statusText };
        }

        return response.json();
      }
    );

    this.app.get('/api/metrics', async (reply: FastifyReply) => {
      const response = await fetch(`${this.routerUrl}/api/v1/metrics`);

      if (!response.ok) {
        reply.code(response.status);
        return { error: response.statusText };
      }

      return response.json();
    });

    this.app.get('/api/skills', async (reply: FastifyReply) => {
      const response = await fetch(`${this.routerUrl}/api/v1/skills`);

      if (!response.ok) {
        reply.code(response.status);
        return { error: response.statusText };
      }

      return response.json();
    });

    this.app.post<{ Body: { skill_name: string; input: Record<string, unknown> } }>(
      '/api/skills/execute',
      async (request: FastifyRequest<{ Body: { skill_name: string; input: Record<string, unknown> } }>, reply: FastifyReply) => {
        const response = await fetch(`${this.routerUrl}/api/v1/skills/execute`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(request.body),
        });

        if (!response.ok) {
          reply.code(response.status);
          return { error: response.statusText };
        }

        return response.json();
      }
    );

    this.app.get<{ Params: { name: string } }>(
      '/api/skills/:name',
      async (request: FastifyRequest<{ Params: { name: string } }>, reply: FastifyReply) => {
        const { name } = request.params;
        const response = await fetch(`${this.routerUrl}/api/v1/skills/${name}`);

        if (!response.ok) {
          reply.code(response.status);
          return { error: response.statusText };
        }

        return response.json();
      }
    );

    try {
      await this.app.listen({ port: this.port, host: '127.0.0.1' });
      console.log(`REST API adapter started on port ${this.port}`);
    } catch (err) {
      this.app.log.error(err);
      throw err;
    }
  }

  async stop(): Promise<void> {
    if (this.app) {
      await this.app.close();
    }
  }
}
