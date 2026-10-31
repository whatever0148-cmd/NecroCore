# ROM licences

Every program in this directory was written from scratch for NecroCore in 2026 and
is released under the repository's MIT licence. None of them is a historical
binary, a disassembly, or a copy of a published game; the names say "STYLE"
because they are games in the style of the arcade classics, written in CHIP-8
assembly for the assembler in `crates/necro-rom-lifter/src/asm.rs`.

| File | What it is | Source |
|---|---|---|
| `PONG-STYLE.asm` / `.ch8` | Two paddles, one ball and a second ball behind a patch point, first to 9 | original, MIT |
| `INVADERS-STYLE.asm` / `.ch8` | A marching row of four invaders and a shooter | original, MIT |
| `BREAKOUT-STYLE.asm` / `.ch8` | Paddle, ball, two rows of eight bricks | original, MIT |

The `.ch8` binaries are produced by `necro-cli assemble <file>.asm`; the
`.patch.json` sidecars list the hot-patch points the assembler recorded from the
`.patch` directives.

The 4x5 hexadecimal font loaded at address 0x50 is the conventional CHIP-8 font
whose bit patterns have been reproduced in public emulator documentation since the
1990s; it is not subject to copyright as a set of 16 glyph bitmaps.
| `NECRORUNNER-1991.asm` / `.ch8` | A 128x64 SUPER-CHIP runner (key 5 jumps), the "1991" is a style, not a release date | original, MIT |
