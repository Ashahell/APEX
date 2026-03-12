import { Telegraf, Context, TaskRequest, TaskResponse } from './types.js';
import { BaseAdapter } from '../base.js';

export class TelegramAdapter extends BaseAdapter {
  private bot: Telegraf<Context>;

  constructor(
    token: string,
    onTask: (task: TaskRequest) => Promise<void>
  ) {
    super('telegram', onTask);
    this.bot = new Telegraf(token);
  }

  async start(): Promise<void> {
    this.bot.on('message', async (ctx: Context) => {
      const message = ctx.message;
      if (!message || !('text' in message)) return;
      if (!message.from) return;

      const task = this.createTaskRequest(
        String(message.message_id),
        String(message.from.id),
        message.text,
        new Date(),
        'chat' in ctx && 'id' in ctx.chat ? String(ctx.chat.id) : undefined
      );

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
}
