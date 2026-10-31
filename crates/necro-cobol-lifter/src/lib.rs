//! necro-cobol-lifter: front end for a documented COBOL-74/85 subset.
//!
//! Supported (see `COBOL-SUBSET.md` at the repo root for the full grammar):
//! * IDENTIFICATION DIVISION with PROGRAM-ID,
//! * DATA DIVISION / WORKING-STORAGE with level 01, 05 and 77 items,
//!   `PIC [S]9(n)[V9(m)]`, `PIC X(n)`, `COMP-3` (packed decimal, same fixed-point
//!   semantics, recorded as usage), `VALUE`,
//! * PROCEDURE DIVISION paragraphs with MOVE, ADD, SUBTRACT, MULTIPLY, DIVIDE,
//!   COMPUTE [ROUNDED], IF / ELSE / END-IF, PERFORM [THRU] [n TIMES | UNTIL],
//!   GO TO, DISPLAY, EXIT, STOP RUN.
//!
//! Level 77 numeric items are the program's inputs: the prover seeds them.
//!
//! The front end lowers to [`necro_core::NecroIR`]; `exec` interprets that IR with
//! COBOL fixed-point truncation rules and `wat` compiles the same IR to WebAssembly
//! text with bit-identical arithmetic.

pub mod exec;
pub mod fixed;
pub mod lower;
pub mod parse;
pub mod wat;

pub use exec::{run_ir, Outcome, Status};
pub use lower::lift;
pub use parse::{parse, Program};
pub use wat::emit_wat;

/// Parse, lower and return the IR plus front-end warnings.
pub fn lift_source(name: &str, src: &str) -> Result<(necro_core::NecroIR, Vec<String>), String> {
    let prog = parse(src)?;
    lift(name, &prog)
}
