//! Compiles NecroIR (COBOL origin) to WebAssembly text.
//!
//! Memory layout: variable `i` is an i64 at byte `8*i`; the PERFORM stack starts at
//! `STACK_BASE` with 8 bytes per frame (`end_para: i32`, `ret: i32`).
//! Exported function `run(fuel: i64) -> i32` returns the [`crate::exec::Status`] code.
use crate::fixed::{self, W};
use necro_core::{BinOp, Expr, Inst, NecroIR, StoreMode, Terminator};
use std::fmt::Write;

pub const STACK_BASE: usize = 4096;
pub const STACK_LIMIT: usize = crate::exec::STACK_LIMIT;

fn expr(ir: &NecroIR, e: &Expr, out: &mut String) {
    match e {
        Expr::Const(c) => write!(out, "(i64.const {c})").unwrap(),
        Expr::Var(v) => {
            let i = ir.var_index(v).expect("validated");
            let d = &ir.variables[i];
            let f = fixed::pow10(W - d.scale.min(W));
            write!(out, "(i64.mul (i64.load offset={} (i32.const 0)) (i64.const {f}))", 8 * i).unwrap();
        }
        Expr::Bin(l, op, r) => {
            let mut a = String::new();
            let mut b = String::new();
            expr(ir, l, &mut a);
            expr(ir, r, &mut b);
            match op {
                BinOp::Add => write!(out, "(i64.add {a} {b})").unwrap(),
                BinOp::Sub => write!(out, "(i64.sub {a} {b})").unwrap(),
                BinOp::Mul => write!(out, "(i64.div_s (i64.mul {a} {b}) (i64.const {}))", fixed::pow10(W)).unwrap(),
                BinOp::Div => write!(out, "(i64.div_s (i64.mul {a} (i64.const {})) {b})", fixed::pow10(W)).unwrap(),
                BinOp::Eq => write!(out, "(i64.extend_i32_u (i64.eq {a} {b}))").unwrap(),
                BinOp::Ne => write!(out, "(i64.extend_i32_u (i64.ne {a} {b}))").unwrap(),
                BinOp::Lt => write!(out, "(i64.extend_i32_u (i64.lt_s {a} {b}))").unwrap(),
                BinOp::Le => write!(out, "(i64.extend_i32_u (i64.le_s {a} {b}))").unwrap(),
                BinOp::Gt => write!(out, "(i64.extend_i32_u (i64.gt_s {a} {b}))").unwrap(),
                BinOp::Ge => write!(out, "(i64.extend_i32_u (i64.ge_s {a} {b}))").unwrap(),
                BinOp::And => write!(out, "(i64.extend_i32_u (i32.and (i64.ne {a} (i64.const 0)) (i64.ne {b} (i64.const 0))))").unwrap(),
                BinOp::Or => write!(out, "(i64.extend_i32_u (i32.or (i64.ne {a} (i64.const 0)) (i64.ne {b} (i64.const 0))))").unwrap(),
            }
        }
    }
}

fn store(ir: &NecroIR, var: &str, mode: StoreMode, value: &str, out: &mut String) {
    let i = ir.var_index(var).expect("validated");
    let d = &ir.variables[i];
    let f = fixed::pow10(W - d.scale.min(W));
    let mut v = value.to_string();
    if mode == StoreMode::Round && f > 1 {
        let half = f / 2;
        v = format!("(i64.add (local.tee $t {v}) (select (i64.const {half}) (i64.const -{half}) (i64.ge_s (local.get $t) (i64.const 0))))");
    }
    v = format!("(i64.div_s {v} (i64.const {f}))");
    let digits = d.int_digits + d.scale;
    if digits < 19 {
        v = format!("(i64.rem_s {v} (i64.const {}))", fixed::pow10(digits));
    }
    if !d.signed {
        v = format!("(select (i64.sub (i64.const 0) (local.tee $t {v})) (local.get $t) (i64.lt_s (local.get $t) (i64.const 0)))");
    }
    writeln!(out, "      (i64.store offset={} (i32.const 0) {v})", 8 * i).unwrap();
}

