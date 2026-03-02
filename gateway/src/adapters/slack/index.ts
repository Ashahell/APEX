import { App, SlackEvent, SayFn } from '@slack/bolt';
import { ChannelAdapter, TaskRequest, TaskResponse } from '../types.js';

export class SlackAdapter implements ChannelAdapter {
  readonly channel = 'slack' as const;
  private app: App;
  private onTask: (task: TaskRequest) => Promise<void>;

  constructor(
    token: string,
    signingSecret: string,
    onTask: (task: TaskRequest) => Promise<void>
  ) {
    this.app = new App({
      token,
      signingSecret,
    });
    this.onTask = onTask;
  }

  async start(): Promise<void> {
    this.app.message(async ({ message, say }) => {
      if (this.isUserMessage(message)) {
        const task: TaskRequest = {
          messageId: message.ts,
          channel: 'slack',
          threadId: message.thread_ts,
          author: message.user,
          content: message.text || '',
          timestamp: new Date(Number(message.ts) * 1000),
        };
        
        await this.onTask(task);
      }
    });

    await this.app.start(3002);
    console.log('Slack adapter started on port 3002');
  }

  async stop(): Promise<void> {
    await this.app.stop();
  }

  async send(response: TaskResponse): Promise<void> {
    console.log('Slack send:', response);
  }

  private isUserMessage(message: SlackEvent): message is SlackEvent & { text: string; user: string; ts: string } {
    return 'user' in message && 'text' in message && 'ts' in message;
  }
}
