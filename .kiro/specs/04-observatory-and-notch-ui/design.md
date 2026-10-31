# Design: Observatory and notch UI (Phase 2)

## Shape
A native macOS app (Swift, AppKit for the notch strip, SwiftUI for the cockpit)
that talks to the Rust crates through the `necro-cli` binary and the `necro-mcp`
server. No Electron: disk is tight on the build machine and the notch strip must
be a borderless always-on-top window.

```
Observatory (SwiftUI) --JSON-RPC over stdio--> necro-cli mcp
     |                                           |-- necro_autopsy
     |                                           |-- necro_parity
     +--notch strip (AppKit)--------------------->-- necro_hotpatch
```

## Correctness properties
- The UI never computes a number; it displays what the MCP tools return, with
  the tool call recorded next to it.
- A hot patch from the notch is the same `PatchTable::apply` the prover uses, so
  what the user sees is what the parity run proved.

## Design language
Dark first. Emerald accent (#10b981) for proven parity, crimson (#f43f5e) for
divergences, amber for warnings. Respect reduced motion. Readable at 390 px.

## Open questions for Phase 2
- Whether to render the CFG with a force layout or a layered (Sugiyama) layout.
- Whether the notch strip should also show the COBOL fuel counter during a run.