/// Emits the WebAssembly text module for a COBOL-origin program.
pub fn emit_wat(ir: &NecroIR) -> String {
    let n = ir.blocks.len();
    let mut s = String::new();
    writeln!(s, ";; NecroCore lift of {} ({} blocks, {} variables)", ir.name, n, ir.variables.len()).unwrap();
    s.push_str("(module\n  (memory (export \"mem\") 1)\n  (global $fuel (mut i64) (i64.const 0))\n");
    writeln!(s, "  (global $sp (mut i32) (i32.const {STACK_BASE}))").unwrap();
    s.push_str("  (func $reset_stack (export \"reset_stack\")\n");
    writeln!(s, "    (global.set $sp (i32.const {STACK_BASE})))").unwrap();
    s.push_str("  (func (export \"run\") (param $fuel0 i64) (result i32)\n    (local $pc i32) (local $t i64)\n");
    s.push_str("    (global.set $fuel (local.get $fuel0))\n");
    writeln!(s, "    (local.set $pc (i32.const {}))", ir.entry.0).unwrap();
    s.push_str("    (loop $dispatch\n");
    s.push_str("      (block $Lbad\n");
    for i in (0..n).rev() {
        writeln!(s, "      (block $L{i}").unwrap();
    }
    s.push_str("      (br_table");
    for i in 0..n {
        write!(s, " $L{i}").unwrap();
    }
    s.push_str(" $Lbad (local.get $pc))\n");
    for (i, b) in ir.blocks.iter().enumerate() {
        writeln!(s, "      ) ;; L{i} = {}", b.label).unwrap();
        s.push_str("      (if (i64.le_s (global.get $fuel) (i64.const 0)) (then (return (i32.const 2))))\n");
        s.push_str("      (global.set $fuel (i64.sub (global.get $fuel) (i64.const 1)))\n");
        for inst in &b.insts {
            if let Inst::Assign { var, expr: e, mode, .. } = inst {
                let mut v = String::new();
                expr(ir, e, &mut v);
                store(ir, var, *mode, &v, &mut s);
            }
        }
        match &b.term {
            Terminator::Jump(t) => writeln!(s, "      (local.set $pc (i32.const {})) (br $dispatch)", t.0).unwrap(),
            Terminator::Branch { cond, then_to, else_to } => {
                let mut c = String::new();
                expr(ir, cond, &mut c);
                writeln!(s, "      (local.set $pc (select (i32.const {}) (i32.const {}) (i64.ne {c} (i64.const 0)))) (br $dispatch)", then_to.0, else_to.0).unwrap();
            }
            Terminator::Perform { entry, end_para, ret } => {
                writeln!(s, "      (if (i32.ge_u (global.get $sp) (i32.const {})) (then (return (i32.const 4))))", STACK_BASE + 8 * STACK_LIMIT).unwrap();
                writeln!(s, "      (i32.store (global.get $sp) (i32.const {end_para}))").unwrap();
                writeln!(s, "      (i32.store offset=4 (global.get $sp) (i32.const {}))", ret.0).unwrap();
                s.push_str("      (global.set $sp (i32.add (global.get $sp) (i32.const 8)))\n");
                writeln!(s, "      (local.set $pc (i32.const {})) (br $dispatch)", entry.0).unwrap();
            }
            Terminator::ParaEnd { para, next } => {
                writeln!(s, "      (if (i32.and (i32.gt_u (global.get $sp) (i32.const {STACK_BASE})) (i32.eq (i32.load (i32.sub (global.get $sp) (i32.const 8))) (i32.const {para})))").unwrap();
                s.push_str("        (then (global.set $sp (i32.sub (global.get $sp) (i32.const 8)))\n");
                s.push_str("              (local.set $pc (i32.load offset=4 (global.get $sp))))\n");
                writeln!(s, "        (else (local.set $pc (i32.const {}))))", next.0).unwrap();
                s.push_str("      (br $dispatch)\n");
            }
            Terminator::Halt => s.push_str("      (return (i32.const 0))\n"),
            Terminator::Call { .. } | Terminator::Return => s.push_str("      (return (i32.const 3))\n"),
        }
    }
    s.push_str("      ) ;; Lbad\n      (return (i32.const 3))\n    )\n    (i32.const 3)\n  )\n)\n");
    s
}
