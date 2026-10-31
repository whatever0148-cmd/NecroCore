# Tasks: COBOL lifter and prover

- [x] 1. Tokenizer with spans, PIC strings as one token, period handling (`parse.rs`)
  - _Requirements: 1.1, 1.2_
- [x] 2. Parser for DATA and PROCEDURE divisions, conditions and COMPUTE expressions
  - _Requirements: 1.1_
- [x] 3. Fixed-point rules shared by both executors (`fixed.rs`) with proptest
  - _Requirements: 2.1, 2.2, 2.3_
- [x] 4. Lowering to NecroIR with PERFORM stack semantics (`lower.rs`)
  - _Requirements: 3.1_
- [x] 5. Interpreter with fuel and stack limit (`exec.rs`)
  - _Requirements: 2.1, 3.2_
- [x] 6. WAT emitter with br_table dispatch and memory stack (`wat.rs`)
  - _Requirements: 3.1, 3.2_
- [x] 7. `prove_cobol` with seeded edge-biased vectors, divergences and timings (`necro-prover`)
  - _Requirements: 4.1, 4.2, 4.3_
- [x] 8. Three period-style samples with deliberate defects and tests per sample
  - _Requirements: 5.1_
- [ ] 9. Phase 2: OCCURS tables and SEARCH, string MOVE semantics for PIC X
  - _Requirements: 1.1_
