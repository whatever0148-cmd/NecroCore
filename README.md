# NecroCore

Legacy compiler translation, static AST/CFG analysis, memory-leak diagnostics and
retro bytecode emulation on WebAssembly, built with Kiro for Kiroween 2026
(Resurrection category).

NecroCore takes a dead program, a 1978 COBOL batch job or a CHIP-8 ROM, lifts it
to one intermediate representation, finds three kinds of rot in its control-flow
graph, compiles it to WebAssembly, and proves with 10,000 seeded vectors that the
lifted module behaves exactly like the reference interpreter. The Halloween names
are the UI labels. The technical terms sit next to them everywhere.

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
| `crates/necro-rom-lifter` | CHIP-8 and SUPER-CHIP decoder, flat-image interpreter, two-pass assembler with `.patch` points, NecroIR and WebAssembly lifters |
| `crates/necro-prover` | differential shadow execution against `wasmtime`, parity, divergence vectors, microsecond timings |
| `crates/necro-kiro-bridge` | async `kiro-cli` driver with a hard timeout and a deterministic offline fallback |
| `crates/necro-daemon` | the lean template: std-only HTTP and SSE daemon, one UI shell, `AppSpec` |
| `apps/cryptledger` | app 1, the COBOL studio. Own binary, port 4301 |
| `apps/phantomarcade` | app 2, the CHIP-8 and SUPER-CHIP lifter and live modder. Own binary, port 4302 |
| `apps/macos-notch` | the Swift notch pill |
| `necro-cli` | `autopsy`, `lift`, `assemble`, `benchmark`, and `mcp` (the `necro-mcp` server for Kiro) |
| `samples/cobol` | three period-style COBOL programs with deliberate defects (two-digit years, COMP-3 drift, a dead paragraph, a loop that never ends on an edge input) |
| `samples/roms` | four original games: three CHIP-8 and the 128x64 SUPER-CHIP NECRORUNNER-1991 (source, binary, patch table), see `samples/roms/LICENSES.md` |
| `.kiro/` | six specs, ten steering docs, nine hooks, four custom agents, the MCP server config |

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
  truncates to zero in COMP-3 and the balance never reaches the target. The
  scanner does not see that one (the two loops share a paragraph and collapse
  into one component), the prover does. WIRE-CLEARING-1985 spins on 1,138 inputs
  where the PIC 99 sequence counter wraps before it passes the batch size.
- CHIP-8 speedup is about 1x at 300 steps per vector because each vector copies
  the machine image in and out of the module and makes 12 host calls. The
  interpreter is already a tight loop over a byte array. The COBOL modules are
  9x to 17x faster than the interpreter per vector.
- "left after heal" is a rescan of the healed IR: ghosts detached, zombie loops
  given a fuel guard, vampires given a cap guard. 15 found, 0 left.
- The bridge column says `offline` because the benchmark does not call
  `kiro-cli` by default. `--live-kiro` does, with a 120 s timeout and no tool
  access.

## Live demo

Hosted copy, no install: https://whatever0148-cmd.github.io/NecroCore/

It opens on PhantomArcade, where the four lifted games run as WebAssembly in your
browser and the patch sliders write into the running module. The CryptLedger and
Evidence tabs show the real scan results and benchmark. The buttons that call
`kiro-cli` and the prover need the local daemon (below).

## Run the full apps

Needs Rust stable. `kiro-cli` is optional: without it the bridge uses its offline
fallback and says so.

```bash
git clone https://github.com/whatever0148-cmd/NecroCore && cd NecroCore
cargo build --release
./target/release/cryptledger        # COBOL studio on http://127.0.0.1:4301
./target/release/phantomarcade      # ROM lifter and live modder on http://127.0.0.1:4302
./target/release/necro-cli observatory --serve   # both together on http://127.0.0.1:4300
```

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
Every item has a job, and `benchmarks/hook-firings.log` shows which ones Kiro
actually fired.
- **Specs** (`.kiro/specs/`): six specs in EARS form, from the core engine (01) to
  the machine-room redesign (05) and the guardrails (06). The scanner also writes
  EARS requirements for every finding (`NecroAutopsyReport::requirements_markdown`),
  so an autopsy becomes a spec input (`autopsy-PAYROLL-BATCH-1982/`).
- **Steering** (`.kiro/steering/`, 10 docs): `product`, `tech`, `structure` and
  `proof-before-trust` load every time. `voice`, `cobol-subset`, `chip8-semantics`,
  `observatory-design` and `prover-and-wasm` load only for the files they cover.
  `kiro-workflow` is loaded by hand with `#kiro-workflow`.
