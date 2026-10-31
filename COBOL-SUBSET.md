# The COBOL subset NecroCore understands

This is the whole grammar the parser in `crates/necro-cobol-lifter/src/parse.rs`
accepts. Anything else fails with a line number.

## Source form
- Free layout. A line whose first non-blank character is `*` is a comment.
- A sentence ends with a period followed by whitespace or end of line.
- Names and keywords are case-insensitive. String literals use `'` or `"`.

## IDENTIFICATION DIVISION
`PROGRAM-ID. name.` is read; everything else in the division is skipped.

## DATA DIVISION
`WORKING-STORAGE SECTION.` with items:

```
level name [PIC picture] [COMP-3 | COMPUTATIONAL-3 | PACKED-DECIMAL] [USAGE ...] [VALUE literal].
```
- Levels 01, 05 and 77. Items without a PIC are group items and define nothing.
- Pictures: `9(n)`, `S9(n)`, `9(n)V9(m)`, `V9(m)`, `99`, `X(n)`. `9` digits may be
  written out (`999`) or repeated (`9(3)`).
- Level 77 numeric items are the program's **inputs**: the prover seeds them.
- COMP-3 is recorded (it affects the drift warning) but stored like any other
  fixed-point number.
- PIC X items are alphanumeric: they can be the target of MOVE and appear in
  DISPLAY, but they take no part in arithmetic.

## PROCEDURE DIVISION
Paragraph headers are a bare name followed by a period. Statements:

| Statement | Form |
|---|---|
| MOVE | `MOVE src TO dst` |
| ADD | `ADD a TO dst [ROUNDED]` |
| SUBTRACT | `SUBTRACT a FROM dst [ROUNDED]` |
| MULTIPLY | `MULTIPLY a BY dst [ROUNDED]` |
| DIVIDE | `DIVIDE a INTO dst [ROUNDED]` or `DIVIDE a BY b GIVING dst [ROUNDED]` |
| COMPUTE | `COMPUTE dst [ROUNDED] = expr` with `+ - * /` and parentheses |
| IF | `IF cond [THEN] stmts [ELSE stmts] END-IF` (a period also closes it) |
| PERFORM | `PERFORM p [THRU q]`, `PERFORM p [THRU q] n TIMES`, `PERFORM p [THRU q] UNTIL cond` (test before) |
| GO TO | `GO TO p` |
| DISPLAY | `DISPLAY ...` (operands skipped; side effect only) |
| EXIT / CONTINUE | no-op |
| STOP RUN | halts |

Conditions: `a op b` with `=`, `>`, `<`, `>=`, `<=`, `NOT =`, `EQUAL [TO]`,
`GREATER [THAN] [OR EQUAL [TO]]`, `LESS [THAN] [OR EQUAL [TO]]`, optional `IS`,
combined with `AND`, `OR`, `NOT` and parentheses. `ZERO`, `ZEROS`, `SPACES` are
accepted figurative constants.

## Arithmetic semantics (shared by the interpreter and the WebAssembly module)
- Working scale 10^4: every expression value is an i64 holding value x 10,000.
- `+ -` wrap on overflow like `i64.add`/`i64.sub`.
- `*` is `(a * b) / 10^4` with wrapping multiply and truncating division.
- `/` is `(a * 10^4) / b`; division by zero (or MIN / -1) is a trap on both sides.
- Storing into a PIC: truncate to the target scale (or round half away from zero
  with ROUNDED), keep the low `int + scale` digits (COBOL high-order truncation),
  drop the sign for an unsigned PIC.
- Comparisons produce 0 or 1.

## Execution
- One unit of fuel per basic block entered; fuel exhaustion returns status 2.
- The PERFORM stack holds 64 frames; a deeper recursion returns status 4.
- Running off the end of the last paragraph halts.
