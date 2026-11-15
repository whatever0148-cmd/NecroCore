---
inclusion: fileMatch
fileMatchPattern: ["samples/**/*.cob", "crates/necro-cobol-lifter/**"]
---

# The COBOL subset

The parser accepts only the documented subset. Anything else is a parse error
with a line number, never a silent guess.

#[[file:COBOL-SUBSET.md]]

## Arithmetic that must not drift
- Every value is an i64 at working scale W = 4 (four decimal places), see
  `crates/necro-cobol-lifter/src/fixed.rs`.
- Stores truncate toward zero unless the statement says ROUNDED, then cut the
  high-order digits to the PIC width, then drop the sign for unsigned PICs.
- `div` returns None exactly where WebAssembly `i64.div_s` would trap.
- The interpreter and the WAT emitter both call these functions. Change the rule
  in one place and the prover will tell you if the two sides disagree.
