import { Telegraf, Context, ChannelAdapter, TaskRequest, TaskResponse } from './types.js';

export class TelegramAdapter implements ChannelAdapter {
  readonly channel = 'telegram' as const;
  private bot: Telegraf<Context>;
  private onTask: (task: TaskRequest) => Promise<void>;

  constructor(
    token: string,
    onTask: (task: TaskRequest) => Promise<void>
  ) {
    this.bot = new Telegraf(token);
    this.onTask = onTask;
  }

  async start(): Promise<void> {
    this.bot.on('message', async (ctx: Context) => {
      const message = ctx.message;
      if (!message || !('text' in message)) return;
      if (!message.from) return;

      const task: TaskRequest = {
        messageId: String(message.message_id),
        channel: 'telegram',
        threadId: 'chat' in ctx && 'id' in ctx.chat ? String(ctx.chat.id) : undefined,
        author: String(message.from.id),
        content: message.text,
        timestamp: new Date(),
      };

      await this.onTask(task);
    });

    this.bot.on('callback_query', async (ctx: Context) => {
      await ctx.answerCallbackQuery();
    });

    await this.bot.launch();
  }

  async stop(): Promise<void> {
    this.bot.stop();
  }

  async send(response: TaskResponse): Promise<void> {
  }
}
