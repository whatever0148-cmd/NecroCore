---
inclusion: fileMatch
fileMatchPattern: ["crates/necro-rom-lifter/**", "samples/roms/**"]
---

# CHIP-8 and SUPER-CHIP behaviour we chose

Old interpreters disagree on a few opcodes. NecroCore picks one behaviour and the
interpreter (`interp.rs`) and the lifter (`wat.rs`) must match it bit for bit.

- `8xy6` and `8xyE` shift Vx in place (Vy is ignored). VF gets the bit shifted out,
  written after the result.
- `8xy4`, `8xy5`, `8xy7` write VF after the result, so VF as a destination ends
  up holding the flag.
- `Fx55` and `Fx65` leave I unchanged. Every memory access masks the address with
  `& 0xFFF`.
- `Fx1E` wraps I at 12 bits. `Bnnn` jumps to nnn + V0, masked to 12 bits.
- `DXYN` wraps sprites inside the row and sets VF on any collision. SUPER-CHIP
  high-res uses the 128x64 layout and 16x16 sprites when N is 0.
- `Fx0A` with no key held returns WaitingForKey and keeps PC on the opcode.
- Sound is never produced. The sound timer is only a byte.

ROMs in `samples/roms/` are our own MIT programs. Never add historical binaries.
Patch points live in `<ROM>.patch.json` and the `.asm` source, never as raw byte
edits.
