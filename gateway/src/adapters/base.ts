import { TaskRequest, TaskResponse, Channel } from '../../types.js';

/**
 * Base adapter class for channel adapters
 * Provides common functionality for all adapters
 */
export abstract class BaseAdapter {
  readonly channel: Channel;
  protected onTask: (task: TaskRequest) => Promise<void>;
  
  constructor(channel: Channel, onTask: (task: TaskRequest) => Promise<void>) {
    this.channel = channel;
    this.onTask = onTask;
  }
  
  /**
   * Start the adapter - must be implemented by subclasses
   */
  abstract start(): Promise<void>;
  
  /**
   * Stop the adapter - must be implemented by subclasses
   */
  abstract stop(): Promise<void>;
  
  /**
   * Send a response to the channel
   * Default implementation does nothing - override to implement
   */
  async send(response: TaskResponse): Promise<void> {
    // Default no-op implementation
    // Subclasses can override to implement actual sending
  }
  
  /**
   * Helper to create a TaskRequest from common fields
   */
  protected createTaskRequest(
    messageId: string,
    author: string,
    content: string,
    timestamp: Date,
    threadId?: string
  ): TaskRequest {
    return {
      messageId,
      channel: this.channel,
      author,
      content,
      timestamp,
      threadId,
    };
  }
}
