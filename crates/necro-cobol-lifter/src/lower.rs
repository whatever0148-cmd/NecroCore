//! Lowers a parsed COBOL program to NecroIR blocks.
use crate::fixed;
use crate::parse::{Arith, Cond, CondOp, Program, Stmt};
use necro_core::{BinOp, Block, BlockId, Expr, Inst, NecroIR, Span, StoreMode, Terminator, VarDecl};
use std::collections::HashMap;

struct Lowerer<'a> {
    prog: &'a Program,
    blocks: Vec<Block>,
    para_index: HashMap<String, usize>,
    para_entry: Vec<BlockId>,
    alnum: Vec<String>,
    hidden: usize,
    warnings: Vec<String>,
    vars: Vec<VarDecl>,
}

impl<'a> Lowerer<'a> {
    fn new_block(&mut self, label: String, span: Span) -> BlockId {
        let id = BlockId(self.blocks.len());
        self.blocks.push(Block { id, label, insts: vec![], term: Terminator::Halt, span });
        id
    }

    fn is_alnum(&self, v: &str) -> bool {
        self.alnum.iter().any(|a| a == v)
    }

    fn arith(&mut self, a: &Arith) -> Result<Expr, String> {
        Ok(match a {
            Arith::Num(n) => Expr::Const(fixed::literal(n).ok_or_else(|| format!("bad numeric literal {n}"))?),
            Arith::Var(v) => {
                if self.vars.iter().all(|d| &d.name != v) {
                    return Err(format!("unknown data item {v}"));
                }
                Expr::var(v)
            }
            Arith::Str(_) => Expr::Const(0),
            Arith::Neg(x) => Expr::bin(Expr::Const(0), BinOp::Sub, self.arith(x)?),
            Arith::Bin(l, op, r) => {
                let op = match op {
                    '+' => BinOp::Add,
                    '-' => BinOp::Sub,
                    '*' => BinOp::Mul,
                    _ => BinOp::Div,
                };
                Expr::bin(self.arith(l)?, op, self.arith(r)?)
            }
        })
    }

    fn cond(&mut self, c: &Cond) -> Result<Expr, String> {
        Ok(match c {
            Cond::Cmp(l, op, r) => {
                let op = match op {
                    CondOp::Eq => BinOp::Eq,
                    CondOp::Ne => BinOp::Ne,
                    CondOp::Lt => BinOp::Lt,
                    CondOp::Le => BinOp::Le,
                    CondOp::Gt => BinOp::Gt,
                    CondOp::Ge => BinOp::Ge,
                };
                Expr::bin(self.arith(l)?, op, self.arith(r)?)
            }
            Cond::And(a, b) => Expr::bin(self.cond(a)?, BinOp::And, self.cond(b)?),
            Cond::Or(a, b) => Expr::bin(self.cond(a)?, BinOp::Or, self.cond(b)?),
            Cond::Not(a) => Expr::bin(self.cond(a)?, BinOp::Eq, Expr::Const(0)),
        })
    }

    fn para(&self, name: &str, span: Span) -> Result<usize, String> {
        self.para_index.get(name).copied().ok_or_else(|| format!("line {}: unknown paragraph {name}", span.line))
    }

