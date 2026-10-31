# Tasks: core resurrection engine

- [x] 1. Define `NecroIR`, `Block`, `Inst`, `Terminator`, `Expr`, `VarDecl`, `Span` (`crates/necro-core/src/ir.rs`)
  - _Requirements: 1.1, 1.2_
- [x] 2. Build the CFG with Perform return edges and iterative Tarjan SCCs (`cfg.rs`)
  - _Requirements: 2.1, 2.2_
- [x] 3. Implement the Ghost, Zombie and Vampire scans (`scan.rs`)
  - _Requirements: 3.1, 3.2, 3.3_
- [x] 4. Render EARS requirements from findings (`report.rs`)
  - _Requirements: 3.4_
- [x] 5. Implement `heal` with fuel and cap guards and verify by rescan (`heal.rs`)
  - _Requirements: 4.1, 4.2_
- [ ] 6. Phase 2: dominator-based loop headers so nested loops get one guard each
  - _Requirements: 4.1_