- **Hooks** (`.kiro/hooks/`, 9). One script, `scripts/hooks/hook.py`, does the work.
  | Hook | Trigger | Job |
  |---|---|---|
  | `guard-benchmarks` | PreToolUse write | blocks hand edits to saved benchmark files |
  | `judges-voice` | PostToolUse write | returns em dashes, semicolons and buzzwords in judge-facing text |
  | `sync-hosted-page` | PostToolUse write | rebuilds `docs/index.html` when the UI changes |
  | `crate-tests` | PostToolUse write | runs the tests of the crate that was just edited |
  | `on-exorcise-defect` | PostToolUse write | rechecks the samples after scanner or healer edits |
  | `session-brief` | SessionStart | gives Kiro the last commits, open tasks and the last proof |
  | `on-parity-verify` | Stop | runs a 1,000-vector proof before the agent stops |
  | `task-proof` | PostTaskExecution | tests plus a proof when a spec task ends |
  | `on-legacy-drop` | PostFileSave | runs the autopsy on a dropped `.cob` or `.ch8` |
- **Custom agents** (`.kiro/agents/`, 4, run with `kiro-cli --v3 chat --agent <name>`):
  `exorcist` heals findings in the engine crates. `prover` has no file-write tool,
  and its shell runs only the benchmark and the tests without asking. `scribe`
  writes only README.md and `deliverables/`. `medium` has no write tool and no
  shell, it hot-patches running games through `necro_hotpatch`.
- **MCP** (`.kiro/settings/mcp.json`): `necro-mcp` exposes `necro_autopsy`,
  `necro_parity` and `necro_hotpatch` to the agent.
- **Kiro CLI bridge** (`crates/necro-kiro-bridge`): turns a report into spec text
  and patch proposals through `kiro-cli chat --no-interactive --trust-tools=`.
- **Property-based tests**: `proptest` on the fixed-point store rule and literal
  parsing.

### What Kiro did on 2026-11-15
- The `scribe` agent fixed the four prose semicolons that `judges-voice` found in
  this README. The hook fired on each of its writes and the last check was clean.
- The `prover` agent ran `necro-cli benchmark --vectors 10000 --live-kiro`. All 7
  programs got a real kiro-cli 2.37.0 turn (107.8 s in total, 0 offline
  fallbacks) and 70,000 of 70,000 vectors matched
  (`benchmarks/2026-11-15-live-kiro.json`, `benchmarks/2026-11-15-live-kiro-trace.json`).
- Later that day I fixed three bugs in BREAKOUT-STYLE: a missed ball froze the
  game on a blank screen, the GRAVITY slider only changed a line that runs at
  start, and clearing one brick hid its whole row. Three new tests fail on the
  old ROM and pass on the new one. The run was repeated on the fixed ROM:
  70,000 of 70,000 matched, 18 found (the old lose loop was a Zombie), 0 left,
  7 of 7 real kiro-cli turns in 102.7 s (`benchmarks/2026-11-15-r2-live-kiro.json`).
  The live site and the local daemon read this run.
- In those 5 sessions Kiro fired 7 of the 9 hooks: `session-brief`,
  `guard-benchmarks`, `judges-voice`, `sync-hosted-page`, `crate-tests`,
  `on-exorcise-defect` and `on-parity-verify`. `task-proof` and `on-legacy-drop`
  have not been fired by Kiro yet.
- What did not work at first: the default engine could not find the workspace
  agents (only `--v3` loads them), and a catch-all deny rule in `scribe` blocked
  its allowed writes too. Both fixes are in the commits.

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

## Phase 2: local daemon, notch pill, live Kiro trace

`necro-cli observatory --serve` starts a zero-dependency HTTP and Server-Sent-Events daemon on
127.0.0.1:4300 (loopback Host and Origin checks, no external crates). The page it serves has three views.
- **CryptLedger.** Pick a COBOL sample or drop your own .cob. Buttons run the real code paths:
  Autopsy CFG (clickable graph tied to source lines), Exorcise via Kiro (a real kiro-cli turn, then heal,
  rescan and a before and after WebAssembly diff), Prove 10,000 vectors (ten batches of 1,000 streamed
  through wasmtime, same seeded vectors as the benchmark), and Generate .kiro/specs/ (writes
  requirements, design and tasks files).
