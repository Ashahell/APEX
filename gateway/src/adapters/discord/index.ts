import { Client, GatewayIntentBits, Events, Message, ChannelAdapter, TaskRequest, TaskResponse } from './types.js';

export class DiscordAdapter implements ChannelAdapter {
  readonly channel = 'discord' as const;
  private client: Client;
  private onTask: (task: TaskRequest) => Promise<void>;
  private token: string;

  constructor(
    token: string,
    onTask: (task: TaskRequest) => Promise<void>
  ) {
    this.token = token;
    this.onTask = onTask;
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

      const task: TaskRequest = {
        messageId: message.id,
        channel: 'discord',
        threadId: message.threadId || undefined,
        author: message.author.id,
        content: message.content,
        timestamp: message.createdAt,
      };

      await this.onTask(task);
    });

    this.client.on(Events.Error, (error: Error) => {
      console.error('Discord client error:', error);
    });

    await this.client.login(this.token);
    console.log('Discord adapter started');
  }

  async stop(): Promise<void> {
    this.client.destroy();
  }

  async send(response: TaskResponse): Promise<void> {
    console.log('Discord send:', response);
  }
}
