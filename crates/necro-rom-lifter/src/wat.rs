//! CHIP-8 -> WebAssembly text. One compiled case per even rom address; patchable
//! immediates are loaded from the machine image at run time so a hot patch takes
//! effect on the next instruction without touching any other state.
//!
//! Two exports:
//! * `step(n)` runs one instruction per host call, reloading the program counter and
//!   calling the `$tick` helper each time. Its output is unchanged by the step_batch
//!   refactor so every existing test keeps its meaning.
//! * `step_batch(n, period)` runs up to `n` instructions inside WebAssembly, keeping the
//!   program counter and the timer divider in locals, and reads the key mask for each
//!   block of `period` instructions from the schedule table at `KEY_SCHEDULE`. It
//!   returns the same status codes as `step`.
use crate::decode::{decode, Op};
use crate::layout::*;
use crate::patch::{PatchField, PatchTable};
use std::fmt::Write;

fn v(x: u8) -> String {
    format!("(i32.load8_u offset={} (i32.const 0))", V + x as usize)
}
fn set_v(x: u8, val: &str) -> String {
    format!("(i32.store8 offset={} (i32.const 0) {val})", V + x as usize)
}
fn ld32(a: usize) -> String {
    format!("(i32.load offset={a} (i32.const 0))")
}
fn st32(a: usize, val: &str) -> String {
    format!("(i32.store offset={a} (i32.const 0) {val})")
}

fn imm_kk(pc: u16, kk: u8, patches: &PatchTable) -> String {
    match patches.field_at(pc) {
        Some(PatchField::Kk) => format!("(i32.load8_u offset={} (i32.const 0))", pc as usize + 1),
        _ => format!("(i32.const {kk})"),
    }
}
fn imm_n(pc: u16, n: u8, patches: &PatchTable) -> String {
    match patches.field_at(pc) {
        Some(PatchField::N) => format!("(i32.and (i32.load8_u offset={} (i32.const 0)) (i32.const 15))", pc as usize + 1),
        _ => format!("(i32.const {n})"),
    }
}

/// Stores the local program counter and the local timer divider back into the machine
/// image. Used on every `step_batch` exit path so the image after a call is byte for
/// byte what the interpreter has after the same number of instructions.
fn writeback() -> String {
    format!("{} {}", st32(PC, "(local.get $pc)"), st32(TIMER_DIV, "(local.get $div)"))
}
/// Sets the next program counter. `step` writes straight to the PC field of the image;
/// `step_batch` writes to the `$np` local instead (PC is only written back on exit).
fn set_next(expr: &str, batch: bool) -> String {
    if batch { format!("(local.set $np {expr})") } else { st32(PC, expr) }
}
/// Resets the PC to the current instruction's address before a fault return. `step`
/// writes PC back to the image; `step_batch` writes PC and the timer divider back.
fn fault_reset(batch: bool) -> String {
    if batch { writeback() } else { st32(PC, "(local.get $pc)") }
}