- **PhantomArcade.** The three lifted ROMs at a 60 or 120 ticks per second module clock, a register panel,
  a RAM heat map, a Multi-Ball preset (Pong now has a second ball behind a patch point), and a text box.
  The text box sends your sentence to kiro-cli, which answers with patch values. They are written into
  the running module without a reset.
- **Evidence.** The saved benchmark and the saved kiro-cli transcript.

`apps/macos-notch/necro_notch.swift` is a borderless dark-glass pill that docks under the MacBook notch.
Drop a .cob or .ch8 on it, or press Resurrect sample. It posts the file to the daemon, which saves it under
samples/dropped/ and runs the command from `.kiro/hooks/on-legacy-drop.json`. The pill shows the Ghost,
Vampire and Zombie counts and opens the healed result in the Observatory. Build and check:

```
swiftc -O apps/macos-notch/necro_notch.swift -o apps/macos-notch/necro_notch
apps/macos-notch/necro_notch --sample PAYROLL-BATCH-1982     # same path as a drop, no window
```

Live Kiro benchmark (`necro-cli benchmark --live-kiro`, output in `benchmarks/2026-10-31-phase2.txt`,
timings in that .txt): all 6 targets got a real kiro-cli turn, 117.7 s in total,
and parity stayed at 60,000 of 60,000. COBOL speedups on that run were 8.59x, 14.57x and 16.63x. The run
shared the machine with a browser and a compiler, so they sit a little under the Phase 1 numbers.

Limits. The Observatory page was checked in an embedded browser at 1920 by 1080 and 375 wide. The in-app
browser pane does not run animation frames while hidden, so the game loop was driven by hand there. The
notch pill was checked by launching it (it appears under the notch) and by the headless drop path, not by
dragging a file with a mouse. The mod box depends on what kiro-cli answers, and on one live run it left
the zero-gravity request out until the prompt got a hint. 

## Why it matters

