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

### Measured results (from `benchmarks/2026-11-01-register-locals.txt` and `benchmarks/2026-11-01-register-locals-longrun.txt`)
| What | Result |
|---|---|
| Differential vectors | 70,000 of 70,000 matched (7 programs, 10,000 each, seed 7) |
| COBOL, WebAssembly vs reference interpreter | 8.69x, 16.77x and 17.25x faster per vector |
| CHIP-8 and SUPER-CHIP, lifted WebAssembly vs native Rust reference interpreter | 1.61x to 2.40x faster at 300 steps per vector and 1.87x to 2.32x at 30,000 steps |
| Problems found and left after healing | 19 found, 0 left |
| Real kiro-cli turns in the live benchmark (earlier run, `benchmarks/2026-10-31-phase3.txt`) | 7 of 7, 142 s in total |
| Tests | 33 passed, 0 failed. Clippy shows 0 warnings |

## How it fits the four tracks
**Resurrection.** This is the main entry. A 1978 style interest program and a 1991 style SUPER-CHIP game come back as live WebAssembly, and the proof is a number, not a promise.

**Frankenstein.** The parts do not belong together, and they are stitched into one tool: COBOL decimal arithmetic, 8-bit game bytecode, WebAssembly, a Rust daemon with server-sent events, a Swift program that lives under the MacBook notch, and the Kiro CLI.

**Skeleton Crew.** The template is one shared engine (the intermediate form, the scanner, the healer, the prover and the Kiro bridge) plus `crates/necro-daemon`, a small server and UI shell. Two apps sit on it in separate folders. `apps/cryptledger` is the COBOL studio and `apps/phantomarcade` is the ROM lifter and live modder. Each has its own Cargo package, its own binary and its own port, and its `main.rs` is about 25 lines. A third app would cost one more folder.

**Costume Contest.** The interface treats bugs as monsters, and the monsters are the function. A Ghost row is dead code, a Zombie row is a loop that never ends. The notch pill takes a file dropped on it and shows the three counts. The page is dark and quiet. The Impeccable detector reports 0 findings, and I measured it at 1920 by 1080 and at 375 pixels wide with no sideways scrolling.

## Best Startup pitch
The paid part of legacy modernization is trust. Banks and insurers do not lack tools that translate code. They lack evidence that the translated program does what the old one did. NecroCore's prover produces that evidence as a table of vectors and a list of exact divergences, which a reviewer or an auditor can read.

