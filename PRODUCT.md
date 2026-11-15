# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users
Primary: teams who still maintain COBOL programs that move money (banks, payroll, clearing), whose experts are retiring and who are afraid to touch code nobody fully understands. Their job: find what is broken or dead in an old program, fix it, and prove the fixed or modernised version still behaves exactly like the original before anything ships. (Confirmed by Ahmed, 2026-11-15.)

Secondary: people who want old CHIP-8 and SUPER-CHIP games to keep running and be moddable (the PhantomArcade app).

Evaluating audience for the Kiroween 2026 entry: hackathon judges who open the hosted link for a few minutes.

## Product Purpose
NecroCore takes a dead program (a 1978-style COBOL batch job or a CHIP-8 ROM), lifts it to one intermediate form, finds three kinds of rot in its control flow (unreachable code, unbounded growth, loops that never end), asks Kiro for a fix plan, heals it, compiles it to WebAssembly, and proves with thousands of seeded inputs that the result matches the reference interpreter. Success: a maintainer can trust a change to an old money program because the proof says so.

## Positioning
Proof before trust. NecroCore never claims a program is fixed or modernised unless the prover shows every tested input still matches the original, and every number on screen comes from a saved run. (Confirmed by Ahmed, 2026-11-15.)

## Operating Context
- Runs locally: a std-only Rust daemon on 127.0.0.1 serves two apps (CryptLedger on 4301, PhantomArcade on 4302) and the combined Observatory (4300). It calls `kiro-cli` and the wasmtime prover.
- A static hosted copy (GitHub Pages, `docs/index.html`) embeds the saved reports and compiled ROMs; there the games run live in the browser and the daemon-only actions are unavailable.
- A macOS notch pill accepts a dropped COBOL file and shows the three counts.

## Capabilities and Constraints
- COBOL subset documented in `COBOL-SUBSET.md`; three sample programs from 1978, 1982, 1985 with deliberate defects.
- Four original ROMs (three CHIP-8, one SUPER-CHIP 128x64), MIT, no historical binaries.
- Kiro is used through steering, specs, hooks, an MCP server (`necro-mcp`) and `kiro-cli` turns; with no `kiro-cli` the bridge uses an offline fallback and must say so.
- The Halloween labels (Ghost, Vampire, Zombie) always sit next to the technical term (unreachable code, unbounded accumulator, non-terminating loop).

## Brand Commitments
- Name: NecroCore. App names: CryptLedger, PhantomArcade, Observatory.
- Voice for judges and users: plain, first-person student voice, no em dashes, no semicolons, no buzzwords.

## Evidence on Hand
- Benchmarks in `benchmarks/` (JSON and text), including 10,000-vector runs and a 70,000-vector run with 100% parity, live Kiro traces (`live-kiro-trace.json`, `live-kiro-mcp-trace.json`, `kiro-coding-sessions.json`), hook firing log.
- 33 passing tests.
- No real users, customers, testimonials or bank partners exist. Never invent them. Cite only sourced facts about COBOL's prevalence (the Reuters 2017 figure already cited in `deliverables/`).

## Product Principles
1. Proof before trust: no claim without a run behind it.
2. Show the rot plainly: every finding links to its exact lines.
3. The friendly label never hides the technical term.
4. Work offline and locally first: nothing about a user's program leaves their machine.
5. Honest about limits: when Kiro or the scanner cannot see something, say so.

## Accessibility & Inclusion
Readable at 390 px phones and desktop, keyboard reachable, all motion respects prefers-reduced-motion, text contrast meets WCAG AA even inside themed effects.
