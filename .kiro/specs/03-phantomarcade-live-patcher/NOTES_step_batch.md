# Implementation notes: step_batch (requirements 6, design "Superblock execution")

## Goal
- Add `step_batch(n: i32, period: i32) -> i32` export to `wat.rs`, keep `step` unchanged (req 6.5).
- Make `prove_chip8_at` use `step_batch` for the module side, two host calls per vector (req 6.6).
- Only edit `crates/necro-rom-lifter`, `crates/necro-prover`, and the task checkboxes in `tasks.md`.

## Status codes (interp.rs StepStatus)
Ok=0, WaitingForKey=1, BadPc=3, UnknownOpcode=5, StackFault=6.

## Memory map additions (layout.rs)
- `KEY_SCHEDULE: usize = 0x2000` — consecutive u32 key masks, one per `period` instructions.
- `KEY_SCHEDULE_BLOCK0: usize = 0x1FFC` — u32 starting block index step_batch reads.
  Both outside the 0x1500 image; 0x1500..0x2000 is unused; schedule at 0x2000.

## wat.rs design
### Helpers (free functions, `batch: bool` param)
- `writeback()` -> `(i32.store offset=PC (i32.const 0) (local.get $pc)) (i32.store offset=TIMER_DIV (i32.const 0) (local.get $div))`
- `set_next(expr, batch)`: batch -> `(local.set $np {expr})`; else -> `st32(PC, expr)`
- `fault_reset(batch)`: batch -> `writeback()`; else -> `st32(PC, "(local.get $pc)")`

### emit_dispatch(rom, patches, batch) -> String
Shared br_table + per-case bodies. Identical to the original dispatch when batch=false
(set_next=st32(PC,..), fault_reset=st32(PC,$pc), Lbad=(return 3)). For batch=true,
control-flow "next pc" writes go to `$np` local; fault resets write PC+div (writeback);
Lbad writes writeback + (return 3). All non-PC effects unchanged.

### step (unchanged output)
Uses emit_dispatch(false). Prologue loads $pc from memory each iter; bad-pc -> (return 3);
prologue st32(PC, pc+2); after dispatch (call $tick); n--; loop. Byte-identical to before.

### step_batch (new)
Locals: $pc $np $idx $t $i $div $blk $bcount.
- Init: $pc=load PC; $div=load TIMER_DIV; $blk=load KEY_SCHEDULE_BLOCK0; $bcount=$period.
- loop $next: br_if $stop (n==0).
  - block start (bcount==period): store schedule[blk] (0x2000 + 4*blk) into KEYS.
  - bad-pc: (then writeback (return 3)).
  - $np = $pc + 2 (default next); $idx = (pc-ROM_BASE)/2; emit_dispatch(true).
  - $pc = $np (advance).
  - inline tick (mirrors $tick exactly but div in local; else branch = set $div=$div no-op).
  - $bcount--; if bcount==0: $blk++, $bcount=$period.
  - n--; br $next.
- after loop: writeback; (i32.const 0).

Key correctness: during a run PC lives in $pc/$np (not memory); written back on EVERY
exit (fault returns inline via writeback; normal exit at end). Image byte-for-byte
matches interpreter after same instruction count (req 6.3). Fault stops at once and
leaves image as `step` would (req 6.4): on fault $pc is still current (not advanced),
writeback stores current -> matches interpreter's set32(PC, pc).

### INLINE_TICK const (raw string), mirrors $tick body with substitutions:
- `(i32.load offset=0x106C (i32.const 0))` -> `(local.get $div)`
- `(i32.store offset=0x106C (i32.const 0) (i32.const 0))` -> `(local.set $div (i32.const 0))`
- `(else (i32.store offset=0x106C (i32.const 0) (local.get $d)))` -> `(else (local.set $div (local.get $div)))`
- DT/ST ifs unchanged (0x1060/0x1061).
- Removed `(func $tick`, `(local $d i32)`, final func-close `)`. Net balanced.
Paren counts per line (verified vs PRELUDE):
- set div=div+1 line: `1)))` (3: const1, add, set)
- if ge_u line: `8))` (2: const8, ge_u) [if stays open]
- set div=0 line: `0))` (2: const, set)
- DT if line: `0))` (2) ; DT then+store line: `1)))))` (5: const1, sub, store8, then, if)
- ST if line: `0))` ; ST then+store line: `1))))))` (6: +outer-then close)
- else line: `div))))` (4: local.get, set, else, if)  [no func close]

## prover prove_chip8_at design
- `patch_at = ((steps/2 + KEY_PERIOD/2) / KEY_PERIOD) * KEY_PERIOD; patch_at = patch_at.min(steps);`
  Round to nearest multiple of KEY_PERIOD so the two calls split on a block boundary
  and the second call's key changes align with the interpreter's (keys[done/25] at
  multiples of 25). For steps=300 -> 150 (exact). deviation < 25 for odd cases.
- Interpreter side: unchanged except uses rounded patch_at (still keys[done/KEY_PERIOD]
  per chunk clipped at patch_at). Both sides use same patch_at => parity preserved.
- Replace `step` handle with `step_batch` (typed (i32,i32)->i32). Remove `step` (clippy).
- Module side per vector:
  1. mem.write(0, fresh.image).
  2. Write whole key schedule once at 0x2000: keys[i] as u32.
  3. Write 0u32 at KEY_SCHEDULE_BLOCK0; call step_batch(patch_at, KEY_PERIOD).
  4. If code!=0: b_status=from_code (interpreter halted in first half too).
     else: hot-patch module image (read image, apply byte, write byte) exactly as interp;
     write blk0=(patch_at/KEY_PERIOD) as u32 at KEY_SCHEDULE_BLOCK0;
     call step_batch(steps-patch_at, KEY_PERIOD); if code2!=0 b_status=from_code.
  5. mem.read(0, wasm_image). Compare a.image==wasm_image && a_status==b_status.
- blk always in range: patch_at multiple of KEY_PERIOD => last blk = ceil(steps/period)-1.
- patch_at < steps for steps>=1 (nearest multiple of 25 to steps/2 is < steps); degenerate
  small steps -> patch_at=0 (first call no-op, patch at step 0, both consistent).
- n==0 first call: br_if $stop exits before any KEYS write => KEYS untouched (matches
  interpreter which sets keys inside its chunk loop only when steps>0).

## Verification (human runs cargo test)
- rom.rs `run_wasm` uses `step` (unchanged) -> still passes.
- observatory `page_embeds_every_sample` compiles emit_wat (now +step_batch) for all
  4 ROMs -> catches any WAT syntax error.
- prove parity must stay 100% (no test asserts it directly, but benchmark + mcp rely).
- clippy must be clean (no unused `step` var; no warnings).

## tasks.md
- Check box 11 and 12.
