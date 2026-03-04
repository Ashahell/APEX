import { ChannelAdapter, TaskRequest, TaskResponse } from '../../types.js';

export interface WhatsAppMessage {
  MessageSid: string;
  From: string;
  To: string;
  Body: string;
}

export class WhatsAppAdapter implements ChannelAdapter {
  readonly channel = 'whatsapp' as const;
  private accountSid: string;
  private authToken: string;
  private fromNumber: string;
  private onTask: (task: TaskRequest) => Promise<void>;

  constructor(
    accountSid: string,
    authToken: string,
    fromNumber: string,
    onTask: (task: TaskRequest) => Promise<void>
  ) {
    this.accountSid = accountSid;
    this.authToken = authToken;
    this.fromNumber = fromNumber;
    this.onTask = onTask;
  }

  async start(): Promise<void> {
    console.log('WhatsApp adapter started (webhook mode)');
  }

  async stop(): Promise<void> {
    console.log('WhatsApp adapter stopped');
  }

  async handleIncomingMessage(message: WhatsAppMessage): Promise<void> {
    const task: TaskRequest = {
      messageId: message.MessageSid,
      channel: 'whatsapp',
      author: message.From,
      content: message.Body,
      timestamp: new Date(),
    };

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
        console.error('Failed to send WhatsApp message:', await result.text());
      }
    } catch (error) {
      console.error('WhatsApp send error:', error);
    }
  }
}
