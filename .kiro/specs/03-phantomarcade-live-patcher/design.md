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

## SUPER-CHIP layout (Phase 3)
The machine image grows from 0x1200 to 0x1500 bytes. The display keeps its address (0x1100) and
now holds 1024 bytes. Low-res mode uses the first 256 bytes at 8 bytes per row, high-res mode uses
all 1024 at 16 bytes per row. The mode byte sits at 0x1070 and the 8 RPL flags at 0x1078. The
interpreter and the WebAssembly helpers ($draw, $scd, $scr, $scl, $setres) read the same mode byte,
so a mode change needs no recompilation. The Observatory reads the mode byte to size its canvas.

## Superblock execution (requirement 6)
Today `step(n)` already loops inside WebAssembly, but it reloads the program counter from memory and calls a
`$tick` helper on every instruction, and the prover crosses the host boundary about twelve times per vector to
change the key mask. The batch entry point removes both costs.
- Memory map: the key schedule table lives at 0x2000 as consecutive u32 key masks, one per `period`
  instructions. 0x2000 is outside the 0x1500-byte machine image, so the image comparison is unchanged.
- `step_batch` keeps `$pc` and the timer divider in locals. Each case still writes its own effects to memory.
  Control-flow cases update the local `$pc`. The locals are stored back to PC and TIMER_DIV on every exit path,
  including the early returns for bad PC, stack fault, unknown opcode and wait-for-key.
- The timers DT and ST still live in the image and are decremented when the divider wraps, inline, with no call.
- The key mask word at KEYS is rewritten at the start of each block, so SKP, SKNP and LD Vx,K read it as before.
- `prove_chip8_at` writes the whole key schedule once per vector, calls `step_batch` for the first half, applies
  the hot patch, and calls it again for the second half. Parity is checked exactly as before.

## Fused basic blocks (requirement 7)
Each ROM address gets, besides its single-instruction case, a fused case when the instructions that follow it
are straight-line (arithmetic, loads and stores, I and font ops, draws, scrolls, mode switches, RPL flags). The
fused case checks that `n` and the key block `bcount` both have room, runs up to six such instructions back to
back, ticks the timers once for all of them (the divider is at most 7, six ticks wrap it at most once), then
sets `$pc` to the ending instruction and runs that instruction's normal code. Branches, calls, returns, skips,
key reads, DT and ST instructions and unknown opcodes can only end a run, so faults, key reads and PC changes
happen exactly where `step` would have them. Patch points are read from memory inside the fused code, so a hot
patch inside a run takes effect on the next execution.
