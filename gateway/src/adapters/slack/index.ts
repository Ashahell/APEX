import { App, SlackEvent } from '@slack/bolt';
import { TaskRequest, TaskResponse } from '../types.js';
import { BaseAdapter } from '../base.js';

export class SlackAdapter extends BaseAdapter {
  private app: App;

  constructor(
    token: string,
    signingSecret: string,
    onTask: (task: TaskRequest) => Promise<void>
  ) {
    super('slack', onTask);
    this.app = new App({
      token,
      signingSecret,
    });
  }

  async start(): Promise<void> {
    this.app.message(async ({ message }) => {
      if (this.isUserMessage(message)) {
        const task = this.createTaskRequest(
          message.ts,
          message.user,
          message.text || '',
          new Date(Number(message.ts) * 1000),
          message.thread_ts
        );
        
        await this.onTask(task);
      }
    });

    await this.app.start(3002);
  }

  async stop(): Promise<void> {
    await this.app.stop();
  }

  private isUserMessage(message: SlackEvent): message is SlackEvent & { text: string; user: string; ts: string } {
    return 'user' in message && 'text' in message && 'ts' in message;
  }
}
