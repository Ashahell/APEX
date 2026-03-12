import { TaskRequest, TaskResponse } from '../../types.js';
import { BaseAdapter } from '../base.js';

// Use require to avoid ES module issues
const nodemailer = require('nodemailer');
const ImapClient = require('imapflow');
const simpleParser = require('mailparser');

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

export interface EmailAdapterConfig {
  smtpHost: string;
  smtpPort: number;
  smtpUser: string;
  smtpPass: string;
  imapHost?: string;
  imapPort?: number;
  imapUser?: string;
  imapPass?: string;
  fromAddress?: string;
}

export class EmailAdapter extends BaseAdapter {
  private config: EmailAdapterConfig;
  private smtpTransport: any = null;
  private imapClient: any = null;

  constructor(config: EmailAdapterConfig, onTask: (task: TaskRequest) => Promise<void>) {
    super('email', onTask);
    this.config = config;
  }

  async start(): Promise<void> {
    // Start IMAP client if configured
    if (this.config.imapHost && this.config.imapUser && this.config.imapPass) {
      await this.connectImap();
    }
  }

  async stop(): Promise<void> {
    if (this.imapClient) {
      await this.imapClient.logout();
    }
    if (this.smtpTransport) {
      await this.smtpTransport.close();
    }
  }

  private async connectImap(): Promise<void> {
    this.imapClient = new ImapClient.ImapClient({
      host: this.config.imapHost!,
      port: this.config.imapPort || 993,
      auth: {
        user: this.config.imapUser!,
        pass: this.config.imapPass!,
      },
      secure: true,
    });

    await this.imapClient.connect();
    await this.imapClient.mailboxOpen('INBOX');
    
    // Listen for new emails
    this.imapClient.on('mail', async () => {
      await this.checkForNewEmails();
    });
  }

  private async checkForNewEmails(): Promise<void> {
    if (!this.imapClient) return;

    const messages = await this.imapClient.search(
      { unseen: true },
      { limit: 10 }
    );

    for (const seq of messages) {
      const message = await this.imapClient.fetchOne(seq, {
        source: true,
        uid: true,
      });
      
      const parsed = await simpleParser(message.source);
      
      await this.handleIncomingEmail({
        from: parsed.from?.value[0]?.address || 'unknown',
        to: this.config.fromAddress || '',
        subject: parsed.subject || '',
        text: parsed.text || '',
        html: parsed.html || undefined,
      });
      
      await this.imapClient.flag(seq, ['\\Seen'], true);
    }
  }

  async handleIncomingEmail(message: EmailMessage): Promise<void> {
    const task = this.createTaskRequest(
      `${message.from}-${Date.now()}`,
      message.from,
      `[${message.subject}] ${message.text}`,
      new Date()
    );

    await this.onTask(task);
  }

  async sendEmail(to: string, subject: string, text: string, html?: string): Promise<void> {
    if (!this.smtpTransport) {
      this.smtpTransport = nodemailer.createTransport({
        host: this.config.smtpHost,
        port: this.config.smtpPort,
        auth: {
          user: this.config.smtpUser,
          pass: this.config.smtpPass,
        },
      });
    }

    await this.smtpTransport.sendMail({
      from: this.config.fromAddress || this.config.smtpUser,
      to,
      subject,
      text,
      html,
    });
  }

  async send(response: TaskResponse): Promise<void> {
    // Send response back via email
    // The message ID should be the email address to reply to
    const to = response.messageId;
    await this.sendEmail(to, `Re: APEX Task`, response.content);
  }
}