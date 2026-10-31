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