const PRELUDE: &str = r#"
  (func $cls
    (local $i i32)
    (local.set $i (i32.const 0))
    (loop $l
      (i64.store (i32.add (i32.const 0x1100) (local.get $i)) (i64.const 0))
      (local.set $i (i32.add (local.get $i) (i32.const 8)))
      (br_if $l (i32.lt_u (local.get $i) (i32.const 1024)))))
  (func $setres (param $hi i32)
    (i32.store8 offset=0x1070 (i32.const 0) (local.get $hi))
    (call $cls))
  (func $scd (param $n i32)
    (local $stride i32) (local $h i32) (local $y i32) (local $k i32)
    (local.set $stride (select (i32.const 16) (i32.const 8) (i32.load8_u offset=0x1070 (i32.const 0))))
    (local.set $h (select (i32.const 64) (i32.const 32) (i32.load8_u offset=0x1070 (i32.const 0))))
    (if (i32.gt_u (local.get $n) (local.get $h)) (then (local.set $n (local.get $h))))
    (local.set $y (local.get $h))
    (block $done
      (loop $rows
        (br_if $done (i32.eqz (local.get $y)))
        (local.set $y (i32.sub (local.get $y) (i32.const 1)))
        (local.set $k (i32.const 0))
        (loop $cols
          (i32.store8
            (i32.add (i32.const 0x1100) (i32.add (i32.mul (local.get $y) (local.get $stride)) (local.get $k)))
            (select
              (i32.load8_u (i32.add (i32.const 0x1100) (i32.add (i32.mul (i32.sub (local.get $y) (local.get $n)) (local.get $stride)) (local.get $k))))
              (i32.const 0)
              (i32.ge_u (local.get $y) (local.get $n))))
          (local.set $k (i32.add (local.get $k) (i32.const 1)))
          (br_if $cols (i32.lt_u (local.get $k) (local.get $stride))))
        (br $rows))))
  (func $scr
    (local $stride i32) (local $h i32) (local $y i32) (local $k i32) (local $a i32)
    (local.set $stride (select (i32.const 16) (i32.const 8) (i32.load8_u offset=0x1070 (i32.const 0))))
    (local.set $h (select (i32.const 64) (i32.const 32) (i32.load8_u offset=0x1070 (i32.const 0))))
    (local.set $y (i32.const 0))
    (loop $rows
      (local.set $k (local.get $stride))
      (loop $cols
        (local.set $k (i32.sub (local.get $k) (i32.const 1)))
        (local.set $a (i32.add (i32.const 0x1100) (i32.add (i32.mul (local.get $y) (local.get $stride)) (local.get $k))))
        (i32.store8 (local.get $a)
          (i32.or
            (i32.shr_u (i32.load8_u (local.get $a)) (i32.const 4))
            (select
              (i32.shl (i32.load8_u (i32.sub (local.get $a) (i32.const 1))) (i32.const 4))
              (i32.const 0)
              (i32.gt_u (local.get $k) (i32.const 0)))))
        (br_if $cols (i32.gt_u (local.get $k) (i32.const 0))))
      (local.set $y (i32.add (local.get $y) (i32.const 1)))
      (br_if $rows (i32.lt_u (local.get $y) (local.get $h)))))
  (func $scl
    (local $stride i32) (local $h i32) (local $y i32) (local $k i32) (local $a i32)
    (local.set $stride (select (i32.const 16) (i32.const 8) (i32.load8_u offset=0x1070 (i32.const 0))))
    (local.set $h (select (i32.const 64) (i32.const 32) (i32.load8_u offset=0x1070 (i32.const 0))))
    (local.set $y (i32.const 0))
    (loop $rows
      (local.set $k (i32.const 0))
      (loop $cols
        (local.set $a (i32.add (i32.const 0x1100) (i32.add (i32.mul (local.get $y) (local.get $stride)) (local.get $k))))
        (i32.store8 (local.get $a)
          (i32.or
            (i32.shl (i32.load8_u (local.get $a)) (i32.const 4))
            (select
              (i32.shr_u (i32.load8_u (i32.add (local.get $a) (i32.const 1))) (i32.const 4))
              (i32.const 0)
              (i32.lt_u (i32.add (local.get $k) (i32.const 1)) (local.get $stride)))))
        (local.set $k (i32.add (local.get $k) (i32.const 1)))
        (br_if $cols (i32.lt_u (local.get $k) (local.get $stride))))
      (local.set $y (i32.add (local.get $y) (i32.const 1)))
      (br_if $rows (i32.lt_u (local.get $y) (local.get $h)))))
  (func $rnd (result i32)
    (local $x i32)
    (local.set $x (i32.load offset=0x1068 (i32.const 0)))
    (local.set $x (i32.xor (local.get $x) (i32.shl (local.get $x) (i32.const 13))))
    (local.set $x (i32.xor (local.get $x) (i32.shr_u (local.get $x) (i32.const 17))))
    (local.set $x (i32.xor (local.get $x) (i32.shl (local.get $x) (i32.const 5))))
    (i32.store offset=0x1068 (i32.const 0) (local.get $x))
    (i32.and (local.get $x) (i32.const 255)))
  (func $tick
    (local $d i32)
    (local.set $d (i32.add (i32.load offset=0x106C (i32.const 0)) (i32.const 1)))
    (if (i32.ge_u (local.get $d) (i32.const 8))
      (then
        (i32.store offset=0x106C (i32.const 0) (i32.const 0))
        (if (i32.load8_u offset=0x1060 (i32.const 0))
          (then (i32.store8 offset=0x1060 (i32.const 0) (i32.sub (i32.load8_u offset=0x1060 (i32.const 0)) (i32.const 1)))))
        (if (i32.load8_u offset=0x1061 (i32.const 0))
          (then (i32.store8 offset=0x1061 (i32.const 0) (i32.sub (i32.load8_u offset=0x1061 (i32.const 0)) (i32.const 1))))))
      (else (i32.store offset=0x106C (i32.const 0) (local.get $d)))))
  (func $draw (param $x i32) (param $y i32) (param $n i32)
    (local $r i32) (local $c i32) (local $bits i32) (local $idx i32) (local $a i32) (local $mask i32) (local $vf i32)
    (local $w i32) (local $h i32) (local $rows i32) (local $cols i32) (local $hi i32) (local $ip i32)
    (local.set $hi (i32.ne (i32.load8_u offset=0x1070 (i32.const 0)) (i32.const 0)))
    (local.set $w (select (i32.const 128) (i32.const 64) (local.get $hi)))
    (local.set $h (select (i32.const 64) (i32.const 32) (local.get $hi)))
    (local.set $ip (i32.load offset=0x1010 (i32.const 0)))
    (if (i32.eqz (local.get $n))
      (then (local.set $rows (select (i32.const 16) (i32.const 0) (local.get $hi))) (local.set $cols (i32.const 16)))
      (else (local.set $rows (local.get $n)) (local.set $cols (i32.const 8))))
    (local.set $vf (i32.const 0))
    (local.set $r (i32.const 0))
    (block $done
      (loop $rows_
        (br_if $done (i32.ge_u (local.get $r) (local.get $rows)))
        (if (i32.eq (local.get $cols) (i32.const 16))
          (then (local.set $bits (i32.or
            (i32.shl (i32.load8_u (i32.and (i32.add (local.get $ip) (i32.shl (local.get $r) (i32.const 1))) (i32.const 4095))) (i32.const 8))
            (i32.load8_u (i32.and (i32.add (local.get $ip) (i32.add (i32.shl (local.get $r) (i32.const 1)) (i32.const 1))) (i32.const 4095))))))
          (else (local.set $bits (i32.shl (i32.load8_u (i32.and (i32.add (local.get $ip) (local.get $r)) (i32.const 4095))) (i32.const 8)))))
        (local.set $c (i32.const 0))
        (loop $cols_
          (if (i32.and (local.get $bits) (i32.shr_u (i32.const 0x8000) (local.get $c)))
            (then
              (local.set $idx (i32.add
                (i32.mul (i32.rem_u (i32.add (local.get $y) (local.get $r)) (local.get $h)) (local.get $w))
                (i32.rem_u (i32.add (local.get $x) (local.get $c)) (local.get $w))))
              (local.set $a (i32.add (i32.const 0x1100) (i32.shr_u (local.get $idx) (i32.const 3))))
              (local.set $mask (i32.shr_u (i32.const 128) (i32.and (local.get $idx) (i32.const 7))))
              (if (i32.and (i32.load8_u (local.get $a)) (local.get $mask)) (then (local.set $vf (i32.const 1))))
              (i32.store8 (local.get $a) (i32.xor (i32.load8_u (local.get $a)) (local.get $mask)))))
          (local.set $c (i32.add (local.get $c) (i32.const 1)))
          (br_if $cols_ (i32.lt_u (local.get $c) (local.get $cols))))
        (local.set $r (i32.add (local.get $r) (i32.const 1)))
        (br $rows_)))
    (i32.store8 offset=0x100F (i32.const 0) (local.get $vf)))
