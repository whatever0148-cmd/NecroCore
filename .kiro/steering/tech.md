---
inclusion: always
---

# NecroCore: technology

- Language: Rust 2021, stable toolchain from `~/.cargo/bin` (rustc 1.99.1 at the
  time of writing). One Cargo workspace, five library crates and one binary.
- Runtime: `wasmtime` 50 with Cranelift, used as a library (`default-features =
  false`, features `cranelift`, `runtime`, `wat`). Modules are emitted as
  WebAssembly text and compiled at run time.
- Async: `tokio` only in `necro-kiro-bridge`, for the subprocess and the timeout.
- CLI: `clap` derive. Serialization: `serde` + `serde_json`.
- Property-based tests: `proptest` for the fixed-point store rules and the CHIP-8
  arithmetic.
- Kiro: `kiro-cli` 2.34 (`~/.local/bin/kiro-cli`), signed in with a Builder ID.
  The bridge calls `kiro-cli chat --no-interactive --trust-tools=` so the model
  cannot run tools.
- Budget: $0. No paid services, no API keys, no network at test time.

## Conventions
- `cargo test --release --workspace` must pass with 0 failed and 0 ignored.
- `cargo clippy --workspace --all-targets` warnings are fixed, not allowed.
- Arithmetic that both the interpreter and the WebAssembly module perform lives
  in one place (`necro_cobol_lifter::fixed`, `necro_rom_lifter::interp`) and uses
  wrapping operations that mirror the wasm instructions exactly.
- Sound is never produced. The CHIP-8 sound timer is modelled as a byte; nothing
  plays it.
