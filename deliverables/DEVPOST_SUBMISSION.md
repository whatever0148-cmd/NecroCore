# NecroCore

## Tagline
Bring dead programs back as WebAssembly, then prove they still behave.

## Category
Resurrection. The project also touches the other three tracks, and the sections below say how.

## Inspiration
Old programs do not die. They get stuck. A bank still runs a payroll job that somebody wrote in 1982, and nobody dares to touch it because nobody can say what it really does. Old games have the same problem in a friendlier way. A ROM runs on an emulator, but you cannot ask it questions or change it while it runs.

I wanted one tool that does three things for any old program. It reads it and says what is wrong with it. It rebuilds it as WebAssembly. It proves, with numbers, that the rebuilt copy behaves like the original.

## What it does
NecroCore takes two kinds of legacy programs.
- COBOL batch programs, in a subset I documented in the repo. I wrote three samples styled after 1978, 1982 and 1985 jobs, each with deliberate defects.
- CHIP-8 and SUPER-CHIP game ROMs. I wrote four original games, including NECRORUNNER-1991, a 128x64 SUPER-CHIP runner.

For each program it does the following.
1. **Autopsy.** It lifts the program to one intermediate form, builds the control-flow graph, and reports three static problems. The UI calls them Ghost (unreachable code), Vampire (a counter that only grows and bounds nothing) and Zombie (a loop that can never exit). Every finding carries a line and byte range.
2. **Lift.** It compiles the program to WebAssembly text that wasmtime runs.
3. **Prove.** It feeds 10,000 seeded vectors to both a reference interpreter and the WebAssembly module and compares every result byte for byte.
4. **Exorcise.** It detaches the ghosts and puts a guard on every loop, then scans again. In the last run it found 19 problems across the 7 sample programs and 0 were left after healing.
5. **Spec.** It writes the findings as EARS requirements into `.kiro/specs/` and asks kiro-cli for a fix plan.
6. **Hot patch.** A ROM can be changed while it runs. Pong gets a second ball, a faster pace or wider paddles by writing one byte into the module. The score and the screen stay.

### Measured results (from `benchmarks/2026-10-31-phase3.txt`)
| What | Result |
|---|---|
| Differential vectors | 70,000 of 70,000 matched (7 programs, 10,000 each, seed 7) |
| COBOL, WebAssembly vs interpreter | 9.07x, 16.49x and 16.48x faster per vector |
| ROMs, WebAssembly vs interpreter | 0.92x to 1.06x, so no real speedup. Copying the machine image costs about as much as the work |
| Problems found and left after healing | 19 found, 0 left |
| Real kiro-cli turns in the benchmark | 7 of 7, 142 s in total |
| Tests | 32 passed, 0 failed. Clippy shows 0 warnings |

## How it fits the four tracks
**Resurrection.** This is the main entry. A 1978 style interest program and a 1991 style SUPER-CHIP game come back as live WebAssembly, and the proof is a number, not a promise.

**Frankenstein.** The parts do not belong together, and they are stitched into one tool: COBOL decimal arithmetic, 8-bit game bytecode, WebAssembly, a Rust daemon with server-sent events, a Swift program that lives under the MacBook notch, and the Kiro CLI.

**Skeleton Crew.** The template is one shared engine (the intermediate form, the scanner, the healer, the prover and the Kiro bridge) plus `crates/necro-daemon`, a small server and UI shell. Two apps sit on it in separate folders. `apps/cryptledger` is the COBOL studio and `apps/phantomarcade` is the ROM lifter and live modder. Each has its own Cargo package, its own binary and its own port, and its `main.rs` is about 25 lines. A third app would cost one more folder.

**Costume Contest.** The interface treats bugs as monsters, and the monsters are the function. A Ghost row is dead code, a Zombie row is a loop that never ends. The notch pill takes a file dropped on it and shows the three counts. The page is dark and quiet. The Impeccable detector reports 0 findings, and I measured it at 1920 by 1080 and at 375 pixels wide with no sideways scrolling.

## Best Startup pitch
The paid part of legacy modernization is trust. Banks and insurers do not lack tools that translate code. They lack evidence that the translated program does what the old one did. NecroCore's prover produces that evidence as a table of vectors and a list of exact divergences, which is something a reviewer or auditor can read.

The path from here is narrow on purpose. Start with batch programs that are small and well defined. Sell the autopsy report and the parity proof as a service for one program at a time. Grow the COBOL subset from the real programs customers bring. I am a first-year student and this is a hackathon build, so I am not quoting a market size. I did not want to paste a number I cannot source. The need is real and the proof step is the part I can already demonstrate.

## How we built it with Kiro
I want to be exact about this, so here is what is real and what is not.

**Spec-driven development.** Before writing the Rust I wrote four specs in `.kiro/specs/`, each with `requirements.md` in EARS form (WHEN something happens THE SYSTEM SHALL do something), `design.md` and `tasks.md`.
- `01-core-resurrection-engine`
- `02-cryptledger-cobol-prover`
- `03-phantomarcade-live-patcher`
- `04-observatory-and-notch-ui`

I then worked down the task lists and ticked them off. Late in the build I added a SUPER-CHIP requirement to spec 03 before writing the code for it. The Spec button in CryptLedger also writes new specs of the same shape. For example, `.kiro/specs/autopsy-PAYROLL-BATCH-1982/` is generated from a real scan.

