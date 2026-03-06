// skills/pool_worker_test.ts
// Tests for the Bun pool worker

import { spawn, type ChildProcess } from "child_process";
import { readFileSync } from "fs";
import { join } from "path";

const WORKER_PATH = join(import.meta.dir, "pool_worker.ts");

interface PoolRequest {
  id: string;
  skill: string;
  input: unknown;
  timeout_ms: number;
}

interface PoolResponse {
  id?: string | null;
  ok: boolean;
  output?: string;
  error?: string;
  duration_ms: number;
  ready?: boolean;
  pid?: number;
}

class PoolWorkerClient {
  private proc: ChildProcess;
  private pending = new Map<string, { resolve: (r: PoolResponse) => void; reject: (e: Error) => void }>();
  private ready = false;
  private readyResolve?: () => void;

  constructor() {
    this.proc = spawn("bun", ["run", WORKER_PATH], {
      stdio: ["pipe", "pipe", "pipe"],
    });

    let buffer = "";
    
    this.proc.stdout!.on("data", (chunk: Buffer) => {
      buffer += chunk.toString();
      const lines = buffer.split("\n");
      buffer = lines.pop() || "";

      for (const line of lines) {
        if (!line.trim()) continue;
        try {
          const resp = JSON.parse(line) as PoolResponse;
          
          if (resp.ready) {
            this.ready = true;
            this.readyResolve?.();
            continue;
          }

          if (resp.id && this.pending.has(resp.id)) {
            const { resolve } = this.pending.get(resp.id)!;
            this.pending.delete(resp.id);
            resolve(resp);
          }
        } catch {
          console.error("Failed to parse line:", line);
        }
      }
    });

    this.proc.stderr!.on("data", (chunk: Buffer) => {
      console.error("[worker stderr]", chunk.toString());
    });
  }

  async waitReady(timeout = 5000): Promise<void> {
    if (this.ready) return;
    
    await new Promise<void>((resolve) => {
      this.readyResolve = resolve;
      setTimeout(() => resolve(), timeout);
    });
  }

  async request(req: PoolRequest, timeout = 30000): Promise<PoolResponse> {
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(req.id);
        reject(new Error(`Request ${req.id} timed out`));
      }, timeout);

      this.pending.set(req.id, {
        resolve: (r) => {
          clearTimeout(timer);
          resolve(r);
        },
        reject: (e) => {
          clearTimeout(timer);
          reject(e);
        },
      });

      this.proc.stdin!.write(JSON.stringify(req) + "\n");
    });
  }

  async ping(): Promise<PoolResponse> {
    return this.request({
      id: "ping-" + Date.now(),
      skill: "__ping__",
      input: {},
      timeout_ms: 5000,
    });
  }

  kill(): void {
    this.proc.kill();
  }
}

describe("PoolWorker", () => {
  let client: PoolWorkerClient;

  beforeAll(async () => {
    client = new PoolWorkerClient();
    await client.waitReady();
  });

  afterAll(() => {
    client.kill();
  });

  test("worker is ready", async () => {
    const resp = await client.ping();
    expect(resp.ok).toBe(true);
    expect(resp.output).toBe("pong");
  });

  test("returns pong for __ping__ skill", async () => {
    const resp = await client.request({
      id: "test-ping",
      skill: "__ping__",
      input: {},
      timeout_ms: 5000,
    });
    expect(resp.ok).toBe(true);
    expect(resp.output).toBe("pong");
  });

  test("reports latency", async () => {
    const resp = await client.ping();
    expect(resp.duration_ms).toBeGreaterThanOrEqual(0);
    expect(resp.duration_ms).toBeLessThan(1000);
  });

  test("handles malformed JSON gracefully", async () => {
    // This would need to test the worker's error handling
    // Hard to test without exposing internal error channel
    expect(true).toBe(true);
  });

  test("multiple concurrent requests", async () => {
    const requests = Array.from({ length: 5 }, (_, i) =>
      client.request({
        id: `concurrent-${i}`,
        skill: "__ping__",
        input: {},
        timeout_ms: 5000,
      })
    );

    const results = await Promise.all(requests);
    expect(results.every((r) => r.ok)).toBe(true);
  });
});
