import { connect, NatsConnection, JetStreamClient } from 'nats';
import Pino from 'pino';
import { randomUUID, createHmac } from 'crypto';

export interface GatewayConfig {
  natsUrl?: string;
  routerUrl?: string;
  port?: number;
  sharedSecret?: string;
}

const DEFAULT_CONFIG: GatewayConfig = {
  natsUrl: process.env.APEX_NATS_URL || 'localhost:4222',
  routerUrl: process.env.APEX_ROUTER_URL || 'http://localhost:3000',
  port: parseInt(process.env.APEX_GATEWAY_PORT || '3001', 10),
  sharedSecret: process.env.APEX_SHARED_SECRET || (() => {
    // Allow dev secret in test/development mode
    if (process.env.NODE_ENV === 'test') {
      return 'dev-secret-change-in-production';
    }
    throw new Error('APEX_SHARED_SECRET environment variable must be set');
  })(),
};

export interface TaskRequest {
  content: string;
  channel?: string;
  thread_id?: string;
  author?: string;
}

export interface TaskResponse {
  task_id: string;
  status: string;
  tier: string;
  capability_token: string;
}

export class Gateway {
  private nc: NatsConnection | null = null;
  private js: JetStreamClient | null = null;
  private config: GatewayConfig;
  private logger: Pino.Logger;
  private adapters: Map<string, unknown> = new Map();

  constructor(config: GatewayConfig = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.logger = Pino({
      name: 'apex-gateway',
      level: process.env.LOG_LEVEL || 'info',
    });
  }

  async start(): Promise<void> {
    this.logger.info('Starting APEX Gateway...');

    try {
      this.nc = await connect({ servers: this.config.natsUrl });
      this.js = this.nc.jetstream();
      this.logger.info({ nats: this.config.natsUrl }, 'Connected to NATS');

      await this.subscribeToTasks();
      this.logger.info('Subscribed to task queue');

    } catch (error) {
      this.logger.error({ error }, 'Failed to start gateway - running in HTTP-only mode');
    }
  }

  async stop(): Promise<void> {
    this.logger.info('Stopping APEX Gateway...');

    for (const [name, adapter] of this.adapters) {
      try {
        const stopFn = (adapter as { stop?: () => Promise<void> }).stop;
        if (stopFn) await stopFn.call(adapter);
        this.logger.info({ adapter: name }, 'Stopped adapter');
      } catch (error) {
        this.logger.error({ error, adapter: name }, 'Failed to stop adapter');
      }
    }

    if (this.nc) {
      this.nc.close();
      this.logger.info('NATS connection closed');
    }
  }

  async createTask(request: TaskRequest): Promise<TaskResponse> {
    const response = await this.signedFetch(`${this.config.routerUrl}/api/v1/tasks`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        content: request.content,
        channel: request.channel,
        thread_id: request.thread_id,
        author: request.author,
      }),
    });

    if (!response.ok) {
      throw new Error(`Failed to create task: ${response.statusText}`);
    }

    return response.json() as Promise<TaskResponse>;
  }

  async getTask(taskId: string): Promise<TaskResponse> {
    const response = await this.signedFetch(`${this.config.routerUrl}/api/v1/tasks/${taskId}`);
    if (!response.ok) {
      throw new Error(`Failed to get task: ${response.statusText}`);
    }
    return response.json() as Promise<TaskResponse>;
  }

  async getMetrics(): Promise<unknown> {
    const response = await this.signedFetch(`${this.config.routerUrl}/api/v1/metrics`);
    if (!response.ok) {
      throw new Error(`Failed to get metrics: ${response.statusText}`);
    }
    return response.json();
  }

  private async subscribeToTasks(): Promise<void> {
    if (!this.js) {
      this.logger.warn('JetStream not available, skipping NATS subscription');
      return;
    }

    const sub = await this.js.subscribe('apex.tasks.inbound', { max: 0 });
    
    (async () => {
      try {
        for await (const msg of sub) {
          try {
            const task = JSON.parse(new TextDecoder().decode(msg.data));
            this.logger.info({ taskId: task.messageId }, 'Received task from NATS');
            
            await this.routeTask(task);
            
            msg.ack();
          } catch (error) {
            this.logger.error({ error }, 'Failed to process task');
            msg.nak();
          }
        }
      } catch (error) {
        this.logger.error({ error }, 'Subscription error');
      }
    })();
  }

  private async routeTask(task: Record<string, unknown>): Promise<void> {
    try {
      const content = String(task.content || '');
      
      const taskResponse = await this.createTask({
        content,
        channel: task.channel as string,
        thread_id: task.thread_id as string,
        author: task.author as string,
      });

      this.logger.info({ 
        taskId: taskResponse.task_id, 
        tier: taskResponse.tier 
      }, 'Task created via router');

      await this.publishResponse({
        messageId: task.messageId,
        taskId: taskResponse.task_id,
        status: taskResponse.status,
        tier: taskResponse.tier,
        capabilityToken: taskResponse.capability_token,
      });
    } catch (error) {
      this.logger.error({ error }, 'Failed to route task');
      throw error;
    }
  }

  private async publishResponse(response: Record<string, unknown>): Promise<void> {
    if (!this.js) {
      this.logger.warn('JetStream not available, skipping response publish');
      return;
    }

    const data = new TextEncoder().encode(JSON.stringify(response));
    this.js.publish('apex.results', data);
  }

  registerAdapter(name: string, adapter: unknown): void {
    this.adapters.set(name, adapter);
  }

  private signRequest(method: string, path: string, body: string): { signature: string; timestamp: number } {
    const timestamp = Math.floor(Date.now() / 1000);
    const message = `${timestamp}${method}${path}${body}`;
    const signature = createHmac('sha256', this.config.sharedSecret!)
      .update(message)
      .digest('hex');
    return { signature, timestamp };
  }

  private async signedFetch(url: string, options: RequestInit = {}): Promise<Response> {
    const method = options.method || 'GET';
    const body = typeof options.body === 'string' ? options.body : JSON.stringify(options.body || {});
    const urlPath = new URL(url).pathname;
    const { signature, timestamp } = this.signRequest(method, urlPath, body);

    const headers = new Headers(options.headers);
    headers.set('X-APEX-Signature', signature);
    headers.set('X-APEX-Timestamp', timestamp.toString());

    return fetch(url, { ...options, headers });
  }
}

async function main() {
  const gateway = new Gateway();
  
  process.on('SIGINT', async () => {
    await gateway.stop();
    process.exit(0);
  });

  await gateway.start();

  const testTask = await gateway.createTask({
    content: 'Hello from gateway!',
    channel: 'api',
    author: 'test',
  });
}

main().catch(console.error);
