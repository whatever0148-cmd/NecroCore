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
crates/necro-daemon/   the lean template: std-only HTTP and SSE daemon, one UI shell, AppSpec
apps/cryptledger/      app 1: the COBOL studio (own binary, port 4301, views crypt + proof)
apps/phantomarcade/    app 2: the CHIP-8 and SUPER-CHIP lifter and live modder (own binary, port 4302, views arcade + proof)
apps/macos-notch/      the Swift notch pill that drops files on the local daemon (port 4300)
necro-cli/             the combined binary: autopsy, lift, assemble, benchmark, mcp, observatory (both apps, port 4300)
samples/cobol/         three period-style COBOL programs with deliberate defects
samples/roms/          four original games, CHIP-8 and SUPER-CHIP (.asm source, .ch8 binary, .patch.json)
```

## Skeleton Crew shape
The template is the engine crates plus `necro-daemon`. An app is a folder under `apps/` with its own
`Cargo.toml` and a `main.rs` of about 25 lines that names the app, picks its views and its port. A new
app costs one folder, not a fork of the daemon.

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
