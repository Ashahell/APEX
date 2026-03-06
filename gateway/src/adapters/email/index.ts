import { ChannelAdapter, TaskRequest, TaskResponse } from '../../types.js';

export interface EmailMessage {
  from: string;
  to: string;
  subject: string;
  text: string;
  html?: string;
  attachments?: Array<{
    filename: string;
    contentType: string;
    data: string;
  }>;
}

export class EmailAdapter implements ChannelAdapter {
  readonly channel = 'email' as const;
  private smtpHost: string;
  private smtpPort: number;
  private smtpUser: string;
  private smtpPass: string;
  private onTask: (task: TaskRequest) => Promise<void>;

  constructor(
    smtpHost: string,
    smtpPort: number,
    smtpUser: string,
    smtpPass: string,
    onTask: (task: TaskRequest) => Promise<void>
  ) {
    this.smtpHost = smtpHost;
    this.smtpPort = smtpPort;
    this.smtpUser = smtpUser;
    this.smtpPass = smtpPass;
    this.onTask = onTask;
  }

  async start(): Promise<void> {
  }

  async stop(): Promise<void> {
  }

  async handleIncomingEmail(message: EmailMessage): Promise<void> {
    const task: TaskRequest = {
      messageId: `${message.from}-${Date.now()}`,
      channel: 'email',
      author: message.from,
      content: message.text,
      timestamp: new Date(),
    };

    await this.onTask(task);
  }

  async send(response: TaskResponse): Promise<void> {
  }
}
