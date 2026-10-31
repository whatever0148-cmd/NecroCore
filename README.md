# NecroCore

Legacy compiler translation, static AST/CFG analysis, memory-leak diagnostics and
retro bytecode emulation on WebAssembly, built with Kiro for Kiroween 2026
(Resurrection category).

NecroCore takes a dead program, a 1978 COBOL batch job or a CHIP-8 ROM, lifts it
to one intermediate representation, finds three kinds of rot in its control-flow
graph, compiles it to WebAssembly, and proves with 10,000 seeded vectors that the
lifted module behaves exactly like the reference interpreter. The Halloween names
are the UI labels; the technical terms sit next to them everywhere.

| Label | Technical term | What the scanner checks |
|---|---|---|
| Ghost | unreachable code | no path from the entry in the CFG (PERFORM-context aware) |
| Vampire | unbounded accumulator / buffer growth | a variable that only grows inside a loop and bounds no exit |
| Zombie | non-terminating loop | a loop with no exit, or whose exits read only loop-invariant variables |

## What is in the box

| Crate | Role |
|---|---|
| `crates/necro-core` | `NecroIR`, CFG with Tarjan loops, the scanner, `NecroAutopsyReport` with byte and line offsets and EARS text, `heal` |
| `crates/necro-cobol-lifter` | parser for the documented COBOL subset (`COBOL-SUBSET.md`), lowering, fixed-point interpreter, WebAssembly text emitter |
| `crates/necro-rom-lifter` | CHIP-8 decoder, flat-image interpreter, two-pass assembler with `.patch` points, NecroIR and WebAssembly lifters |
| `crates/necro-prover` | differential shadow execution against `wasmtime`, parity, divergence vectors, microsecond timings |
| `crates/necro-kiro-bridge` | async `kiro-cli` driver with a hard timeout and a deterministic offline fallback |
| `necro-cli` | `autopsy`, `lift`, `assemble`, `benchmark`, and `mcp` (the `necro-mcp` server for Kiro) |
| `samples/cobol` | three period-style COBOL programs with deliberate defects (two-digit years, COMP-3 drift, a dead paragraph, a loop that never ends on an edge input) |
| `samples/roms` | three original CHIP-8 games (source, binary, patch table), see `samples/roms/LICENSES.md` |
| `.kiro/` | steering, four specs, three hooks, the MCP server config |

## Measured results

Everything below is the unedited output of `cargo run --release --bin necro-cli -- benchmark`
on 2026-10-31 (macOS 27.0.1, Apple M2, rustc 1.99.1, wasmtime 50). The same
text is in `benchmarks/2026-10-31-phase1.txt` and the JSON in
`benchmarks/2026-10-31-phase1.json`.

```
NecroCore benchmark: 10000 vectors per target, seed 7, COBOL fuel 10000 blocks, ROM 300 steps per vector
host: macos aarch64, necro-cli 0.1.0, wasmtime 50 (Cranelift), release build

| target | kind | lift us | blocks | edges | ghosts | vampires | zombies | warnings | left after heal | parity | interp us/vec | wasm us/vec | speedup | compile us | non-halting vecs | bridge |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| BANK-INTEREST-1978 | cobol | 494.1 | 26 | 23 | 1 | 1 | 0 | 4 | 0 | 100.00% (10000/10000) | 62.76 | 6.92 | 9.07x | 10494 | 2845 | offline (2 patches, 0 ms) |
| PAYROLL-BATCH-1982 | cobol | 77.0 | 15 | 11 | 1 | 1 | 1 | 2 | 0 | 100.00% (10000/10000) | 458.59 | 27.70 | 16.55x | 1430 | 10000 | offline (3 patches, 0 ms) |
| WIRE-CLEARING-1985 | cobol | 68.0 | 22 | 16 | 1 | 1 | 0 | 2 | 0 | 100.00% (10000/10000) | 46.18 | 2.71 | 17.07x | 1365 | 1138 | offline (2 patches, 0 ms) |
| BREAKOUT-STYLE | chip8 | 181.1 | 62 | 164 | 0 | 0 | 2 | 0 | 0 | 100.00% (10000/10000) | 2.23 | 2.04 | 1.09x | 2593 | 0 | offline (2 patches, 0 ms) |
| INVADERS-STYLE | chip8 | 155.1 | 61 | 113 | 0 | 2 | 2 | 0 | 0 | 100.00% (10000/10000) | 2.47 | 2.41 | 1.02x | 2472 | 0 | offline (4 patches, 0 ms) |
| PONG-STYLE | chip8 | 134.8 | 55 | 135 | 1 | 0 | 1 | 0 | 0 | 100.00% (10000/10000) | 2.20 | 2.35 | 0.94x | 2412 | 0 | offline (2 patches, 0 ms) |

pathologies found: 15, remaining after heal: 0, healed: 15
vectors: 60000, matched: 60000, overall parity: 100.000%
  BANK-INTEREST-1978: FuelExhausted x 2845
  BANK-INTEREST-1978: Halted x 7155
  PAYROLL-BATCH-1982: FuelExhausted x 10000
  WIRE-CLEARING-1985: FuelExhausted x 1138
  WIRE-CLEARING-1985: Halted x 8862
  BREAKOUT-STYLE: Ok x 10000
  INVADERS-STYLE: Ok x 10000
  PONG-STYLE: Ok x 10000
wrote benchmarks/2026-10-31-phase1.json
```

