use crate::ir::{BinOp, Block, BlockId, Expr, Inst, NecroIR, Span, StoreMode, Terminator, VarDecl};
use crate::report::{NecroAutopsyReport, PathologyKind};

/// Default fuel given to a healed Zombie loop and the cap applied to a Vampire.
pub const HEAL_FUEL: i64 = 100_000;
pub const HEAL_CAP: i64 = 1_000_000;

/// Produces a healed copy of the program:
/// * Ghost blocks are detached (their terminator becomes `Halt`, so they hold no edges),
/// * every Zombie loop gets a guard block on its back edges that counts fuel and halts,
/// * every Vampire gets a guard that leaves the loop once the variable passes a cap.
///
/// Healing never changes behaviour below the guard limits; the prover can confirm
/// that by replaying the same vectors through the healed module.
///
/// Each affected loop gets exactly one merged guard: the Zombie fuel counter and every
/// Vampire cap are AND-combined into a single branch, so one guard block covers the
/// whole loop.
pub fn heal(ir: &NecroIR, report: &NecroAutopsyReport) -> NecroIR {
    let mut out = ir.clone();

    for p in &report.pathologies {
        if p.kind == PathologyKind::Ghost {
            if let Some(i) = label_of(&out, &p.block) {
                out.blocks[i].insts.clear();
                out.blocks[i].term = Terminator::Halt;
                out.blocks[i].label = format!("{}__exorcised", p.block);
            }
        }
    }
    // One guard per loop: a loop that is both a Zombie and hosts Vampires gets a
    // single guard whose condition is the AND of every bound.
    let mut groups: Vec<(Vec<String>, Vec<&crate::report::Pathology>)> = Vec::new();
    for p in report.pathologies.iter().filter(|p| p.kind != PathologyKind::Ghost) {
        match groups.iter_mut().find(|(k, _)| *k == p.blocks) {
            Some((_, v)) => v.push(p),
            None => groups.push((p.blocks.clone(), vec![p])),
        }
    }
    for (blocks, paths) in groups {
        let loop_idx: Vec<usize> = blocks.iter().filter_map(|l| label_of(&out, l)).collect();
        if loop_idx.is_empty() {
            continue;
        }
        let header = loop_idx[0];
        let first = paths[0];
        let bail = BlockId(out.blocks.len());
        out.blocks.push(Block { id: bail, label: format!("{}__bailout", first.block), insts: vec![], term: Terminator::Halt, span: first.span });
        let guard = BlockId(out.blocks.len());
        let mut insts = Vec::new();
        let mut cond: Option<Expr> = None;
        let and = |c: Expr, cond: &mut Option<Expr>| {
            *cond = Some(match cond.take() {
                Some(prev) => Expr::bin(prev, BinOp::And, c),
                None => c,
            });
        };
        for p in &paths {
            match p.kind {
                PathologyKind::Zombie => {
                    let name = format!("__necro_fuel_{}", out.blocks.len());
                    out.variables.push(VarDecl { name: name.clone(), int_digits: 18, scale: 0, signed: true, is_input: false, initial: HEAL_FUEL, span: Span::default() });
                    insts.push(Inst::Assign { var: name.clone(), expr: Expr::bin(Expr::var(&name), BinOp::Sub, Expr::Const(1)), mode: StoreMode::Truncate, span: p.span });
                    and(Expr::bin(Expr::var(&name), BinOp::Gt, Expr::Const(0)), &mut cond);
                }
                PathologyKind::Vampire => {
                    let v = p.variable.clone().unwrap_or_default();
                    and(Expr::bin(Expr::var(&v), BinOp::Lt, Expr::Const(HEAL_CAP)), &mut cond);
                }
                PathologyKind::Ghost => {}
            }
        }
        out.blocks.push(Block {
            id: guard,
            label: format!("{}__guard", first.block),
            insts,
            term: Terminator::Branch { cond: cond.unwrap_or(Expr::Const(1)), then_to: BlockId(header), else_to: bail },
            span: first.span,
        });
        let header_id = BlockId(header);
        for &b in &loop_idx {
            let t = &mut out.blocks[b].term;
            match t {
                Terminator::Jump(x) if *x == header_id => *x = guard,
                Terminator::Branch { then_to, else_to, .. } => {
                    if *then_to == header_id {
                        *then_to = guard;
                    }
                    if *else_to == header_id {
                        *else_to = guard;
                    }
                }
                Terminator::Perform { entry, ret, .. } => {
                    if *entry == header_id {
                        *entry = guard;
                    }
                    if *ret == header_id {
                        *ret = guard;
                    }
                }
                Terminator::Call { ret, .. } if *ret == header_id => *ret = guard,
                Terminator::ParaEnd { next, .. } if *next == header_id => *next = guard,
                _ => {}
            }
        }
    }
    out
}

fn label_of(ir: &NecroIR, l: &str) -> Option<usize> {
    ir.blocks.iter().position(|b| b.label == l)
}
