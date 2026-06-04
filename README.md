# Encom

> The agent runtime that runs.

Encom is an open-source AI agent framework: a small, fast, embeddable runtime for agents that **actually do work** — read your inbox, drive the browser, run cron jobs, manage files, hold persistent memory, talk to any model, and ship as a single binary.

It's the layer underneath **[Encom OS](https://github.com/asmitdash/encom-os)** — a Linux distribution where Encom is wired into the OS itself, not bolted on as an app. Encom alone runs anywhere: macOS, Linux, Windows, Docker, embedded.

```
┌─────────────────────────────────────────────────┐
│  Skills (TypeScript)  ←  what the agent can do  │
├─────────────────────────────────────────────────┤
│  Encom Core (Rust daemon)                       │
│  · Memory   · Scheduler   · Sandbox   · IPC     │
├─────────────────────────────────────────────────┤
│  Model Adapters                                 │
│  OpenAI · Anthropic · xAI · Gemini · Mistral    │
│  Cohere · Groq · DeepSeek · Ollama (local)      │
└─────────────────────────────────────────────────┘
```

## Why another one

OpenClaw, Hermes, Letta, Mastra, and the rest are valuable but each makes one of three trades that Encom refuses:

1. **JS-only runtimes** — Node-based daemons can't ship in an OS image without dragging the whole runtime. Encom's daemon is Rust, single static binary, < 10 MB.
2. **Cloud-first defaults** — telemetry, hosted skills, account walls. Encom is local-first; the cloud is opt-in for individual model adapters only.
3. **Skill author experience as an afterthought** — Encom skills are TypeScript with a typed manifest, run in a sandboxed V8 isolate, and reload on save.

If you want a framework you can drop into a custom OS, an embedded device, a server, or your laptop without renting anything from anyone — that's Encom.

## Quick start (local dev)

```bash
# install
curl -fsSL https://encom.sh/install | bash    # planned; for now:
cargo install encom-cli                        # Rust toolchain required

# run the daemon
encom daemon

# in another shell
encom chat
> summarize my unread github notifications
```

## Configuration

Encom reads `~/.config/encom/config.toml`:

```toml
[model]
default = "anthropic"

[model.anthropic]
api_key_env = "ANTHROPIC_API_KEY"
model = "claude-opus-4-7"

[model.openai]
api_key_env = "OPENAI_API_KEY"
model = "gpt-5"

[model.ollama]
host = "http://localhost:11434"
model = "llama3.3:70b"

[memory]
backend = "sqlite"
path = "~/.local/share/encom/memory.db"

[skills]
dirs = ["~/.config/encom/skills", "/usr/share/encom/skills"]
```

## Writing a skill

A skill is a directory with a manifest and an entry point:

```
~/.config/encom/skills/inbox-triage/
├── encom.toml
└── index.ts
```

```toml
# encom.toml
name = "inbox-triage"
description = "Triage unread email and propose replies"
version = "0.1.0"

[permissions]
network = ["api.gmail.com"]
fs = []
secrets = ["GMAIL_TOKEN"]
```

```typescript
// index.ts
import { skill, llm, secret } from "@asmitdash/encom";

export default skill({
  async run({ args }) {
    const token = await secret("GMAIL_TOKEN");
    const unread = await fetch("https://api.gmail.com/...", { headers: { Authorization: `Bearer ${token}` } });
    const triage = await llm.complete({
      system: "Sort these emails into urgent / reply-later / archive.",
      user: JSON.stringify(await unread.json()),
    });
    return triage;
  },
});
```

## Architecture

Encom is two pieces:

- **`encom-core`** — Rust daemon. Owns memory (SQLite + vector index), scheduling (tokio), the model-adapter trait, the skill-runner sandbox, and a Unix-socket / named-pipe IPC surface. Single static binary.
- **`@asmitdash/encom`** — TypeScript SDK. The skill author's surface area: `llm.complete`, `secret()`, `fs.read()`, `http.fetch()`, `memory.recall()`. Runs inside a V8 isolate the daemon spawns per skill invocation.

Models are pluggable via the `ModelAdapter` trait in [crates/encom-models](crates/encom-models). Adding a new provider is one file.

## Roadmap

- [x] Phase 0 — repo scaffold, README, CI
- [ ] Phase 1 — Rust daemon: memory, scheduler, IPC, model adapters
- [ ] Phase 2 — TS Skills SDK + 4 example skills
- [ ] Phase 3 — CLI polish, daemon installers (systemd unit, launchd plist, Windows service)
- [ ] Phase 4 — Skill marketplace + signing
- [ ] Phase 5 — Encom OS integration hooks (mounted as the OS agent layer)

## License

MIT. See [LICENSE](LICENSE).

## Author

[asmitdash](https://github.com/asmitdash) — building [Encom OS](https://github.com/asmitdash/encom-os) on top of this.