    /// Lowers a statement list so that control continues at `cont`; returns the entry block.
    fn lower_seq(&mut self, stmts: &[Stmt], para: usize, cont: BlockId) -> Result<BlockId, String> {
        let mut entry = cont;
        let mut pending: Vec<Inst> = Vec::new();
        let pname = self.prog.paragraphs[para].name.clone();
        let flush = |me: &mut Self, pending: &mut Vec<Inst>, entry: BlockId| -> BlockId {
            if pending.is_empty() {
                return entry;
            }
            pending.reverse();
            let span = pending[0].span();
            let id = me.new_block(format!("{pname}.{}", span.line), span);
            me.blocks[id.0].insts = std::mem::take(pending);
            me.blocks[id.0].term = Terminator::Jump(entry);
            id
        };
        for s in stmts.iter().rev() {
            match s {
                Stmt::Move { src, dst, span } => {
                    if self.is_alnum(dst) || matches!(src, Arith::Str(_)) {
                        pending.push(Inst::Effect { what: format!("MOVE to {dst}"), span: *span });
                    } else {
                        let expr = self.arith(src)?;
                        pending.push(Inst::Assign { var: dst.clone(), expr, mode: StoreMode::Truncate, span: *span });
                    }
                }
                Stmt::Arith { dst, expr, rounded, span } => {
                    let expr = self.arith(expr)?;
                    if self.vars.iter().all(|d| &d.name != dst) {
                        return Err(format!("line {}: unknown data item {dst}", span.line));
                    }
                    pending.push(Inst::Assign {
                        var: dst.clone(),
                        expr,
                        mode: if *rounded { StoreMode::Round } else { StoreMode::Truncate },
                        span: *span,
                    });
                }
                Stmt::Display { span } => pending.push(Inst::Effect { what: "DISPLAY".into(), span: *span }),
                Stmt::Exit { .. } => {}
                Stmt::StopRun { span } => {
                    // Statements after STOP RUN never run; they stay in the IR as a detached block.
                    flush(self, &mut pending, entry);
                    let b = self.new_block(format!("{pname}.STOP-RUN"), *span);
                    self.blocks[b.0].term = Terminator::Halt;
                    entry = b;
                }
                Stmt::GoTo { para: target, span } => {
                    flush(self, &mut pending, entry);
                    let t = self.para(target, *span)?;
                    let b = self.new_block(format!("{pname}.GOTO-{target}"), *span);
                    self.blocks[b.0].term = Terminator::Jump(self.para_entry[t]);
                    entry = b;
                }
                Stmt::If { cond, then, els, span } => {
                    entry = flush(self, &mut pending, entry);
                    let then_e = self.lower_seq(then, para, entry)?;
                    let else_e = if els.is_empty() { entry } else { self.lower_seq(els, para, entry)? };
                    let c = self.cond(cond)?;
                    let b = self.new_block(format!("{pname}.IF-{}", span.line), *span);
                    self.blocks[b.0].term = Terminator::Branch { cond: c, then_to: then_e, else_to: else_e };
                    entry = b;
                }
                Stmt::Perform { first, last, times, until, span } => {
                    entry = flush(self, &mut pending, entry);
                    let f = self.para(first, *span)?;
                    let l = match last {
                        Some(l) => self.para(l, *span)?,
                        None => f,
                    };
                    let target_entry = self.para_entry[f];
                    if let Some(u) = until {
                        let c = self.cond(u)?;
                        let h = self.new_block(format!("{pname}.UNTIL-{}", span.line), *span);
                        let call = self.new_block(format!("{pname}.PERFORM-{first}-{}", span.line), *span);
                        self.blocks[call.0].term = Terminator::Perform { entry: target_entry, end_para: l, ret: h };
                        self.blocks[h.0].term = Terminator::Branch { cond: c, then_to: entry, else_to: call };
                        entry = h;
                    } else if let Some(n) = times {
                        let k = format!("__perform_{}", self.hidden);
                        self.hidden += 1;
                        self.vars.push(VarDecl { name: k.clone(), int_digits: 18, scale: 0, signed: true, is_input: false, initial: 0, span: *span });
                        let n_expr = self.arith(n)?;
                        let h = self.new_block(format!("{pname}.TIMES-{}", span.line), *span);
                        let call = self.new_block(format!("{pname}.PERFORM-{first}-{}", span.line), *span);
                        self.blocks[call.0].insts.push(Inst::Assign { var: k.clone(), expr: Expr::bin(Expr::var(&k), BinOp::Sub, Expr::Const(1)), mode: StoreMode::Truncate, span: *span });
                        self.blocks[call.0].term = Terminator::Perform { entry: target_entry, end_para: l, ret: h };
                        self.blocks[h.0].term = Terminator::Branch { cond: Expr::bin(Expr::var(&k), BinOp::Gt, Expr::Const(0)), then_to: call, else_to: entry };
                        let init = self.new_block(format!("{pname}.TIMES-INIT-{}", span.line), *span);
                        self.blocks[init.0].insts.push(Inst::Assign { var: k, expr: n_expr, mode: StoreMode::Truncate, span: *span });
                        self.blocks[init.0].term = Terminator::Jump(h);
                        entry = init;
                    } else {
                        let call = self.new_block(format!("{pname}.PERFORM-{first}-{}", span.line), *span);
                        self.blocks[call.0].term = Terminator::Perform { entry: target_entry, end_para: l, ret: entry };
                        entry = call;
                    }
                }
            }
        }
        Ok(flush(self, &mut pending, entry))
    }
}

