# Design: Kiro guardrails

- One script, `scripts/hooks/hook.py`, with one subcommand per job. Kiro pipes the
  event JSON on stdin (`hook_event_name`, `tool_name`, `tool_input.path`). Exit 0
  adds stdout to the agent's context. Any other exit sends stderr to the agent and,
  for PreToolUse, blocks the tool.
- Hooks in `.kiro/hooks/`: `guard-benchmarks` (PreToolUse, write),
  `judges-voice`, `sync-hosted-page`, `crate-tests` (PostToolUse, write),
  `session-brief` (SessionStart), `task-proof` (PostTaskExecution), plus the
  three Phase 1 hooks.
- Steering: `proof-before-trust` (always), `voice`, `cobol-subset`,
  `chip8-semantics`, `observatory-design`, `prover-and-wasm` (fileMatch),
  `kiro-workflow` (manual).
- Agents in `.kiro/agents/`: tools and `permissions.rules` limit each one.
  `prover` and `medium` cannot write files. `scribe` can write only README.md and
  `deliverables/`.
- The voice check skips fenced code, inline code and link targets, so commands
  and URLs are not flagged.
