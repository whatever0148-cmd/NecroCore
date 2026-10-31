//! CHIP-8 -> WebAssembly text. One compiled case per even ROM address; patchable
//! immediates are loaded from the machine image at run time so a hot patch takes
//! effect on the next instruction without touching any other state.
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

/// Emits the WebAssembly text module for a ROM. Exports `mem` and
/// `step(n: i32) -> i32` (status code as in [`crate::interp::StepStatus`]).
pub fn emit_wat(name: &str, rom: &[u8], patches: &PatchTable) -> String {
    let m = rom.len() / 2;
    let mut s = String::new();
    writeln!(s, ";; NecroCore lift of {name} ({} bytes, {m} instruction slots, {} patch points)", rom.len(), patches.entries.len()).unwrap();
    s.push_str("(module\n  (memory (export \"mem\") 1)\n");
    s.push_str(PRELUDE);
    s.push_str("  (func (export \"step\") (param $n i32) (result i32)\n    (local $pc i32) (local $idx i32) (local $t i32) (local $i i32)\n");
    s.push_str("    (block $stop\n    (loop $next\n");
    s.push_str("      (br_if $stop (i32.eqz (local.get $n)))\n");
    s.push_str(&format!("      (local.set $pc {})\n", ld32(PC)));
    writeln!(s, "      (if (i32.or (i32.and (local.get $pc) (i32.const 1)) (i32.or (i32.lt_u (local.get $pc) (i32.const {ROM_BASE})) (i32.ge_u (i32.add (local.get $pc) (i32.const 1)) (i32.const {})))) (then (return (i32.const 3))))", ROM_BASE + rom.len()).unwrap();
    s.push_str(&format!("      {}\n", st32(PC, "(i32.add (local.get $pc) (i32.const 2))")));
    writeln!(s, "      (local.set $idx (i32.shr_u (i32.sub (local.get $pc) (i32.const {ROM_BASE})) (i32.const 1)))").unwrap();
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
        let skip = format!("      (if (then {}))", st32(PC, "(i32.add (local.get $pc) (i32.const 4))"));
        let code = match op {
            Op::Cls => "      (call $cls)".to_string(),
            Op::Ret => format!(
                "      (if (i32.eqz {sp}) (then {rst} (return (i32.const 6))))\n      {dec}\n      {setpc}",
                sp = ld32(SP),
                rst = st32(PC, "(local.get $pc)"),
                dec = st32(SP, &format!("(i32.sub {} (i32.const 1))", ld32(SP))),
                setpc = st32(PC, &format!("(i32.load (i32.add (i32.const {STACK}) (i32.shl {} (i32.const 2))))", ld32(SP)))
            ),
            Op::Jp(a) => format!("      {}", st32(PC, &format!("(i32.const {a})"))),
            Op::Call(a) => format!(
                "      (if (i32.ge_u {sp} (i32.const 16)) (then {rst} (return (i32.const 6))))\n      (i32.store (i32.add (i32.const {STACK}) (i32.shl {sp} (i32.const 2))) (i32.add (local.get $pc) (i32.const 2)))\n      {inc}\n      {setpc}",
                sp = ld32(SP),
                rst = st32(PC, "(local.get $pc)"),
                inc = st32(SP, &format!("(i32.add {} (i32.const 1))", ld32(SP))),
                setpc = st32(PC, &format!("(i32.const {a})"))
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
            Op::JpV0(a) => format!("      {}", st32(PC, &format!("(i32.and (i32.add (i32.const {a}) {}) (i32.const 4095))", v(0)))),
            Op::Rnd(x, kk) => format!("      {}", set_v(x, &format!("(i32.and (call $rnd) {})", imm_kk(pc, kk, patches)))),
            Op::Drw(x, y, n) => format!("      (call $draw {} {} {})", v(x), v(y), imm_n(pc, n, patches)),
            Op::Skp(x) => format!("      (i32.and {} (i32.shl (i32.const 1) (i32.and {} (i32.const 15))))\n{skip}", ld32(KEYS), v(x)),
            Op::Sknp(x) => format!("      (i32.eqz (i32.and {} (i32.shl (i32.const 1) (i32.and {} (i32.const 15)))))\n{skip}", ld32(KEYS), v(x)),
            Op::LdVxDt(x) => format!("      {}", set_v(x, &format!("(i32.load8_u offset={DT} (i32.const 0))"))),
            Op::LdVxK(x) => format!(
                "      (if (i32.eqz {keys}) (then {rst} (return (i32.const 1))))\n      {}",
                set_v(x, &format!("(i32.ctz {keys})", keys = ld32(KEYS))),
                keys = ld32(KEYS),
                rst = st32(PC, "(local.get $pc)")
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
            Op::Unknown(_) => format!("      {} (return (i32.const 5))", st32(PC, "(local.get $pc)")),
        };
        s.push_str(&code);
        s.push_str("\n      (br $after)\n");
    }
    s.push_str("      ) ;; Lbad\n      (return (i32.const 3))\n      ) ;; after\n");
    s.push_str("      (call $tick)\n      (local.set $n (i32.sub (local.get $n) (i32.const 1)))\n      (br $next)\n    ))\n    (i32.const 0)\n  )\n)\n");
    s
}
