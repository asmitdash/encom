/**
 * Encom Skill SDK.
 *
 * The surface a skill author imports. The host (encom-core) injects the
 * implementations of `llm`, `secret`, `memory`, etc. at runtime via the
 * sandbox bridge — Phase 0 ships only the type surface and a `skill()`
 * helper that turns a skill module into the shape the runner expects.
 */

export interface SkillContext {
  args: Record<string, unknown>;
}

export interface SkillResult {
  ok: boolean;
  text?: string;
  data?: unknown;
}

export interface SkillModule {
  run(ctx: SkillContext): Promise<SkillResult | string | unknown>;
}

export function skill(mod: SkillModule): SkillModule {
  return mod;
}

export interface CompletionOptions {
  system?: string;
  user: string;
  model?: string;
  maxTokens?: number;
  temperature?: number;
}

export interface CompletionResult {
  text: string;
  model: string;
}

/**
 * The host injects the real implementation. Importing this module outside
 * the Encom sandbox throws to make the failure obvious instead of silently
 * returning fake data.
 */
function notInSandbox(name: string): never {
  throw new Error(
    `[encom] '${name}' was called outside the Encom sandbox. Run this skill via the encom daemon.`,
  );
}

export const llm = {
  complete(_opts: CompletionOptions): Promise<CompletionResult> {
    return notInSandbox("llm.complete");
  },
};

export function secret(_name: string): Promise<string> {
  return notInSandbox("secret");
}

export const memory = {
  put(_namespace: string, _key: string, _value: string): Promise<void> {
    return notInSandbox("memory.put");
  },
  get(_namespace: string, _key: string): Promise<string | null> {
    return notInSandbox("memory.get");
  },
};
