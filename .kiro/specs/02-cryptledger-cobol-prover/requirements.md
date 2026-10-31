# Requirements: COBOL lifter and prover (CryptLedger)

## Introduction
A reference interpreter for a documented COBOL subset, a lifter from that subset
to WebAssembly text, and a differential prover that shows the two agree.

## Requirement 1: subset
1. WHEN the parser reads a program THE SYSTEM SHALL accept IDENTIFICATION,
   DATA (WORKING-STORAGE, levels 01/05/77, PIC 9/S9/V9/X, COMP-3, VALUE) and
   PROCEDURE divisions with MOVE, ADD, SUBTRACT, MULTIPLY, DIVIDE, COMPUTE
   [ROUNDED], IF/ELSE/END-IF, PERFORM [THRU] [n TIMES | UNTIL], GO TO, DISPLAY,
   EXIT and STOP RUN.
2. IF a statement outside the subset appears THEN THE SYSTEM SHALL fail with the
   line number and the statement name.

## Requirement 2: fixed-point semantics
1. WHEN a value is stored THE SYSTEM SHALL truncate to the target's scale (or
   round when ROUNDED), drop high-order digits beyond the PIC width, and drop
   the sign for unsigned PICs.
2. WHEN an expression is evaluated THE SYSTEM SHALL use 64-bit wrapping
   arithmetic at working scale 10^4 so that the interpreter and the wasm module
   compute identical bits.
3. WHEN a division by zero occurs THE SYSTEM SHALL report a Trap in the
   interpreter and the wasm module alike.

## Requirement 3: lifting
1. WHEN `emit_wat` runs THE SYSTEM SHALL emit one `br_table` case per IR block,
   a PERFORM stack in linear memory, and a fuel counter checked at every block.
2. WHEN fuel reaches zero THE SYSTEM SHALL return status 2 (FuelExhausted) from
   `run`, matching the interpreter.

## Requirement 4: differential proof
1. WHEN `prove_cobol` runs n vectors THE SYSTEM SHALL seed every level-77 item
   from the vector seed, biased toward 0, 1, max and max-1, run both executors,
   and compare status and every variable.
2. WHEN a vector diverges THE SYSTEM SHALL record its index, seed, inputs and
   both outcomes.
3. WHEN the run completes THE SYSTEM SHALL report matched count, parity percent,
   total and per-vector microseconds for each side, and the speedup ratio.

## Requirement 5: samples
1. WHEN the three sample programs are lifted THE SYSTEM SHALL find: a Ghost in
   BANK-INTEREST-1978 (AUDIT-TRAIL) with a two-digit-year warning; a Zombie and a
   Ghost in PAYROLL-BATCH-1982; a Vampire and a Ghost in WIRE-CLEARING-1985, and
   the prover shall show WIRE-CLEARING never terminates when WS-BATCH-SIZE is 99.
