---
inclusion: manual
---

# How we work with Kiro on NecroCore

Use `#kiro-workflow` in chat to load this.

1. New feature: write `.kiro/specs/NN-name/requirements.md` in EARS form, then
   `design.md`, then `tasks.md`. One task is one reviewable change.
2. Pick an agent from `.kiro/agents/`:
   - `exorcist` heals findings and edits the engine crates.
   - `prover` runs proofs and reports parity. It has no file-write tool.
   - `scribe` writes the README and deliverables in the judges' voice.
   - `medium` hot-patches running PhantomArcade games through necro-mcp.
3. Hooks do the checking. `crate-tests` after each Rust edit, `judges-voice`
   after each write to judge-facing text, `guard-benchmarks` before any write,
   `task-proof` when a spec task ends, `on-parity-verify` when the agent stops.
4. Every hook firing is logged to `benchmarks/hook-firings.log`. Lines marked
   `by_hand` were run by us to test the hook, not fired by Kiro.
