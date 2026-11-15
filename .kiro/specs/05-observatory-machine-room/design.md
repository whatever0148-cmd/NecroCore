# Design: Observatory machine-room redesign

- One file, `crates/necro-daemon/src/ui.html`, served by the daemon and embedded
  into `docs/index.html` by `necro-cli observatory` (data in `EMB`).
- World: warm black ground, amber phosphor for live text and the proof, rust for
  structure, oxide red for live rot, green phosphor only in the arcade cabinet.
  Tokens and rules: DESIGN.md and `.impeccable/design.json`.
- CryptLedger layout: a deck (tools, listing, control-flow graph) and a rail
  (counts, run it, proof, Kiro plan), findings full width below. Under 1020 px
  the deck dissolves (`display: contents`) and the order is listing, rail, graph.
- The graph lays blocks out per line so labels are not cut, and arrows use
  `graph-edge-umber` (#8c6849, 3.71:1).
- Static mode reads `B.autopsies`, `B.bench` and `B.live_trace` from the page.
- Checks: `node cap.mjs` captures at 1440 and 390 px, the finish reviewer, and
  `scripts/hooks/hook.py` for the copy that judges read.
