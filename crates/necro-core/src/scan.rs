use crate::cfg::Cfg;
use crate::ir::{Expr, Inst, NecroIR, Terminator};
use crate::report::{NecroAutopsyReport, Pathology, PathologyKind};

/// Variables written anywhere in the given block set.
fn written_in(ir: &NecroIR, blocks: &[usize]) -> Vec<String> {
    let mut out = Vec::new();
    for &b in blocks {
        for i in &ir.blocks[b].insts {
            match i {
                Inst::Assign { var, .. } | Inst::Append { var, .. } => out.push(var.clone()),
                Inst::Effect { .. } => {}
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Runs the static pathology scanner over a program.
pub fn scan(ir: &NecroIR, lifter_warnings: Vec<String>) -> NecroAutopsyReport {
    let cfg = Cfg::build(ir);
    let reach = cfg.reachable();
    let mut paths: Vec<Pathology> = Vec::new();

    // Ghosts: unreachable blocks that hold real code.
    for (i, b) in ir.blocks.iter().enumerate() {
        if reach[i] {
            continue;
        }
        // Empty jumps, paragraph ends and bare halts (END-OF-PROGRAM, exorcised blocks) are not code.
        let has_code = !b.insts.is_empty() || !matches!(b.term, Terminator::Jump(_) | Terminator::ParaEnd { .. } | Terminator::Halt);
        if !has_code {
            continue;
        }
        let preds: Vec<(String, String)> = cfg.pred[i]
            .iter()
            .map(|&p| (ir.blocks[p].label.clone(), b.label.clone()))
            .collect();
        paths.push(Pathology {
            kind: PathologyKind::Ghost,
            label: "Ghost".into(),
            block: b.label.clone(),
            span: b.span,
            variable: None,
            blocks: vec![b.label.clone()],
            cfg_edges: preds,
            detail: if cfg.pred[i].is_empty() {
                format!("Block `{}` has no incoming CFG edge; nothing performs, calls or jumps to it.", b.label)
            } else {
                format!("Block `{}` is only entered from blocks that are themselves unreachable.", b.label)
            },
            suggested_fix: "Delete the block, or add the PERFORM / jump that was meant to reach it.".into(),
        });
    }

    // Loops.
    for comp in cfg.loops() {
        if !comp.iter().any(|&b| reach[b]) {
            continue;
        }
        let in_loop = |x: usize| comp.binary_search(&x).is_ok();
        let labels: Vec<String> = comp.iter().map(|&b| ir.blocks[b].label.clone()).collect();
        let written = written_in(ir, &comp);
        // Exit conditions: Branch terminators inside the loop with a target outside.
        let mut exit_edges: Vec<(String, String)> = Vec::new();
        let mut exit_cond_reads: Vec<String> = Vec::new();
        let mut has_exit = false;
        for &b in &comp {
            for &s in &cfg.succ[b] {
                if !in_loop(s) {
                    has_exit = true;
                    exit_edges.push((ir.blocks[b].label.clone(), ir.blocks[s].label.clone()));
                    if let Terminator::Branch { cond, .. } = &ir.blocks[b].term {
                        cond.reads(&mut exit_cond_reads);
                    }
                }
            }
        }
        exit_cond_reads.sort();
        exit_cond_reads.dedup();
        let header = comp[0];
        let loop_edges: Vec<(String, String)> = cfg
            .edges
            .iter()
            .filter(|(a, b)| in_loop(*a) && in_loop(*b))
            .map(|&(a, b)| (ir.blocks[a].label.clone(), ir.blocks[b].label.clone()))
            .collect();

        let exit_depends_on_loop = exit_cond_reads.iter().any(|v| written.binary_search(v).is_ok());
        // A loop whose only exits are unconditional (Perform returns, Return) is fine:
        // those are structural returns, not spins. We only call a loop a Zombie when
        // it has no exit at all, or every conditional exit is loop-invariant.
        let term_of = |label: &str| &ir.blocks[ir.blocks.iter().position(|b| b.label == label).unwrap()].term;
        let conditional_exits = exit_edges.iter().filter(|(a, _)| matches!(term_of(a), Terminator::Branch { .. })).count();
        // A GO TO out of the loop is a real unconditional exit; PERFORM returns are structural.
        let unconditional_exits = exit_edges.iter().filter(|(a, _)| matches!(term_of(a), Terminator::Jump(_))).count();
        if !has_exit || (conditional_exits > 0 && unconditional_exits == 0 && !exit_depends_on_loop) {
            let detail = if !has_exit {
                if comp.len() == 1 {
                    format!("Block `{}` jumps to itself and has no exit edge (a spin halt).", labels[0])
                } else {
                    format!("The cycle {} has no edge that leaves it.", labels.join(" -> "))
                }
            } else {
                format!(
                    "Every exit condition of the loop reads only [{}], and the loop writes only [{}]; the condition can never change once the loop is entered.",
                    exit_cond_reads.join(", "),
                    written.join(", ")
                )
            };
            paths.push(Pathology {
                kind: PathologyKind::Zombie,
                label: "Zombie".into(),
                block: ir.blocks[header].label.clone(),
                span: ir.blocks[header].span,
                variable: exit_cond_reads.first().cloned(),
                blocks: labels.clone(),
                cfg_edges: loop_edges.clone(),
                detail,
                suggested_fix: "Write the exit variable inside the loop, or bound the loop with a fuel counter.".into(),
            });
        }

        // Vampires: a variable only ever incremented (or appended) inside the loop,
        // never read by any exit condition.
        for v in &written {
            if v.starts_with("__") {
                continue;
            }
            let mut only_grows = true;
            let mut grows = false;
            let mut where_span = None;
            let mut where_block = String::new();
            for &b in &comp {
                for i in &ir.blocks[b].insts {
                    match i {
                        Inst::Assign { var, expr, span, .. } if var == v => {
                            if expr.is_positive_increment_of(v) {
                                grows = true;
                                if where_span.is_none() {
                                    where_span = Some(*span);
                                    where_block = ir.blocks[b].label.clone();
                                }
                            } else {
                                only_grows = false;
                            }
                        }
                        Inst::Append { var, span } if var == v => {
                            grows = true;
                            if where_span.is_none() {
                                where_span = Some(*span);
                                where_block = ir.blocks[b].label.clone();
                            }
                        }
                        _ => {}
                    }
                }
            }
            if grows && only_grows && !exit_cond_reads.contains(v) {
                let decl = ir.var(v);
                let capacity = decl
                    .map(|d| format!("10^{} units of {}", d.int_digits, d.name))
                    .unwrap_or_else(|| "its storage width".into());
                paths.push(Pathology {
                    kind: PathologyKind::Vampire,
                    label: "Vampire".into(),
                    block: where_block,
                    span: where_span.unwrap_or_default(),
                    variable: Some(v.clone()),
                    blocks: labels.clone(),
                    cfg_edges: loop_edges.clone(),
                    detail: format!(
                        "`{}` only grows inside the loop {} and no exit condition reads it; it will silently wrap once it passes {}.",
                        v,
                        labels.join(", "),
                        capacity
                    ),
                    suggested_fix: format!("Reset or bound `{v}` inside the loop, or make an exit condition depend on it."),
                });
            }
        }
    }

    let _ = Expr::Const(0);
    paths.sort_by_key(|a| (a.span.line, a.block.clone()));
    let requirements_markdown = NecroAutopsyReport::render_requirements(&ir.name, &ir.origin, &paths, &lifter_warnings);
    NecroAutopsyReport {
        program: ir.name.clone(),
        origin: ir.origin.clone(),
        block_count: ir.blocks.len(),
        edge_count: cfg.edges.len(),
        cfg_edges: cfg.edges_named(ir),
        pathologies: paths,
        lifter_warnings,
        requirements_markdown,
    }
}
