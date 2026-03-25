import { TaskRequest, TaskResponse } from '../../types.js';
import { BaseAdapter } from '../base.js';

export interface WhatsAppMessage {
  MessageSid: string;
  From: string;
  To: string;
  Body: string;
}

export class WhatsAppAdapter extends BaseAdapter {
  private accountSid: string;
  private authToken: string;
  private fromNumber: string;

  constructor(
    accountSid: string,
    authToken: string,
    fromNumber: string,
    onTask: (task: TaskRequest) => Promise<void>
  ) {
    super('whatsapp', onTask);
    this.accountSid = accountSid;
    this.authToken = authToken;
    this.fromNumber = fromNumber;
  }

  async start(): Promise<void> {
    // WhatsApp uses webhooks - no persistent connection needed
  }

  async stop(): Promise<void> {
    // Cleanup if needed
  }

  async handleIncomingMessage(message: WhatsAppMessage): Promise<void> {
    const task = this.createTaskRequest(
      message.MessageSid,
      message.From,
      message.Body,
      new Date()
    );
    await this.onTask(task);
  }

  async send(response: TaskResponse): Promise<void> {
    const to = response.messageId;
    const url = `https://api.twilio.com/2010-04-01/Accounts/${this.accountSid}/Messages.json`;
    
    const auth = Buffer.from(`${this.accountSid}:${this.authToken}`).toString('base64');
    
    const formData = new URLSearchParams();
    formData.append('To', `whatsapp:${to}`);
    formData.append('From', `whatsapp:${this.fromNumber}`);
    formData.append('Body', response.content);

    try {
      const result = await fetch(url, {
        method: 'POST',
        headers: {
          'Authorization': `Basic ${auth}`,
          'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: formData.toString(),
      });
      
      if (!result.ok) {
        const errorText = await result.text();
        this.logger.error({ status: result.status, error: errorText }, 'Failed to send WhatsApp message');
      }
    } catch (error) {
      this.logger.error({ error }, 'WhatsApp send error');
    }
  }
}
