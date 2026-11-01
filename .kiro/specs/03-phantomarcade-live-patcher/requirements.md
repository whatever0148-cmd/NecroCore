# Requirements: retro bytecode emulation and live patcher (PhantomArcade)

## Introduction
CHIP-8 bytecode decoded, interpreted, lifted to WebAssembly, and patched while it
runs, with parity proven between the interpreter and the module.

## Requirement 1: decoder and interpreter
1. WHEN a 16-bit word is decoded THE SYSTEM SHALL map all 35 CHIP-8 opcodes to an
   `Op` and everything else to `Unknown`.
2. WHEN the interpreter steps THE SYSTEM SHALL keep its whole state in one
   4608-byte image (RAM, V, I, PC, SP, stack, DT, ST, keys, RNG, timer divider,
   display) with the layout in `layout.rs`.
3. WHEN the program counter is odd, below 0x200 or past the ROM THE SYSTEM SHALL
   stop with status BadPc; WHEN an opcode is unknown THE SYSTEM SHALL stop with
   status UnknownOpcode and the PC left on that instruction.
4. WHEN `LD Vx, K` runs with no key pressed THE SYSTEM SHALL return
   WaitingForKey without advancing the PC.

## Requirement 2: assembler and ROMs
1. WHEN `.asm` source is assembled THE SYSTEM SHALL resolve labels in two passes,
   encode every mnemonic in the decoder's table, and emit a `.patch.json` for
   each `.patch NAME` directive.
2. WHEN a `.patch` directive precedes an instruction THE SYSTEM SHALL accept only
   immediates: `kk` of SE/SNE/LD/ADD/RND, `n` of DRW, or a DB byte.
3. WHEN the three sample ROMs run for 300 steps with no keys THE SYSTEM SHALL
   light pixels and move the ball (PONG, BREAKOUT) or the invaders (INVADERS).

## Requirement 3: lifting and hot patching
1. WHEN a ROM is lifted THE SYSTEM SHALL emit one `br_table` case per even ROM
   address and compile every instruction's semantics inline, calling shared
   helpers for CLS, RND, DRW and timers.
2. WHEN an instruction's immediate is in the patch table THE SYSTEM SHALL load
   it from the machine image at run time instead of baking it into the code.
3. WHEN a patch value is written into the image THE SYSTEM SHALL change nothing
   but that byte, so the running game keeps every register, the display and the
   stack.

## Requirement 4: parity
1. WHEN `prove_chip8` runs a vector THE SYSTEM SHALL seed the RNG, change the key
   mask every 25 steps, hot-patch every patch point halfway through, and compare
   the whole machine image and the status at the end.
2. WHEN the interpreter stops early THE SYSTEM SHALL expect the module to stop
   with the same status and image.

## Requirement 5: SUPER-CHIP
User story: as a preservationist, I want the 128x64 SUPER-CHIP extensions so that a 1991-style
high-resolution game lifts and proves the same way as the 64x32 ones.

1. WHEN the decoder reads 00FF or 00FE THE SYSTEM SHALL switch the machine between 128x64 and 64x32
   and clear the display.
2. WHEN the decoder reads 00Cn, 00FB or 00FC THE SYSTEM SHALL scroll the active display n rows
   down, 4 pixels right or 4 pixels left, shifting in blank pixels.
3. WHEN Dxy0 runs in high-resolution mode THE SYSTEM SHALL draw a 16x16 sprite from 32 bytes at I
   and set VF to 1 on any collision, and in low-resolution mode THE SYSTEM SHALL draw nothing.
4. WHEN Fx75 or Fx85 runs THE SYSTEM SHALL copy V0 to Vmin(x,7) into or out of the RPL user flags.
5. WHEN `prove_chip8` runs a SUPER-CHIP ROM THE SYSTEM SHALL compare the whole image, including the
   mode byte and the RPL flags, exactly as for CHIP-8.

## Requirement 6: multi-cycle superblock execution
User story: as a user of the lifted games, I want the module to run many instructions per host call so
that the lifted ROM is clearly faster than the interpreter and not just equal to it.

1. WHEN a ROM is lifted THE SYSTEM SHALL also export `step_batch(n: i32, period: i32) -> i32` that executes
   up to n instructions inside WebAssembly and returns the same status codes as `step`.
2. WHEN `step_batch` runs THE SYSTEM SHALL read the key mask for each block of `period` instructions from a
   key schedule table in module memory outside the compared machine image, so that one host call can cover
   a whole test vector.
3. WHEN `step_batch` runs THE SYSTEM SHALL keep the program counter and the timer divider in locals between
   instructions and write them back to the machine image before it returns, so that the image after the call
   is byte for byte what the interpreter has after the same number of instructions.
4. WHEN an instruction stops with a non-zero status THE SYSTEM SHALL stop `step_batch` at once and leave the
   image exactly as `step` would leave it.
5. WHEN `step` is called THE SYSTEM SHALL behave exactly as before, so every existing test keeps its meaning.
6. WHEN `prove_chip8` runs THE SYSTEM SHALL use `step_batch` for the module side, two host calls per vector
   (before and after the mid-run hot patch), and compare the whole image and status as in requirement 4.

## Requirement 7: fused basic blocks
1. WHEN `step_batch` reaches an address that starts a run of straight-line instructions THE SYSTEM SHALL
   run the whole run, up to six instructions plus the instruction that ends it, after one dispatch, but only
   when at least that many instructions remain in `n` and in the current key block. Otherwise THE SYSTEM
   SHALL execute one instruction as before.
2. WHEN a run is fused THE SYSTEM SHALL apply the timer ticks of its instructions together so that the
   divider, DT and ST equal what per-instruction ticking gives.
3. WHEN an instruction can fault, reads or writes DT or ST, reads the key mask, or changes the program counter
   THE SYSTEM SHALL not place it inside the run, only at its end.
4. WHEN a patch point lies inside a fused run THE SYSTEM SHALL still read its value from the machine image.

## Requirement 8: registers in locals and row-wise drawing
1. WHEN `step_batch` runs THE SYSTEM SHALL hold V0 to VF and I in WebAssembly locals for the whole call, load
   them from the machine image on entry and store them back on every exit path, so the image after the call is
   byte for byte what the interpreter has.
2. WHEN Dxyn or Dxy0 draws THE SYSTEM SHALL XOR each sprite row into at most three display bytes, wrap the row
   inside its own display row, and report a collision when any set sprite bit meets a lit pixel, with the same
   result as the pixel-by-pixel definition.