/// Lifts a parsed program into NecroIR. Returns the IR and front-end warnings.
pub fn lift(name: &str, prog: &Program) -> Result<(NecroIR, Vec<String>), String> {
    let mut vars = Vec::new();
    let mut alnum = Vec::new();
    let mut warnings = Vec::new();
    let mut comp3_decimal: Vec<String> = Vec::new();
    for d in &prog.data {
        if d.alnum_len.is_some() {
            alnum.push(d.name.clone());
            continue;
        }
        let initial = match &d.value {
            Some(v) => {
                let w = fixed::literal(v).ok_or_else(|| format!("line {}: bad VALUE {v}", d.span.line))?;
                fixed::store(w, &VarDecl { name: d.name.clone(), int_digits: d.int_digits, scale: d.scale, signed: d.signed, is_input: false, initial: 0, span: d.span }, StoreMode::Truncate)
            }
            None => 0,
        };
        if d.int_digits == 2 && d.scale == 0 && (d.name.contains("YY") || d.name.contains("YEAR")) {
            warnings.push(format!(
                "two-digit year field `{}` (PIC 99, line {}): dates after 1999 sort before 1978 and age arithmetic goes negative",
                d.name, d.span.line
            ));
        }
        if d.comp3 && d.scale > 0 {
            comp3_decimal.push(d.name.clone());
        }
        vars.push(VarDecl {
            name: d.name.clone(),
            int_digits: d.int_digits,
            scale: d.scale,
            signed: d.signed,
            is_input: d.level == 77,
            initial,
            span: d.span,
        });
    }
    if !comp3_decimal.is_empty() {
        warnings.push(format!(
            "COMP-3 fields with decimal places [{}]: every store truncates unless ROUNDED is written, so repeated arithmetic drifts by up to one unit in the last place per store",
            comp3_decimal.join(", ")
        ));
    }
    let mut lw = Lowerer { prog, blocks: vec![], para_index: HashMap::new(), para_entry: vec![], alnum, hidden: 0, warnings: vec![], vars };
    for (i, p) in prog.paragraphs.iter().enumerate() {
        lw.para_index.insert(p.name.clone(), i);
    }
    for p in &prog.paragraphs {
        let id = lw.new_block(p.name.clone(), p.span);
        lw.para_entry.push(id);
    }
    let end = lw.new_block("END-OF-PROGRAM".into(), Span::default());
    lw.blocks[end.0].term = Terminator::Halt;
    for (i, p) in prog.paragraphs.iter().enumerate() {
        let next = if i + 1 < prog.paragraphs.len() { lw.para_entry[i + 1] } else { end };
        let pend = lw.new_block(format!("{}.END", p.name), p.span);
        lw.blocks[pend.0].term = Terminator::ParaEnd { para: i, next };
        let body = lw.lower_seq(&p.stmts, i, pend)?;
        let e = lw.para_entry[i];
        lw.blocks[e.0].term = Terminator::Jump(body);
    }
    warnings.extend(lw.warnings);
    let entry = lw.para_entry.first().copied().unwrap_or(end);
    Ok((
        NecroIR {
            name: name.to_string(),
            origin: "cobol".into(),
            entry,
            blocks: lw.blocks,
            variables: lw.vars,
            working_scale: fixed::W,
            paragraphs: prog.paragraphs.iter().map(|p| p.name.clone()).collect(),
        },
        warnings,
    ))
}
