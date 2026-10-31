---
inclusion: always
---

# NecroCore: product

NecroCore brings dead programs back as living WebAssembly modules and proves the
resurrection with numbers. It is our Kiroween 2026 entry in the Resurrection
category.

## Who it is for
- Maintainers of mainframe COBOL batch jobs who need to know what a 1978 program
  really does before they touch it.
- Preservationists and hobbyists who want retro bytecode (CHIP-8 first) to run on
  a modern runtime and to be patchable while it runs.
- Anyone who has to explain legacy behaviour to a reviewer with evidence, not
  with a story.

## What it does
1. **Autopsy** (`necro-core`): lifts the program to one intermediate
   representation (NecroIR), builds the control-flow graph, and reports three
   static pathologies with byte and line offsets:
   unreachable code (UI label Ghost), unbounded accumulators or buffer growth
   (Vampire), loops whose exit can never change (Zombie).
2. **Lift** (`necro-cobol-lifter`, `necro-rom-lifter`): compiles the COBOL subset
   and CHIP-8 bytecode to WebAssembly text that `wasmtime` runs.
3. **Prove** (`necro-prover`): runs 10,000 seeded vectors through the reference
   interpreter and the lifted module, reports parity, exact divergence vectors and
   measured microsecond timings.
4. **Heal** (`necro-core::heal`): detaches ghosts and bounds zombies and vampires
   with guards, then rescans to show the count went to zero.
5. **Spec** (`necro-kiro-bridge`): turns the autopsy into EARS requirements and
   patch proposals, through `kiro-cli` when it is available, offline otherwise.

## Rules that do not bend
- Every number in the README comes from `necro-cli benchmark` output on this
  machine. Nothing is typed by hand.
- No stubs, no placeholder TODOs, no fabricated telemetry.
- Plain language for judges: short sentences, no buzzwords.
- The Halloween names (Ghost, Vampire, Zombie) are UI labels. The code and the
  docs use the technical terms next to them.
