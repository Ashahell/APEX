// skills/pool_worker.ts
// Runs as a persistent Bun process.
// Reads newline-delimited JSON from stdin, executes skills, writes responses to stdout.
// Multiple requests can be in-flight concurrently — responses are written as they complete.

import { createInterface } from "readline";

const skillCache = new Map<string, SkillModule>();

// B1: Skill tier registry - maps skill names to their required tiers
const SKILL_TIERS: Record<string, string> = {
  "code.review": "T0",
  "docs.read": "T0",
  "deps.check": "T0",
  "repo.search": "T0",
  "code.generate": "T1",
  "code.refactor": "T1",
  "code.document": "T1",
  "api.design": "T1",
  "ci.configure": "T1",
  "db.schema": "T1",
  "copy.generate": "T1",
  "script.draft": "T1",
  "script.outline": "T1",
  "seo.optimize": "T1",
  "git.commit": "T2",
  "code.test": "T2",
  "db.migrate": "T2",
  "docker.build": "T2",
  "video.edit": "T2",
  "video.generate": "T2",
  "music.generate": "T2",
  "music.extend": "T2",
  "music.remix": "T2",
  "shell.execute": "T3",
  "file.delete": "T3",
  "git.force_push": "T3",
  "db.drop": "T3",
  "aws.lambda": "T3",
  "deploy.kubectl": "T3",
};

const TIER_LEVELS: Record<string, number> = {
  T0: 0,
  T1: 1,
  T2: 2,
  T3: 3,
};

function tierPermits(permitted: string, required: string): boolean {
  const permittedLevel = TIER_LEVELS[permitted] ?? 0;
  const requiredLevel = TIER_LEVELS[required] ?? 0;
  return permittedLevel >= requiredLevel;
}

interface SkillModule {
  execute: (input: unknown) => Promise<SkillResult>;
  healthCheck?: () => Promise<boolean>;
}

interface SkillResult {
  success: boolean;
  output?: string;
  error?: string;
  artifacts?: Array<{ path: string; content: string }>;
}

interface PoolRequest {
  id: string;
  skill: string;
  input: unknown;
  timeout_ms: number;
  permitted_tier?: string;  // B1: tier passed from Router
}

interface PoolResponse {
  id: string;
  ok: boolean;
  output?: string;
  error?: string;
  duration_ms: number;
}

async function loadSkill(name: string): Promise<SkillModule> {
  if (skillCache.has(name)) {
    return skillCache.get(name)!;
  }

  const skillsDir = process.env.APEX_SKILLS_DIR || "./skills";
  const skillPath = `${skillsDir}/${name}/src/index.ts`;

  try {
    const mod = await import(skillPath);

    if (typeof mod.execute !== "function") {
      throw new Error(`Skill ${name} does not export an execute() function`);
    }

    skillCache.set(name, mod as SkillModule);
    return mod as SkillModule;
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : String(err);
    throw new Error(`Failed to load skill ${name}: ${message}`);
  }
}

async function handleRequest(req: PoolRequest): Promise<PoolResponse> {
  const start = performance.now();

  // B2: Skill cache invalidation
  if (req.skill === "__cache_bust__") {
    const target = req.input as { skill?: string } | undefined;
    if (target?.skill) {
      skillCache.delete(target.skill);
    } else {
      skillCache.clear();
    }
    return {
      id: req.id,
      ok: true,
      output: "cache cleared",
      duration_ms: Math.round(performance.now() - start),
    };
  }

  // B1: Validate tier permissions before executing
  // Default to lowest trust (T0) if not provided, fail closed for unknown skills (T3)
  const permittedTier = req.permitted_tier || "T0";
  const declaredTier = SKILL_TIERS[req.skill] || "T3";
  
  if (!tierPermits(permittedTier, declaredTier)) {
    return {
      id: req.id,
      ok: false,
      error: `Tier violation: ${req.skill} requires ${declaredTier}, but request has ${permittedTier}`,
      duration_ms: Math.round(performance.now() - start),
    };
  }

  const timeoutPromise = new Promise<never>((_, reject) =>
    setTimeout(() => reject(new Error("timeout")), req.timeout_ms)
  );

  try {
    const skill = await loadSkill(req.skill);

    const resultPromise = skill.execute(req.input).then((result) => {
      if (!result.success) {
        throw new Error(result.error ?? "Skill returned success: false");
      }
      return result.output ?? "";
    });

    const output = await Promise.race([resultPromise, timeoutPromise]);

    return {
      id: req.id,
      ok: true,
      output,
      duration_ms: Math.round(performance.now() - start),
    };
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : String(err);
    return {
      id: req.id,
      ok: false,
      error: message,
      duration_ms: Math.round(performance.now() - start),
    };
  }
}

function writeResponse(response: PoolResponse | object): void {
  process.stdout.write(JSON.stringify(response) + "\n");
}

writeResponse({ ready: true, pid: process.pid, bun_version: Bun.version });

const rl = createInterface({ input: process.stdin, crlfDelay: Infinity });

rl.on("line", (line: string) => {
  if (!line.trim()) return;

  let parsed: unknown;
  try {
    parsed = JSON.parse(line);
  } catch {
    writeResponse({ id: null, ok: false, error: "Malformed JSON request" });
    return;
  }

  if ((parsed as Record<string, unknown>).ping === true) {
    writeResponse({ pong: true, pid: process.pid });
    return;
  }

  if ((parsed as Record<string, unknown>).skill === "__ping__") {
    writeResponse({ id: (parsed as PoolRequest).id, ok: true, output: "pong", duration_ms: 0 });
    return;
  }

  const req = parsed as PoolRequest;
  handleRequest(req).then(writeResponse);
});

rl.on("close", () => {
  process.exit(0);
});

process.on("unhandledRejection", (reason) => {
  process.stderr.write(`[pool_worker] unhandledRejection: ${reason}\n`);
});
