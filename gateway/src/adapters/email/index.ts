import { TaskRequest, TaskResponse } from '../../types.js';
import { BaseAdapter } from '../base.js';

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

export class EmailAdapter extends BaseAdapter {
  private smtpHost: string;
  private smtpPort: number;
  private smtpUser: string;
  private smtpPass: string;

  constructor(
    smtpHost: string,
    smtpPort: number,
    smtpUser: string,
    smtpPass: string,
    onTask: (task: TaskRequest) => Promise<void>
  ) {
    super('email', onTask);
    this.smtpHost = smtpHost;
    this.smtpPort = smtpPort;
    this.smtpUser = smtpUser;
    this.smtpPass = smtpPass;
  }

  async start(): Promise<void> {
    // Email adapter doesn't need to start a connection
    // Use handleIncomingEmail() to process received emails
  }

  async stop(): Promise<void> {
    // Cleanup if needed
  }

  async handleIncomingEmail(message: EmailMessage): Promise<void> {
    const task = this.createTaskRequest(
      `${message.from}-${Date.now()}`,
      message.from,
      message.text,
      new Date()
    );

    await this.onTask(task);
  }
}
