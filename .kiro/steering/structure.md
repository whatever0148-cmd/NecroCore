---
inclusion: always
---

# NecroCore: structure

```
.kiro/                 specs, steering, hooks, settings/mcp.json (committed, never ignored)
crates/necro-core/     NecroIR, CFG, static scanner, autopsy report, healer
crates/necro-cobol-lifter/  COBOL subset parser, lowering, fixed-point interpreter, WAT emitter
crates/necro-rom-lifter/    CHIP-8 decoder, interpreter, assembler, patch table, NecroIR + WAT lifters
crates/necro-prover/   differential shadow execution against wasmtime
crates/necro-kiro-bridge/   kiro-cli subprocess driver with offline fallback
necro-cli/             the binary: autopsy, lift, assemble, benchmark, mcp
samples/cobol/         three period-style COBOL programs with deliberate defects
samples/roms/          three original CHIP-8 games (.asm source, .ch8 binary, .patch.json)
```

## Naming
- Crates are `necro-<role>`. Modules are nouns (`cfg`, `scan`, `heal`, `wat`).
- Pathology kinds are `Ghost`, `Vampire`, `Zombie` in the enum; the `technical()`
  method gives the plain term for reports.
- Sample programs carry the year of the style they imitate in the COBOL file name
  (`BANK-INTEREST-1978.cob`) because they are written in that period's style. ROMs
  are named `<GAME>-STYLE` because they are original programs, not historical
  binaries.

## Data flow
source -> parser -> NecroIR -> {scan -> report -> bridge -> spec text,
                                 emit_wat -> wasmtime,
                                 interpreter}
report + IR -> heal -> rescan
interpreter + wasmtime -> prover -> parity, divergences, timings -> README table
