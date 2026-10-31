# Design: COBOL lifter and prover

## Pipeline
```
.cob --tokenize--> Tok[] --parse--> Program{data, paragraphs}
     --lower--> NecroIR (one entry stub + one END block per paragraph,
                         straight-line runs coalesced into blocks)
     --exec::run_ir--> Outcome{status, vars}        (reference)
     --wat::emit_wat--> WAT --wasmtime--> run(fuel)  (lifted)
```

## Lowering of control flow
- Paragraph p gets an entry stub `P` and an end block `P.END` with
  `ParaEnd{para: p, next: entry of p+1}`.
- `PERFORM A THRU B` -> `Perform{entry: A, end_para: B, ret: continuation}`.
- `PERFORM A UNTIL c` -> header `Branch(c, cont, call)`, `call: Perform{..., ret: header}` (test before).
- `PERFORM A n TIMES` -> hidden counter `__perform_k`, init, header, decrement, call.
- `IF` -> `Branch`; `GO TO` -> `Jump` to the paragraph stub; `STOP RUN` -> `Halt`.

## Fixed point (`fixed.rs`)
One module holds the rules both executors follow: `load`, `store`, `mul`, `div`,
`literal`. The WAT emitter writes the same i64 sequence (`i64.mul`, `i64.div_s`,
`i64.rem_s`, `select` for abs). `i64.div_s` traps on zero and on MIN/-1; the
interpreter uses `checked_div` which is `None` in exactly those cases.

## WAT module shape
```
(memory (export "mem") 1)          ; var i at 8*i, PERFORM stack at 4096
(global $fuel (mut i64)) (global $sp (mut i32))
(func (export "reset_stack"))
(func (export "run") (param i64) (result i32)
  (loop $dispatch (block $Lbad (block $L{n-1} ... (block $L0 (br_table ...)) code0 ) code1 ... ) (return 3)))
```
Status codes: 0 Halted, 1 Trap (wasm trap), 2 FuelExhausted, 3 BadBlock, 4 StackOverflow.

## Correctness properties
- P1: for every vector, `status` and every raw variable are equal between the
  interpreter and the module. The benchmark prints the measured parity.
- P2: `store` output always fits the PIC width and is non-negative for unsigned
  PICs (proptest `store_never_exceeds_pic_width`).
- P3: `literal` round-trips `ip.ffff` to `ip*10^4 + ffff` (proptest).

## Timing
Per-vector microseconds are measured with `Instant` around: interpreter run;
memory writes + `reset_stack` + `run` + memory reads. Module compilation is
reported separately (`compile_us`).