A 2017 Thomson Reuters graphic, reported by [The Next Web on April 10, 2017](http://thenextweb.com/news/ancient-programming-language-cobol-can-make-you-bank-literally), says about 80 percent of in-person transactions and 95 percent of ATM swipes in the US run on programs written in COBOL. The same Reuters work is widely quoted for 43 percent of banking systems and about 220 billion lines of COBOL in production. I only found those two second hand and their scope is not consistent across articles, so treat them as widely quoted, not verified. I could not trace any yearly maintenance cost figure to a primary study, so none is quoted. NecroCore targets the part that blocks a rewrite, which is proof that the new program computes what the old one did.

## Phase 3: SUPER-CHIP and 70,000 vectors

`necro-rom-lifter` now handles SUPER-CHIP: `00FF` and `00FE` (128x64 and 64x32), `00Cn`, `00FB`
and `00FC` (scroll down, right, left), `Dxy0` (16x16 sprite in high-res, nothing in low-res), and
`Fx75` and `Fx85` (the RPL user flags). The decoder, interpreter, assembler, NecroIR lifter and the
WebAssembly emitter all know them. The machine image grew from 0x1200 to 0x1500 bytes and holds a mode
byte and the eight flags, so the prover still compares the whole image. Mode switches clear the screen
and Dxy0 reports VF as 1 on any collision. Those are my rules, written in spec 03, not a copy of any one
historical interpreter.

`samples/roms/NECRORUNNER-1991.asm` is an original 128x64 runner (key 5 jumps). It scrolls the ground
with SCL, draws 16x16 sprites, keeps its best score in the RPL flags and has three patch points
(OBST_SPEED, JUMP_STEP, FRAME_DELAY). It carries two deliberate defects, a game_over loop that spins
forever and a bonus_level routine nothing calls. The scanner finds a Ghost, a Vampire and two Zombies in it.
The file ends in `.ch8` like the other ROMs so every tool picks it up.

Final run, `necro-cli benchmark --live-kiro` (`benchmarks/2026-10-31-phase3.txt`, machine busy with
other work):

| Result | Value |
|---|---|
| Programs and vectors | 7 programs, 10,000 vectors each, seed 7 |
| Parity | 70,000 of 70,000 matched |
| COBOL wasm vs interpreter | 9.07x, 16.49x, 16.48x |
| ROM wasm vs interpreter | 0.92x to 1.06x (2026-10-31 run, before `step_batch`) |
| Problems found, left after heal | 19 found, 0 left |
| Real kiro-cli turns | 7 of 7, 142 s in total (`benchmarks/2026-10-31-live-kiro-trace.json`) |
| Tests, clippy | 32 passed, 0 failed, 0 warnings |

Other real Kiro evidence in `benchmarks/`: `live-kiro-mcp-trace.json` (kiro-cli read
`.kiro/settings/mcp.json`, started `necro-mcp` and called `necro_autopsy`) and `live-mod-trace.json` (two
mod-box turns for "double ball speed, widen paddles, and enable zero-gravity bounce", both answered with
FRAME_DELAY 1, PADDLE_H 8 and BALL_DY 0). The Observatory switches its canvas between 64x32 and 128x64
from the machine's mode byte. Written submission text is in `deliverables/`.

## Execution speed

The lifted module has three execution paths. `step(n)` is the original per-instruction entry. `step_batch(n, period)`
runs many instructions per host call. Spec 03 requirements 6 to 8 add, in order, batch execution with the PC and timer
in locals, fused basic blocks, and registers held in WebAssembly locals with row-wise sprite drawing.

`kiro-cli` (V3 engine, `.kiro/` loaded) wrote the requirement 6 code in real coding turns: the `step_batch` export,
the key schedule layout, the prover change and a parity test. I wrote the requirement, the design and the tasks,
ran cargo between turns and pasted the errors back, and `benchmarks/kiro-coding-sessions.json` records every turn and
the one hand edit. While it worked Kiro fired `on-exorcise-defect` 18 times and `on-parity-verify` twice
(`benchmarks/hook-firings.log`). `on-legacy-drop` runs from the daemon and the notch pill on every dropped file.

Lifted WebAssembly against the native Rust reference interpreter, 70,000 of 70,000 vectors matched at every step:

| Stage | ROMs, 300 steps per vector | ROMs, 30,000 steps per vector |
|---|---|---|
| `step(n)` host call per chunk (2026-10-31) | 0.92x to 1.06x | not measured |
| `step_batch` (requirement 6) | 0.94x to 1.13x | 1.38x to 1.74x |
| plus fused basic blocks (requirement 7) | 1.00x to 1.41x | 1.51x to 1.89x |
| plus registers in locals and row-wise drawing (requirement 8) | 1.61x to 2.40x | 1.87x to 2.32x |

Final run (`benchmarks/2026-11-01-register-locals.txt` and `-longrun.txt`, M2 MacBook Air, release build, seed 7):
COBOL to WebAssembly runs 8.69x to 17.25x faster than the reference interpreter, and CHIP-8 and SUPER-CHIP lifted
WebAssembly runs 1.61x to 2.40x faster, across 70,000 verified vectors (7 programs, 10,000 each) with 19 findings
reported and 0 left after healing. Run-to-run noise on a laptop is a few percent. The ROM numbers are lower than the
COBOL ones because a ROM step is a few nanoseconds of work in a native interpreter that is already tight, and the
draw and scroll routines cost about the same in both.

## Skeleton Crew layout

The engine crates and `necro-daemon` are the template. The two apps are separate folders with their own
binaries, built on that template.

```
cargo build --release
./target/release/cryptledger        # http://127.0.0.1:4301  COBOL studio
./target/release/phantomarcade      # http://127.0.0.1:4302  ROM lifter and live modder
./target/release/necro-cli observatory --serve   # http://127.0.0.1:4300  both in one page, used by the notch pill
```

Each app main is about 25 lines. It names the app, picks which views of the shared UI shell to show and
sets a port. Everything else comes from the shared crates, so every button runs the same tested code in all
three. The notch pill still talks to the combined daemon on 4300.

## Built during Kiroween 2026, and what came before

- The repository starts on 2026-10-31 at 06:43 Pacific, after the submission
  period opened on 2026-10-30 at 09:00 Pacific. Every line of code here, the
  three COBOL sample programs and the four ROMs were written for this hackathon.
- No code was copied from my earlier projects.
- Third-party code is limited to the Rust crates in the `Cargo.toml` files
  (wasmtime, wat, tokio, clap, serde, serde_json, proptest), used under their
  licences. COBOL and the CHIP-8 and SUPER-CHIP instruction sets are public
  specifications. No historical program or ROM is included
  (`samples/roms/LICENSES.md`).
- AI tools: Kiro for the steering, specs, hooks, MCP server and the coding turns
  logged in `benchmarks/kiro-coding-sessions.json`. Claude Code wrote much of
  the remaining Rust, the spec and steering text and the interface, and ran the
  tests. I directed the build and checked the results.

## Licence
MIT. The sample programs and ROMs are original work written for this project.
