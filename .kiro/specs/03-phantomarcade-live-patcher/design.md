# Design: retro bytecode emulation and live patcher

## State image
Interpreter and module share one byte layout (`layout.rs`), so parity is
`a.image == wasm_memory[0..0x1200]`. Nothing is hidden in Rust-side fields.

```
0x0000 RAM (font at 0x50, ROM at 0x200)
0x1000 V0..VF   0x1010 I   0x1014 PC   0x1018 SP   0x1020 stack[16]
0x1060 DT 0x1061 ST   0x1064 keys   0x1068 rng   0x106C timer divider
0x1100 display 64x32 bits
```

## Defined semantics (where emulators differ)
- `8xy5`: VF = 1 when Vx >= Vy. `8xy7`: VF = 1 when Vy >= Vx.
- `8xy6` / `8xyE`: shift Vx in place; VF takes the shifted-out bit.
- Timers decrement once every 8 steps.
- `Cxkk` uses xorshift32 on the image's RNG word.
- Draw wraps at both edges and sets VF on any erased pixel.
These are choices, applied identically in `interp.rs` and `wat.rs`.

## Lifting
`emit_wat` walks the ROM two bytes at a time. Each slot is a `block` in a nested
`br_table` keyed by `(pc - 0x200) / 2`. The common prologue validates the PC and
stores `pc + 2`; a case may overwrite it (jumps, skips, waits). Data bytes
become cases too; executing them behaves exactly like the interpreter decoding
them.

Patchable immediates become `i32.load8_u` from the instruction's second byte, so
`PatchTable::apply` on the module memory changes the next execution and nothing
else. DB sprite bytes need no code change at all: DRW already reads memory.

## NecroIR lifting for the scanner
Reachable instructions are found by a work list from 0x200. Leaders are jump
and call targets and the instruction after a call or skip. Gaps that decode as
valid code ending in JP or RET, and that no reachable `LD I` names, are added
as `orphan_XXXX` blocks so the scanner can report them as Ghosts. Sprite tables
are excluded by the `LD I` data references. The `JP self` halt at game over is
reported as a Zombie (spin halt), which is true and expected.

## Correctness properties
- P1: for every vector, status and the entire 4608-byte image are equal.
- P2: a hot patch changes exactly one byte of the image (assert in tests).
- P3: the assembler round-trips: `decode(encode(m)) == m` for every encodable
  mnemonic (proptest over register and immediate fields).

## Phase 2
Multi-ball and gravity presets for the Observatory, SUPER-CHIP opcodes, and the
notch UI that shows the patch table live.
