export interface TaskRequest {
  messageId: string;
  channel: Channel;
  threadId?: string;
  author: string;
  content: string;
  attachments?: Attachment[];
  timestamp: Date;
}

export interface Attachment {
  filename: string;
  mimeType: string;
  url?: string;
  content?: string;
}

export type Channel = 'slack' | 'discord' | 'telegram' | 'whatsapp' | 'email' | 'api';

export interface TaskResponse {
  messageId: string;
  content: string;
  channel: Channel;
  threadId?: string;
}

export interface ChannelAdapter {
  readonly channel: Channel;
  start(): Promise<void>;
  stop(): Promise<void>;
  send(response: TaskResponse): Promise<void>;
}

export interface DiscordMessage {
  id: string;
  channelId: string;
  author: { id: string; bot: boolean };
  content: string;
  timestamp: string;
  createdAt: Date;
  threadId?: string;
}

export interface DiscordClient {
  on(event: 'messageCreate', callback: (message: DiscordMessage) => Promise<void>): void;
  on(event: 'error', callback: (error: Error) => void): void;
  login(token: string): Promise<void>;
  destroy(): void;
}

export interface GatewayIntentBits {
  Guilds: number;
  GuildMessages: number;
  MessageContent: number;
}

export interface Discord {
  Client: new (options: { intents: number[] }) => DiscordClient;
  GatewayIntentBits: GatewayIntentBits;
  Events: {
    MessageCreate: string;
    Error: string;
  };
}

export interface TelegramMessage {
  message_id: number;
  from?: { id: number };
  text?: string;
  chat: { id: number };
}

export interface TelegramContext {
  message?: TelegramMessage;
  callback_query?: { data?: string };
  chat?: { id: number };
  answerCallbackQuery(): Promise<void>;
}

export interface Telegraf<T extends TelegramContext> {
  on(event: string, callback: (ctx: T) => Promise<void>): void;
  launch(): Promise<void>;
  stop(): void;
}