**Steering.** `.kiro/steering/product.md`, `tech.md` and `structure.md` are marked `inclusion: always`. They fix the vocabulary (Ghost, Vampire and Zombie are UI labels, the technical terms are dead code, resource leaks and non-terminating loops), the rule that numbers come from real runs, and the layout of the crates.

**Hooks.** Three hooks use the Kiro v1 hook schema.
- `on-legacy-drop.json` runs the autopsy and writes EARS text when a `.cob` or `.ch8` file lands under `samples/`.
- `on-exorcise-defect.json` reruns the lifter tests after an edit to the scanner or the healer.
- `on-parity-verify.json` runs the prover with 1,000 vectors per sample when a session stops.

The notch pill and the daemon actually run the command from `on-legacy-drop.json` on every drop. I tested that and it took 811 ms. I have not watched the Kiro IDE fire the other two hooks, so I will not claim that.

**MCP.** `.kiro/settings/mcp.json` defines `necro-mcp`, a small MCP server I wrote in Rust that speaks JSON-RPC over stdio. It offers `necro_autopsy`, `necro_parity` and `necro_hotpatch`. I ran a real `kiro-cli` turn in the repo. It read the config, started the server, called `necro_autopsy` on the payroll program and reported the Zombie, the Vampire and the Ghost with their lines. The transcript is in `benchmarks/live-kiro-mcp-trace.json`.

**Kiro CLI in the product.** The `necro-kiro-bridge` crate runs `kiro-cli chat --no-interactive` with tools switched off and a timeout, and falls back to a deterministic offline answer if the CLI is missing or fails. Three buttons use it for real.
- Exorcise asks Kiro for requirements and a patch plan.
- The benchmark with `--live-kiro` asked for a plan for each of the 7 programs. Every one of the 7 turns was live. The full prompts and answers are in `benchmarks/live-kiro-trace.json`.
- The mod box in PhantomArcade sends a sentence such as "double ball speed, widen paddles, and enable zero-gravity bounce" and gets patch values back. Kiro answered with `FRAME_DELAY 1`, `PADDLE_H 8` and `BALL_DY 0`. The page wrote those three bytes into the running game with no reset. Two such turns, 13.5 s and 16.1 s, are saved in `benchmarks/live-mod-trace.json`.

**Property-based testing.** The COBOL lifter has `proptest` tests, and the prover is a seeded differential test of its own with 70,000 vectors.

**What I did not use.** I did not use Kiro Powers, custom agents or Kiro Crew. I wrote most of the Rust with Claude Code as my coding partner, and I used Kiro for the specs, steering, hooks, MCP, CLI integration and the evidence above.

## How I built the rest
- **Rust workspace.** Five library crates and one CLI. Rust stable, wasmtime 50, tokio, clap, serde and proptest.
- **COBOL.** A parser for a documented subset, an interpreter with fixed-point decimal rules, and a WebAssembly emitter. The interpreter and the emitter share the same rules: a working scale of 10,000 in a 64-bit integer, truncation or ROUNDED half away from zero, and high-order digits dropped to the PIC width.
- **CHIP-8 and SUPER-CHIP.** A decoder, an interpreter, a two-pass assembler with `.patch` directives, and a WebAssembly emitter with one `br_table` case per ROM address. Patchable bytes are loaded from memory at run time, so a hot patch takes effect on the next instruction.
- **Daemon.** `crates/necro-daemon` is a std-only HTTP and server-sent-events server on 127.0.0.1. It checks the Host and Origin headers. The two apps and the combined Observatory all use it.
- **Notch pill.** `apps/macos-notch/necro_notch.swift`, about 300 lines of AppKit, compiled with `swiftc`.

## Challenges
- My first control-flow graph let a PERFORMed paragraph fall through into the next one, which made every loop look like it had an exit. I rebuilt it so that it tracks the call context.
- The static scanner cannot see that BANK-INTEREST fails to finish on 2,845 of 10,000 inputs. Two loops share one paragraph and merge. The prover catches it. I wrote this limit into the spec instead of hiding it.
- ROM speedups are about 1x. I kept that in the results table.
- Kiro once answered the mod box with valid JSON that was missing its last closing brace. The bridge now repairs that, labels the source as repaired, and has tests for it.

## What I learned
Proving a rewrite is mostly about choosing what to compare. For CHIP-8 I compare the whole machine image, so a hidden difference in a register or in a scrolled pixel shows up. The 128x64 mode added a mode byte and 8 user flags to that image, and the same comparison kept working.

## What is next
A bigger COBOL subset (the REDEFINES clause and file handling) and a Kiro Power that bundles the autopsy.

## Built with
Rust, WebAssembly, wasmtime, Swift, AppKit, tokio, serde, proptest, Kiro CLI, Kiro specs, steering, hooks and MCP.

## Try it
```
cargo build --release
./target/release/cryptledger
./target/release/phantomarcade
```
Open http://127.0.0.1:4301 for CryptLedger and http://127.0.0.1:4302 for PhantomArcade. `./target/release/necro-cli observatory --serve` shows both in one page on port 4300, and the notch pill uses that one. To run the checks:
```
cargo test --release
./target/release/necro-cli benchmark --vectors 10000
```
The notch pill needs macOS with Xcode command line tools:
```
swiftc -O apps/macos-notch/necro_notch.swift -o apps/macos-notch/necro_notch
./apps/macos-notch/necro_notch
```
Kiro CLI is optional. Without it the buttons use the offline answers and say so.