Test run on the same day (`cargo test --release --workspace`): tests passed 23 failed 0 ignored 0.
`cargo clippy --release --workspace --all-targets` reported 0 warnings.

### Reading the numbers
- Parity is byte-for-byte: every COBOL variable and status, or the whole
  4608-byte CHIP-8 machine image, after each vector. 60,000 of 60,000 matched.
- COBOL vectors seed every level-77 item, biased to 0, 1, max and max-1. The
  "non-halting vecs" column counts vectors where both executors ran out of fuel.
  PAYROLL-BATCH-1982 never halts (its end-of-file flag is set in a paragraph
  nobody performs: a Zombie and a Ghost). BANK-INTEREST-1978 fails to halt on
  2,845 of 10,000 inputs because the monthly interest on a small balance
  truncates to zero in COMP-3 and the balance never reaches the target; the
  scanner does not see that one (the two loops share a paragraph and collapse
  into one component), the prover does. WIRE-CLEARING-1985 spins on 1,138 inputs
  where the PIC 99 sequence counter wraps before it passes the batch size.
- CHIP-8 speedup is about 1x at 300 steps per vector because each vector copies
  the machine image in and out of the module and makes 12 host calls; the
  interpreter is already a tight loop over a byte array. The COBOL modules are
  9x to 17x faster than the interpreter per vector.
- "left after heal" is a rescan of the healed IR: ghosts detached, zombie loops
  given a fuel guard, vampires given a cap guard. 15 found, 0 left.
- The bridge column says `offline` because the benchmark does not call
  `kiro-cli` by default; `--live-kiro` does, with a 120 s timeout and no tool
  access.

## Try it

```bash
cargo test --release --workspace
cargo run --release --bin necro-cli -- autopsy samples/cobol/PAYROLL-BATCH-1982.cob --requirements
cargo run --release --bin necro-cli -- lift samples/roms/PONG-STYLE.ch8 -o pong.wat
cargo run --release --bin necro-cli -- benchmark --vectors 1000
```

The MCP server used by `.kiro/settings/mcp.json`:

```bash
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"necro_hotpatch","arguments":{"rom":"samples/roms/PONG-STYLE.ch8","patch":"PADDLE_H","value":9}}}' | cargo run --quiet --release --bin necro-cli -- mcp
```

## How Kiro is used
- **Steering** (`.kiro/steering/`): product, tech and structure files loaded in
  every interaction. The product file carries the rule that every number comes
  from a run.
- **Specs** (`.kiro/specs/`): four specs in EARS form. The scanner itself writes
  EARS requirements for every finding (`NecroAutopsyReport::requirements_markdown`),
  so an autopsy becomes a spec input.
- **Hooks** (`.kiro/hooks/`): `on-legacy-drop` runs the autopsy when a sample is
  saved, `on-exorcise-defect` reruns the sample tests after the scanner or healer
  is edited, `on-parity-verify` runs a 1,000-vector parity check before the agent
  stops.
- **MCP** (`.kiro/settings/mcp.json`): `necro-mcp` exposes `necro_autopsy`,
  `necro_parity` and `necro_hotpatch` to the agent.
- **Kiro CLI bridge** (`crates/necro-kiro-bridge`): turns a report into spec text
  and patch proposals through `kiro-cli chat --no-interactive --trust-tools=`.
- **Property-based tests**: `proptest` on the fixed-point store rule and literal
  parsing.

## Observatory (browser cockpit)

```bash
cargo run --release --bin necro-cli -- benchmark --json benchmarks/run.json
cargo run --release --bin necro-cli -- observatory --bench benchmarks/run.json -o observatory/index.html
python3 -m http.server 8765 --directory observatory   # then open http://127.0.0.1:8765
```

`observatory/index.html` is one file with no network use. It embeds each autopsy
report, the benchmark rows, and every ROM as a compiled wasm binary. The page runs
the lifted module itself (12 steps per animation frame), draws the 64x32 display,
shows the registers, and the Hot patch sliders write one byte into the running
module's memory with no reset. Checked in a browser on 2026-10-31: the ball moved,
holding CHIP-8 key 4 moved the Pong paddle from y 28 to y 16 in 1.5 s, a slider
write changed exactly the patched byte, and the page has no horizontal scroll at
375 px.

## Phase 2 still open
The macOS notch strip (spec 04, requirement 2) is not built. The Observatory is
a browser page, not the native SwiftUI app the spec describes, and the Prove,
Exorcise and Spec buttons are not wired (the page shows the stored benchmark
instead of running it). Multi-ball and SUPER-CHIP are open tasks in spec 03.

## Licence
MIT. The sample programs and ROMs are original work written for this project.
