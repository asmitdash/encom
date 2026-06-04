# @asmitdash/encom

TypeScript SDK for authoring [Encom](https://github.com/asmitdash/encom) skills.

```ts
import { skill, llm } from "@asmitdash/encom";

export default skill({
  async run({ args }) {
    const r = await llm.complete({
      system: "You are concise.",
      user: String(args.prompt ?? "hello"),
    });
    return r.text;
  },
});
```

See the [main README](https://github.com/asmitdash/encom#writing-a-skill) for the full skill model, manifest format, and permission system.

MIT.
