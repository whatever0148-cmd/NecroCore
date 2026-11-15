---
inclusion: fileMatch
fileMatchPattern: ["crates/necro-prover/**", "crates/*/src/wat.rs", "necro-cli/src/main.rs"]
---

# The prover and the WebAssembly side

- Modules are emitted as WebAssembly text and compiled by wasmtime with Cranelift
  at run time. No wasm-bindgen, no JavaScript glue on the proving path.
- The prover runs the reference interpreter and the compiled module side by side
  on seeded vectors (default seed 7) and compares every observable: variables,
  display bytes, registers, timers, the final status.
- A divergence is a bug in the lifter or the interpreter, never noise. Keep the
  first divergent vector in the report so it can be replayed.
- Fuel bounds both sides. FuelExhausted on a Zombie program is an expected
  outcome and counts as a match when both sides run out at the same step.
