import { Client, GatewayIntentBits, Events, Message, TaskRequest, TaskResponse } from './types.js';
import { BaseAdapter } from '../base.js';

export class DiscordAdapter extends BaseAdapter {
  private client: Client;
  private token: string;

  constructor(
    token: string,
    onTask: (task: TaskRequest) => Promise<void>
  ) {
    super('discord', onTask);
    this.token = token;
    this.client = new Client({
      intents: [
        GatewayIntentBits.Guilds,
        GatewayIntentBits.GuildMessages,
        GatewayIntentBits.MessageContent,
      ],
    });
  }

  async start(): Promise<void> {
    this.client.on(Events.MessageCreate, async (message: Message) => {
      if (message.author.bot) return;
      if (!message.content) return;

      const task = this.createTaskRequest(
        message.id,
        message.author.id,
        message.content,
        message.createdAt,
        message.threadId || undefined
      );

      await this.onTask(task);
    });

    this.client.on(Events.Error, (error: Error) => {
      this.logger.error({ error }, 'Discord client error');
    });

    await this.client.login(this.token);
  }

  async stop(): Promise<void> {
    this.client.destroy();
  }
}