"#;

/// The timer tick inlined for `step_batch`, with the divider held in the `$div` local.
/// It is `$tick` with the memory reads/writes of `TIMER_DIV` swapped for `local.get`/
/// `local.set $div` and the func wrapper dropped; the bracket structure is identical, so
/// the image effect matches `$tick` exactly while the divider never leaves the local.
const INLINE_TICK: &str = r#"      (local.set $div (i32.add (local.get $div) (i32.const 1)))
      (if (i32.ge_u (local.get $div) (i32.const 8))
        (then
          (local.set $div (i32.const 0))
          (if (i32.load8_u offset=0x1060 (i32.const 0))
            (then (i32.store8 offset=0x1060 (i32.const 0) (i32.sub (i32.load8_u offset=0x1060 (i32.const 0)) (i32.const 1)))))
          (if (i32.load8_u offset=0x1061 (i32.const 0))
            (then (i32.store8 offset=0x1061 (i32.const 0) (i32.sub (i32.load8_u offset=0x1061 (i32.const 0)) (i32.const 1))))))
        (else (local.set $div (local.get $div))))"#;

/// Emits the `br_table` dispatch and one compiled case per even ROM address. When
/// `batch` is false the output is exactly the dispatch `step` used before; when true the
/// program-counter effects go to the `$np` local and the fault resets write the divider
/// back, so `step_batch` keeps the PC in a local between instructions.
fn emit_dispatch(rom: &[u8], patches: &PatchTable, batch: bool) -> String {
    let m = rom.len() / 2;
    let skip = format!("      (if (then {}))", set_next("(i32.add (local.get $pc) (i32.const 4))", batch));
    let mut s = String::new();
    s.push_str("      (block $after\n      (block $Lbad\n");
    for i in (0..m).rev() {
        writeln!(s, "      (block $L{i}").unwrap();
    }
    s.push_str("      (br_table");
    for i in 0..m {
        write!(s, " $L{i}").unwrap();
    }
    s.push_str(" $Lbad (local.get $idx))\n");
    for i in 0..m {
        let pc = (ROM_BASE + 2 * i) as u16;
        let w = ((rom[2 * i] as u16) << 8) | rom[2 * i + 1] as u16;
        let op = decode(w);
        writeln!(s, "      ) ;; {pc:#05X} {}", op.mnemonic()).unwrap();
        let code = match op {
            Op::Cls => "      (call $cls)".to_string(),
            Op::Ret => format!(
                "      (if (i32.eqz {sp}) (then {rst} (return (i32.const 6))))\n      {dec}\n      {setpc}",
                sp = ld32(SP),
                rst = fault_reset(batch),
                dec = st32(SP, &format!("(i32.sub {} (i32.const 1))", ld32(SP))),
                setpc = set_next(&format!("(i32.load (i32.add (i32.const {STACK}) (i32.shl {} (i32.const 2))))", ld32(SP)), batch),
            ),
            Op::Jp(a) => format!("      {}", set_next(&format!("(i32.const {a})"), batch)),
            Op::Call(a) => format!(
                "      (if (i32.ge_u {sp} (i32.const 16)) (then {rst} (return (i32.const 6))))\n      (i32.store (i32.add (i32.const {STACK}) (i32.shl {sp} (i32.const 2))) (i32.add (local.get $pc) (i32.const 2)))\n      {inc}\n      {setpc}",
                sp = ld32(SP),
                rst = fault_reset(batch),
                inc = st32(SP, &format!("(i32.add {} (i32.const 1))", ld32(SP))),
                setpc = set_next(&format!("(i32.const {a})"), batch),
            ),
            Op::SeKk(x, kk) => format!("      (i32.eq {} {})\n{skip}", v(x), imm_kk(pc, kk, patches)),
            Op::SneKk(x, kk) => format!("      (i32.ne {} {})\n{skip}", v(x), imm_kk(pc, kk, patches)),
            Op::SeVy(x, y) => format!("      (i32.eq {} {})\n{skip}", v(x), v(y)),
            Op::SneVy(x, y) => format!("      (i32.ne {} {})\n{skip}", v(x), v(y)),
            Op::LdKk(x, kk) => format!("      {}", set_v(x, &imm_kk(pc, kk, patches))),
            Op::AddKk(x, kk) => format!("      {}", set_v(x, &format!("(i32.add {} {})", v(x), imm_kk(pc, kk, patches)))),
            Op::LdVy(x, y) => format!("      {}", set_v(x, &v(y))),
            Op::Or(x, y) => format!("      {}", set_v(x, &format!("(i32.or {} {})", v(x), v(y)))),
            Op::And(x, y) => format!("      {}", set_v(x, &format!("(i32.and {} {})", v(x), v(y)))),
            Op::Xor(x, y) => format!("      {}", set_v(x, &format!("(i32.xor {} {})", v(x), v(y)))),
            Op::AddVy(x, y) => format!(
                "      (local.set $t (i32.add {} {}))\n      {}\n      {}",
                v(x),
                v(y),
                set_v(x, "(local.get $t)"),
                set_v(0xF, "(i32.shr_u (local.get $t) (i32.const 8))")
            ),
            Op::Sub(x, y) => format!(
                "      (local.set $t (i32.ge_u {vx} {vy}))\n      {}\n      {}",
                set_v(x, &format!("(i32.sub {vx} {vy})", vx = v(x), vy = v(y))),
                set_v(0xF, "(local.get $t)"),
                vx = v(x),
                vy = v(y)
            ),
            Op::Subn(x, y) => format!(
                "      (local.set $t (i32.ge_u {vy} {vx}))\n      {}\n      {}",
                set_v(x, &format!("(i32.sub {vy} {vx})", vx = v(x), vy = v(y))),
                set_v(0xF, "(local.get $t)"),
                vx = v(x),
                vy = v(y)
            ),
            Op::Shr(x) => format!("      (local.set $t {vx})\n      {}\n      {}", set_v(x, "(i32.shr_u (local.get $t) (i32.const 1))"), set_v(0xF, "(i32.and (local.get $t) (i32.const 1))"), vx = v(x)),
            Op::Shl(x) => format!("      (local.set $t {vx})\n      {}\n      {}", set_v(x, "(i32.shl (local.get $t) (i32.const 1))"), set_v(0xF, "(i32.shr_u (local.get $t) (i32.const 7))"), vx = v(x)),
            Op::LdI(a) => format!("      {}", st32(I, &format!("(i32.const {a})"))),
            Op::JpV0(a) => format!("      {}", set_next(&format!("(i32.and (i32.add (i32.const {a}) {}) (i32.const 4095))", v(0)), batch)),
            Op::Rnd(x, kk) => format!("      {}", set_v(x, &format!("(i32.and (call $rnd) {})", imm_kk(pc, kk, patches)))),
            Op::Drw(x, y, n) => format!("      (call $draw {} {} {})", v(x), v(y), imm_n(pc, n, patches)),
            Op::Skp(x) => format!("      (i32.and {} (i32.shl (i32.const 1) (i32.and {} (i32.const 15))))\n{skip}", ld32(KEYS), v(x)),
            Op::Sknp(x) => format!("      (i32.eqz (i32.and {} (i32.shl (i32.const 1) (i32.and {} (i32.const 15)))))\n{skip}", ld32(KEYS), v(x)),
            Op::LdVxDt(x) => format!("      {}", set_v(x, &format!("(i32.load8_u offset={DT} (i32.const 0))"))),
            Op::LdVxK(x) => format!(
                "      (if (i32.eqz {keys}) (then {rst} (return (i32.const 1))))\n      {}",
                set_v(x, &format!("(i32.ctz {keys})", keys = ld32(KEYS))),
                keys = ld32(KEYS),
                rst = fault_reset(batch)
            ),
            Op::LdDtVx(x) => format!("      (i32.store8 offset={DT} (i32.const 0) {})", v(x)),
            Op::LdStVx(x) => format!("      (i32.store8 offset={ST} (i32.const 0) {})", v(x)),
            Op::AddIVx(x) => format!("      {}", st32(I, &format!("(i32.and (i32.add {} {}) (i32.const 4095))", ld32(I), v(x)))),
            Op::LdFVx(x) => format!("      {}", st32(I, &format!("(i32.add (i32.const {FONT_BASE}) (i32.mul (i32.const 5) (i32.and {} (i32.const 15))))", v(x)))),
            Op::LdBVx(x) => format!(
                "      (local.set $t {vx})\n      (local.set $i {i})\n      (i32.store8 (i32.and (local.get $i) (i32.const 4095)) (i32.div_u (local.get $t) (i32.const 100)))\n      (i32.store8 (i32.and (i32.add (local.get $i) (i32.const 1)) (i32.const 4095)) (i32.rem_u (i32.div_u (local.get $t) (i32.const 10)) (i32.const 10)))\n      (i32.store8 (i32.and (i32.add (local.get $i) (i32.const 2)) (i32.const 4095)) (i32.rem_u (local.get $t) (i32.const 10)))",
                vx = v(x),
                i = ld32(I)
            ),
            Op::LdIVx(x) => format!(
                "      (local.set $i (i32.const 0))\n      (loop $l (i32.store8 (i32.and (i32.add {i} (local.get $i)) (i32.const 4095)) (i32.load8_u (i32.add (i32.const {V}) (local.get $i))))\n        (local.set $i (i32.add (local.get $i) (i32.const 1)))\n        (br_if $l (i32.le_u (local.get $i) (i32.const {x}))))",
                i = ld32(I)
            ),
            Op::LdVxI(x) => format!(
                "      (local.set $i (i32.const 0))\n      (loop $l (i32.store8 (i32.add (i32.const {V}) (local.get $i)) (i32.load8_u (i32.and (i32.add {i} (local.get $i)) (i32.const 4095))))\n        (local.set $i (i32.add (local.get $i) (i32.const 1)))\n        (br_if $l (i32.le_u (local.get $i) (i32.const {x}))))",
                i = ld32(I)
            ),
            Op::Scd(n) => format!("      (call $scd (i32.const {n}))"),
            Op::Scr => "      (call $scr)".to_string(),
            Op::Scl => "      (call $scl)".to_string(),
            Op::Low => "      (call $setres (i32.const 0))".to_string(),
            Op::High => "      (call $setres (i32.const 1))".to_string(),
            Op::StRpl(x) => (0..=x.min(7)).map(|k| format!("      (i32.store8 offset={} (i32.const 0) {})", RPL + k as usize, v(k))).collect::<Vec<_>>().join("\n"),
            Op::LdRpl(x) => (0..=x.min(7)).map(|k| format!("      {}", set_v(k, &format!("(i32.load8_u offset={} (i32.const 0))", RPL + k as usize)))).collect::<Vec<_>>().join("\n"),
            Op::Unknown(_) => format!("      {} (return (i32.const 5))", fault_reset(batch)),
        };
        s.push_str(&code);
        s.push_str("\n      (br $after)\n");
    }
    s.push_str("      ) ;; Lbad\n");
    if batch {
        s.push_str(&format!("      {} (return (i32.const 3))\n", writeback()));
    } else {
        s.push_str("      (return (i32.const 3))\n");
    }
    s.push_str("      ) ;; after\n");
    s
}