**How big is the problem.** I checked these numbers and I want to be clear about how well each one is backed.
- A 2017 Thomson Reuters graphic, reported by [The Next Web on April 10, 2017](http://thenextweb.com/news/ancient-programming-language-cobol-can-make-you-bank-literally), says that in the US about 80 percent of in-person transactions and 95 percent of ATM swipes run on programs written in COBOL. I read this sentence in the article. I could not open the Reuters page itself.
- The same Reuters work is widely quoted as saying that 43 percent of banking systems are built on COBOL and that about 220 billion lines of COBOL are still in production. Many articles repeat both figures, but I only found them second hand, and they disagree about whether 43 percent means the US or the world. I list them as widely quoted, not as verified.
- [Reuters, April 11, 2017](https://www.cnbc.com/2017/04/11/banks-scramble-to-fix-old-systems-as-it-cowboys-ride-into-sunset.html) also reports that banks struggle to find people who can maintain these systems as the engineers who wrote them retire.
- What the industry spends each year on legacy systems is quoted anywhere from tens of billions to a few hundred billion dollars, depending on the scope and the year. I could not trace any single figure to a primary study, so I do not quote one.

**What NecroCore changes.** A manual rewrite of a batch job takes months or years, and the risk that scares people is silent drift in rounding and truncation. NecroCore lifts the program and compares fixed-point results against a reference interpreter, 10,000 seeded vectors per program, in about a second on a laptop. It also lists the dead code and the loops that cannot end before anyone changes a line.

**The path.** Start with small, well-defined batch programs. Sell the autopsy report and the parity proof as a service, one program at a time. Grow the COBOL subset from the real programs customers bring. I am a first-year student and this is a hackathon build, so these are plans and not traction.

## How we built it with Kiro
Kiro is in this project at three levels: it shaped the plan, it wrote a real feature, and it runs inside the product. Everything below is backed by a file in the repo.

**Spec-driven development.** Before writing the Rust I wrote four specs in `.kiro/specs/`, each with `requirements.md` in EARS form (WHEN something happens THE SYSTEM SHALL do something), `design.md` and `tasks.md`: the core engine, the COBOL prover, the ROM live patcher, and the Observatory with the notch UI. I worked down the task lists and ticked them off. The Spec button also writes new specs of the same shape from a real scan, for example `.kiro/specs/autopsy-PAYROLL-BATCH-1982/`.

**Steering.** `.kiro/steering/product.md`, `tech.md` and `structure.md` are marked `inclusion: always`. They fix the vocabulary, the rule that numbers come from real runs, and the layout of the crates and apps.

**Kiro wrote a feature from a spec.** Requirement 6 of spec 03 asks for a faster way to run a ROM, `step_batch(n, period)`, that keeps the program counter and the timer divider in WebAssembly locals and reads its key presses from a schedule table, so that one host call covers a whole test vector. I wrote the requirement, the design and two tasks. Then I drove `kiro-cli` (the V3 engine, with `.kiro/steering`, the hooks and `.kiro/settings/mcp.json` loaded) through real coding turns. In those turns Kiro:
- wrote the `step_batch` export, the inlined timer and the shared dispatch in `crates/necro-rom-lifter/src/wat.rs`,
- added the key schedule layout in `layout.rs`,
- rewrote the prover in `crates/necro-prover/src/lib.rs` to make two host calls per vector,
- wrote a new test, `step_batch_matches_step_with_no_keys`, and ticked tasks 11 and 12.

Kiro could only read and write files in these sessions, so I ran `cargo build` and `cargo test` between turns and pasted the compiler errors back to it. It fixed the two prover errors that way. After one turn that exited with an error, I added the single missing closing parenthesis myself, and the log marks that turn as a human edit. The full transcript of every turn is in `benchmarks/kiro-coding-sessions.json`.

**Hooks that Kiro fired.** The three hooks use the Kiro v1 hook schema. During the coding turns, Kiro fired `on-exorcise-defect` 18 times (a PostToolUse hook that runs the lifter tests when the scanner or the healer is edited) and `on-parity-verify` twice (a Stop hook that runs the prover). The exact payloads it received are in `benchmarks/hook-firings.log`. The third hook, `on-legacy-drop`, runs the same command from the daemon and the notch pill on every file dropped on them, and has not yet been fired from inside a Kiro session.

**MCP.** `.kiro/settings/mcp.json` defines `necro-mcp`, a small MCP server I wrote in Rust that speaks JSON-RPC over stdio, with `necro_autopsy`, `necro_parity` and `necro_hotpatch`. A real `kiro-cli` turn read the config, started the server, called `necro_autopsy` on the payroll program and reported the Zombie, the Vampire and the Ghost with their lines. The transcript is in `benchmarks/live-kiro-mcp-trace.json`.

**Kiro CLI inside the product.** The `necro-kiro-bridge` crate runs `kiro-cli chat --no-interactive` with tools switched off and a timeout, and falls back to a deterministic offline answer if the CLI is missing or fails. The Exorcise button, the `--live-kiro` benchmark and the mod box in PhantomArcade all use it. All 7 turns of the live benchmark were real (142 s in total, `benchmarks/live-kiro-trace.json`). In the mod box, "double ball speed, widen paddles, and enable zero-gravity bounce" came back from Kiro as `FRAME_DELAY 1`, `PADDLE_H 8` and `BALL_DY 0`, and the page wrote those three bytes into the running game with no reset (`benchmarks/live-mod-trace.json`).

**Property-based testing.** The COBOL lifter has `proptest` tests, and the prover is a seeded differential test of its own with 70,000 vectors.

**Who wrote what.** I wrote the specs and the steering and directed the build. Kiro wrote the `step_batch` feature and its test from spec 03 and fired the hooks listed above. I did not use Kiro Powers, custom agents or Kiro Crew.

## How I built the rest
- **Rust workspace.** Five library crates and one CLI. Rust stable, wasmtime 50, tokio, clap, serde and proptest.
- **COBOL.** A parser for a documented subset, an interpreter with fixed-point decimal rules, and a WebAssembly emitter. The interpreter and the emitter share the same rules: a working scale of 10,000 in a 64-bit integer, truncation or ROUNDED half away from zero, and high-order digits dropped to the PIC width.
- **CHIP-8 and SUPER-CHIP.** A decoder, an interpreter, a two-pass assembler with `.patch` directives, and a WebAssembly emitter with one `br_table` case per ROM address. Patchable bytes are loaded from memory at run time, so a hot patch takes effect on the next instruction.
- **Daemon.** `crates/necro-daemon` is a std-only HTTP and server-sent-events server on 127.0.0.1. It checks the Host and Origin headers. The two apps and the combined Observatory all use it.
- **Notch pill.** `apps/macos-notch/necro_notch.swift`, about 300 lines of AppKit, compiled with `swiftc`.

## Challenges
- My first control-flow graph let a PERFORMed paragraph fall through into the next one, which made every loop look like it had an exit. I rebuilt it so that it tracks the call context.
- The static scanner cannot see that BANK-INTEREST fails to finish on 2,845 of 10,000 inputs. Two loops share one paragraph and merge. The prover catches it. I wrote this limit into the spec instead of hiding it.
- ROM steps are only a few nanoseconds of work in a native interpreter that is already tight. Kiro wrote the batch entry point, and I then added fused basic blocks, registers held in WebAssembly locals and row-wise sprite drawing. Together they took the ROM speedup from about 1x to 1.6x to 2.4x with the same 70,000 of 70,000 parity. Draw and scroll routines cost about the same in both runtimes, which is the next place to look.
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
