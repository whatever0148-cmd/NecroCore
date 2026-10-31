# Design: core resurrection engine

## Architecture
```
NecroIR { entry, blocks[Block { label, insts[Inst], term: Terminator, span }], variables[VarDecl] }
        |
        v
Cfg::build  --edges-->  reachable()  +  loops() (iterative Tarjan)
        |
        v
scan()  -> NecroAutopsyReport { pathologies[], cfg_edges[], lifter_warnings[], requirements_markdown }
        |
        v
heal()  -> NecroIR'  (guards inserted, ghosts detached)  -> scan() again
```

### Terminators
`Jump`, `Branch{cond, then, else}`, `Perform{entry, end_para, ret}` (COBOL),
`ParaEnd{para, next}` (COBOL), `Call{target, ret}` and `Return` (CHIP-8), `Halt`.
The CFG treats a `ParaEnd` of paragraph p as returning to every `Perform` whose
range ends at p, which is exact for structured PERFORM use. A `Return` conservatively
edges to every `Call` return point.

### Expressions
`Expr::{Const, Var, Bin}`. The IR is untyped; the front end's executor defines
the arithmetic (fixed point for COBOL, 8-bit for CHIP-8). The scanner only uses
`reads()` and `is_positive_increment_of()`.

## Correctness properties
- P1 (soundness of Ghost): a block reported as a Ghost has no path from the entry
  in the CFG. Since the CFG over-approximates dynamic control flow (every static
  edge is included and `Return` edges to all call sites), a static Ghost is never
  executed. Test: the healed program has the same prover parity as the original.
- P2 (Zombie): if every conditional exit of a cycle reads only variables that no
  block of the cycle writes, the exit condition's value is invariant across the
  cycle, so once entered the cycle cannot be left. Loops with only structural
  exits (PERFORM returns) are not Zombies.
- P3 (Vampire): a variable that only grows inside a cycle and is read by no exit
  condition has no bound inside the cycle. It will wrap at 10^(digits) (COBOL) or
  256 (CHIP-8).
- P4 (heal preserves behaviour below the bound): the guard only diverts control
  when the fuel is exhausted or the cap is reached. The prover's vectors that halt
  inside the bound give byte-identical state on the healed module.

## Known limits
- Two loops that share a performed paragraph collapse into one SCC; the Zombie
  test then sees both loops' exits. BANK-INTEREST-1978 shows this: the TIMES loop
  and the UNTIL loop share POST-MONTH, so the dynamic prover, not the scanner,
  exposes the non-termination on tiny balances.
- Nested loops are guarded at the lowest-numbered block of the SCC.

## Error handling
The scanner never fails: a malformed IR cannot be constructed through the front
ends, and `Cfg::build` tolerates dangling targets by edges to the `HALT` block.

## Testing
Unit tests in `necro-cobol-lifter/tests/lifter.rs` and
`necro-rom-lifter/tests/rom.rs` assert the expected pathology per sample;
`proptest` covers the fixed-point store rule.
