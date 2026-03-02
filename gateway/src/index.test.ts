import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { Gateway, TaskRequest, TaskResponse } from './index';

describe('Gateway', () => {
  describe('TaskRequest', () => {
    it('should have correct shape', () => {
      const request: TaskRequest = {
        content: 'Hello world',
        channel: 'test-channel',
        thread_id: 'thread-123',
        author: 'user1',
      };

      expect(request.content).toBe('Hello world');
      expect(request.channel).toBe('test-channel');
      expect(request.thread_id).toBe('thread-123');
      expect(request.author).toBe('user1');
    });

    it('should allow optional fields', () => {
      const request: TaskRequest = {
        content: 'Hello world',
      };

      expect(request.content).toBe('Hello world');
      expect(request.channel).toBeUndefined();
      expect(request.thread_id).toBeUndefined();
      expect(request.author).toBeUndefined();
    });
  });

  describe('TaskResponse', () => {
    it('should have correct shape', () => {
      const response: TaskResponse = {
        task_id: 'task-123',
        status: 'pending',
        tier: 'instant',
        capability_token: 'tok_abc123',
      };

      expect(response.task_id).toBe('task-123');
      expect(response.status).toBe('pending');
      expect(response.tier).toBe('instant');
      expect(response.capability_token).toBe('tok_abc123');
    });
  });

  describe('Gateway class', () => {
    let gateway: Gateway;

    beforeEach(() => {
      gateway = new Gateway({
        routerUrl: 'http://localhost:3000',
        natsUrl: 'localhost:4222',
      });
    });

    afterEach(async () => {
      await gateway.stop();
    });

    it('should create gateway with default config', () => {
      const defaultGateway = new Gateway();
      expect(defaultGateway).toBeDefined();
    });

    it('should create gateway with custom config', () => {
      const customGateway = new Gateway({
        routerUrl: 'http://custom:4000',
        port: 4001,
      });
      expect(customGateway).toBeDefined();
    });

    it('should register adapter', () => {
      const mockAdapter = {
        start: vi.fn().mockResolvedValue(undefined),
        stop: vi.fn().mockResolvedValue(undefined),
      };

      gateway.registerAdapter('test', mockAdapter);
    });
  });

  describe('Task serialization', () => {
    it('should serialize TaskRequest to JSON', () => {
      const request: TaskRequest = {
        content: 'Test task',
        channel: 'test',
      };

      const json = JSON.stringify(request);
      const parsed = JSON.parse(json);

      expect(parsed.content).toBe('Test task');
      expect(parsed.channel).toBe('test');
    });

    it('should deserialize TaskResponse from JSON', () => {
      const json = JSON.stringify({
        task_id: 'abc123',
        status: 'completed',
        tier: 'deep',
        capability_token: 'tok_xyz',
      });

      const response: TaskResponse = JSON.parse(json);

      expect(response.task_id).toBe('abc123');
      expect(response.status).toBe('completed');
      expect(response.tier).toBe('deep');
      expect(response.capability_token).toBe('tok_xyz');
    });
  });
});
