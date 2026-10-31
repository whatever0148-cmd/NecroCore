# Design: PAYROLL-BATCH-1982

Origin: cobol. Blocks: 15. Edges: 11.

## Pathologies

1. **Zombie** (non-terminating loop) at line 26, block `PROCESS-EMPLOYEE`
   - Every exit condition of the loop reads only [WS-EOF], and the loop writes only [WS-GROSS, WS-NET, WS-OT-HOURS, WS-PROCESSED, WS-TAX, WS-TOTAL-NET]; the condition can never change once the loop is entered.
   - Proposed change: Route every back edge into `PROCESS-EMPLOYEE` through a guard that decrements a fuel counter from 100000 and halts at zero.
2. **Vampire** (unbounded accumulator / buffer growth) at line 37, block `PROCESS-EMPLOYEE.32`
   - `WS-PROCESSED` only grows inside the loop PROCESS-EMPLOYEE, MAIN-PARA.UNTIL-24, MAIN-PARA.PERFORM-PROCESS-EMPLOYEE-24, PROCESS-EMPLOYEE.END, PROCESS-EMPLOYEE.32, PROCESS-EMPLOYEE.28, PROCESS-EMPLOYEE.30, PROCESS-EMPLOYEE.IF-27 and no exit condition reads it; it will silently wrap once it passes 10^3 units of WS-PROCESSED.
   - Proposed change: Insert a guard before `PROCESS-EMPLOYEE.32` that leaves the loop once `WS-PROCESSED` reaches 1000000 and record the bound in the report.
3. **Ghost** (unreachable code) at line 41, block `END-OF-BATCH.41`
   - Block `END-OF-BATCH.41` has no incoming CFG edge; nothing performs, calls or jumps to it.
   - Proposed change: Detach `END-OF-BATCH.41` (line 41, bytes 1767..1783) from the lifted module; keep the source text in the autopsy for the record.
