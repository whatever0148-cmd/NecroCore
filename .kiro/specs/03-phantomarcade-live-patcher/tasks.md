# Tasks: retro bytecode emulation and live patcher

- [x] 1. Decoder for all 35 opcodes with mnemonics (`decode.rs`)
  - _Requirements: 1.1_
- [x] 2. Flat-image interpreter with status codes (`interp.rs`, `layout.rs`)
  - _Requirements: 1.2, 1.3, 1.4_
- [x] 3. Two-pass assembler with `.patch` directives (`asm.rs`, `patch.rs`)
  - _Requirements: 2.1, 2.2_
- [x] 4. PONG-STYLE, INVADERS-STYLE, BREAKOUT-STYLE sources, binaries and licences
  - _Requirements: 2.3_
- [x] 5. WAT lifter with memory-loaded patchable immediates (`wat.rs`)
  - _Requirements: 3.1, 3.2, 3.3_
- [x] 6. NecroIR lifter with orphan sweep (`lift.rs`)
  - _Requirements: 3.1_
- [x] 7. `prove_chip8` with key schedule and mid-run hot patch (`necro-prover`)
  - _Requirements: 4.1, 4.2_
- [ ] 8. Phase 2: multi-ball patch preset, SUPER-CHIP, live patch panel
  - _Requirements: 3.3_