/// Emits the WebAssembly text module for a ROM. Exports `mem`, `step(n: i32) -> i32`
/// and `step_batch(n: i32, period: i32) -> i32` (status codes as in
/// [`crate::interp::StepStatus`]).
pub fn emit_wat(name: &str, rom: &[u8], patches: &PatchTable) -> String {
    let m = rom.len() / 2;
    let mut s = String::new();
    writeln!(s, ";; NecroCore lift of {name} ({} bytes, {m} instruction slots, {} patch points)", rom.len(), patches.entries.len()).unwrap();
    s.push_str("(module\n  (memory (export \"mem\") 1)\n");
    s.push_str(PRELUDE);
    // step(n): one instruction per host call, PC and the timer divider reloaded from the
    // image each time. Output is unchanged by the step_batch refactor.
    s.push_str("  (func (export \"step\") (param $n i32) (result i32)\n    (local $pc i32) (local $idx i32) (local $t i32) (local $i i32)\n");
    s.push_str("    (block $stop\n    (loop $next\n");
    s.push_str("      (br_if $stop (i32.eqz (local.get $n)))\n");
    s.push_str(&format!("      (local.set $pc {})\n", ld32(PC)));
    writeln!(s, "      (if (i32.or (i32.and (local.get $pc) (i32.const 1)) (i32.or (i32.lt_u (local.get $pc) (i32.const {ROM_BASE})) (i32.ge_u (i32.add (local.get $pc) (i32.const 1)) (i32.const {})))) (then (return (i32.const 3))))", ROM_BASE + rom.len()).unwrap();
    s.push_str(&format!("      {}\n", st32(PC, "(i32.add (local.get $pc) (i32.const 2))")));
    writeln!(s, "      (local.set $idx (i32.shr_u (i32.sub (local.get $pc) (i32.const {ROM_BASE})) (i32.const 1)))").unwrap();
    s.push_str(&emit_dispatch(rom, patches, false));
    s.push_str("      (call $tick)\n      (local.set $n (i32.sub (local.get $n) (i32.const 1)))\n      (br $next)\n    ))\n    (i32.const 0)\n  )\n");
    // step_batch(n, period): runs up to n instructions inside WebAssembly, keeping the
    // program counter and the timer divider in locals and reading the key mask for each
    // block of `period` instructions from the schedule at KEY_SCHEDULE. Two host calls
    // cover a whole vector (before and after the mid-run hot patch).
    s.push_str("  (func (export \"step_batch\") (param $n i32) (param $period i32) (result i32)\n    (local $pc i32) (local $np i32) (local $idx i32) (local $t i32) (local $i i32) (local $div i32) (local $blk i32) (local $bcount i32)\n");
    s.push_str(&format!("    (local.set $pc {})\n", ld32(PC)));
    s.push_str(&format!("    (local.set $div {})\n", ld32(TIMER_DIV)));
    s.push_str(&format!("    (local.set $blk {})\n", ld32(KEY_SCHEDULE_BLOCK0)));
    s.push_str("    (local.set $bcount (local.get $period))\n");
    s.push_str("    (block $stop\n    (loop $next\n");
    s.push_str("      (br_if $stop (i32.eqz (local.get $n)))\n");
    // At the start of each block of `period` instructions, load this block's key mask
    // from the schedule table into the image's KEYS word so SKP/SKNP/LD Vx,K read it.
    s.push_str(&format!("      (if (i32.eq (local.get $bcount) (local.get $period))\n        (then (i32.store offset={KEYS} (i32.const 0) (i32.load offset={KEY_SCHEDULE} (i32.shl (local.get $blk) (i32.const 2))))))\n"));
    // A bad PC leaves the image exactly as `step` would: write PC and the divider back, return BadPc.
    writeln!(s, "      (if (i32.or (i32.and (local.get $pc) (i32.const 1)) (i32.or (i32.lt_u (local.get $pc) (i32.const {ROM_BASE})) (i32.ge_u (i32.add (local.get $pc) (i32.const 1)) (i32.const {end})))) (then {wb} (return (i32.const 3))))", end = ROM_BASE + rom.len(), wb = writeback()).unwrap();
    s.push_str("      (local.set $np (i32.add (local.get $pc) (i32.const 2)))\n");
    writeln!(s, "      (local.set $idx (i32.shr_u (i32.sub (local.get $pc) (i32.const {ROM_BASE})) (i32.const 1)))").unwrap();
    s.push_str(&emit_dispatch(rom, patches, true));
    // Advance to the next instruction, then tick the timers with the divider in a local.
    s.push_str("      (local.set $pc (local.get $np))\n");
    s.push_str(INLINE_TICK);
    s.push_str("\n      (local.set $bcount (i32.sub (local.get $bcount) (i32.const 1)))\n");
    s.push_str("      (if (i32.eqz (local.get $bcount))\n        (then\n          (local.set $blk (i32.add (local.get $blk) (i32.const 1)))\n          (local.set $bcount (local.get $period))))\n");
    s.push_str("      (local.set $n (i32.sub (local.get $n) (i32.const 1)))\n      (br $next)\n    ))\n");
    // n exhausted: write PC and the timer divider back, return Ok.
    s.push_str(&format!("    {}\n", writeback()));
    s.push_str("    (i32.const 0)\n  )\n)\n");
    s
}
