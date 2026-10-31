//! CHIP-8 -> NecroIR basic blocks for the static pathology scanner.
use crate::decode::{decode, Op};
use crate::layout::ROM_BASE;
use necro_core::{BinOp, Block, BlockId, Expr, Inst, NecroIR, Span, StoreMode, Terminator, VarDecl};
use std::collections::{BTreeMap, BTreeSet};

fn word(rom: &[u8], addr: u16) -> Option<u16> {
    let i = addr as usize - ROM_BASE;
    if i + 1 < rom.len() {
        Some(((rom[i] as u16) << 8) | rom[i + 1] as u16)
    } else {
        None
    }
}

fn vname(x: u8) -> String {
    format!("V{x:X}")
}

fn span_of(addr: u16, len: usize) -> Span {
    let s = addr as usize - ROM_BASE;
    Span::new(s, s + len, s / 2 + 1)
}

fn inst_of(addr: u16, op: Op) -> Option<Inst> {
    let sp = span_of(addr, 2);
    Some(match op {
        Op::LdKk(x, kk) => Inst::Assign { var: vname(x), expr: Expr::Const(kk as i64), mode: StoreMode::Truncate, span: sp },
        Op::AddKk(x, kk) => Inst::Assign { var: vname(x), expr: Expr::bin(Expr::var(&vname(x)), BinOp::Add, Expr::Const(kk as i64)), mode: StoreMode::Truncate, span: sp },
        Op::LdVy(x, y) => Inst::Assign { var: vname(x), expr: Expr::var(&vname(y)), mode: StoreMode::Truncate, span: sp },
        Op::AddVy(x, y) => Inst::Assign { var: vname(x), expr: Expr::bin(Expr::var(&vname(x)), BinOp::Add, Expr::var(&vname(y))), mode: StoreMode::Truncate, span: sp },
        Op::Sub(x, y) => Inst::Assign { var: vname(x), expr: Expr::bin(Expr::var(&vname(x)), BinOp::Sub, Expr::var(&vname(y))), mode: StoreMode::Truncate, span: sp },
        Op::Subn(x, y) => Inst::Assign { var: vname(x), expr: Expr::bin(Expr::var(&vname(y)), BinOp::Sub, Expr::var(&vname(x))), mode: StoreMode::Truncate, span: sp },
        Op::Or(x, y) | Op::And(x, y) | Op::Xor(x, y) => Inst::Assign { var: vname(x), expr: Expr::bin(Expr::var(&vname(x)), BinOp::Mul, Expr::var(&vname(y))), mode: StoreMode::Truncate, span: sp },
        Op::Shr(x) | Op::Shl(x) => Inst::Assign { var: vname(x), expr: Expr::bin(Expr::var(&vname(x)), BinOp::Mul, Expr::Const(2)), mode: StoreMode::Truncate, span: sp },
        Op::LdI(a) => Inst::Assign { var: "I".into(), expr: Expr::Const(a as i64), mode: StoreMode::Truncate, span: sp },
        Op::AddIVx(x) => Inst::Assign { var: "I".into(), expr: Expr::bin(Expr::var("I"), BinOp::Add, Expr::var(&vname(x))), mode: StoreMode::Truncate, span: sp },
        Op::LdFVx(x) => Inst::Assign { var: "I".into(), expr: Expr::var(&vname(x)), mode: StoreMode::Truncate, span: sp },
        Op::Rnd(x, _) => Inst::Assign { var: vname(x), expr: Expr::var("RNG"), mode: StoreMode::Truncate, span: sp },
        Op::LdVxDt(x) => Inst::Assign { var: vname(x), expr: Expr::var("DT"), mode: StoreMode::Truncate, span: sp },
        Op::LdVxK(x) => Inst::Assign { var: vname(x), expr: Expr::var("KEYS"), mode: StoreMode::Truncate, span: sp },
        Op::LdDtVx(x) => Inst::Assign { var: "DT".into(), expr: Expr::var(&vname(x)), mode: StoreMode::Truncate, span: sp },
        Op::LdStVx(x) => Inst::Assign { var: "ST".into(), expr: Expr::var(&vname(x)), mode: StoreMode::Truncate, span: sp },
        Op::LdVxI(x) => Inst::Assign { var: vname(x), expr: Expr::var("MEM"), mode: StoreMode::Truncate, span: sp },
        Op::Drw(..) | Op::Cls | Op::LdBVx(_) | Op::LdIVx(_) => Inst::Effect { what: op.mnemonic(), span: sp },
        _ => return None,
    })
}

fn branch_cond(op: Op) -> Expr {
    match op {
        Op::SeKk(x, kk) => Expr::bin(Expr::var(&vname(x)), BinOp::Eq, Expr::Const(kk as i64)),
        Op::SneKk(x, kk) => Expr::bin(Expr::var(&vname(x)), BinOp::Ne, Expr::Const(kk as i64)),
        Op::SeVy(x, y) => Expr::bin(Expr::var(&vname(x)), BinOp::Eq, Expr::var(&vname(y))),
        Op::SneVy(x, y) => Expr::bin(Expr::var(&vname(x)), BinOp::Ne, Expr::var(&vname(y))),
        Op::Skp(x) => Expr::bin(Expr::var("KEYS"), BinOp::And, Expr::var(&vname(x))),
        Op::Sknp(x) => Expr::bin(Expr::bin(Expr::var("KEYS"), BinOp::And, Expr::var(&vname(x))), BinOp::Eq, Expr::Const(0)),
        _ => Expr::Const(0),
    }
}

/// Lifts a ROM to NecroIR. Reachable code is found by following control flow
/// from 0x200; gaps that decode as valid code ending in a control instruction,
/// and that no reachable `LD I` names as data, are added as orphan blocks so the
/// scanner can report them as unreachable.
pub fn lift_rom(name: &str, rom: &[u8]) -> (NecroIR, Vec<String>) {
    let end = (ROM_BASE + rom.len()) as u16;
    let mut reachable: BTreeSet<u16> = BTreeSet::new();
    let mut leaders: BTreeSet<u16> = BTreeSet::new();
    let mut data_refs: BTreeSet<u16> = BTreeSet::new();
    let mut work = vec![ROM_BASE as u16];
    leaders.insert(ROM_BASE as u16);
    let mut warnings = Vec::new();
    while let Some(a) = work.pop() {
        if a < ROM_BASE as u16 || a + 1 >= end || reachable.contains(&a) {
            continue;
        }
        if a & 1 == 1 {
            warnings.push(format!("control flow reaches odd address {a:#05X}"));
            continue;
        }
        reachable.insert(a);
        let Some(w) = word(rom, a) else { continue };
        let op = decode(w);
        match op {
            Op::Jp(t) => {
                leaders.insert(t);
                work.push(t);
            }
            Op::Call(t) => {
                leaders.insert(t);
                leaders.insert(a + 2);
                work.push(t);
                work.push(a + 2);
            }
            Op::Ret | Op::Unknown(_) => {}
            Op::JpV0(t) => {
                warnings.push(format!("indirect jump JP V0, {t:#05X} at {a:#05X}: targets are not statically known"));
            }
            Op::SeKk(..) | Op::SneKk(..) | Op::SeVy(..) | Op::SneVy(..) | Op::Skp(_) | Op::Sknp(_) => {
                leaders.insert(a + 2);
                leaders.insert(a + 4);
                work.push(a + 2);
                work.push(a + 4);
            }
            Op::LdI(t) => {
                data_refs.insert(t);
                work.push(a + 2);
            }
            _ => work.push(a + 2),
        }
    }
    // Orphan sweep.
    let mut orphan_starts: Vec<u16> = Vec::new();
    let mut a = ROM_BASE as u16;
    while a + 1 < end {
        if reachable.contains(&a) || data_refs.contains(&a) {
            a += 2;
            continue;
        }
        // candidate region: walk until a reachable address, a data ref, or a control op
        let start = a;
        let mut valid = true;
        let mut ended_in_control = false;
        let mut cur = a;
        while cur + 1 < end && !reachable.contains(&cur) && !data_refs.contains(&cur) {
            let op = decode(word(rom, cur).unwrap());
            if matches!(op, Op::Unknown(_)) {
                valid = false;
                break;
            }
            cur += 2;
            if matches!(op, Op::Jp(_) | Op::Ret) {
                ended_in_control = true;
                break;
            }
        }
        if valid && ended_in_control && cur > start {
            orphan_starts.push(start);
            leaders.insert(start);
        }
        a = if cur > start { cur } else { start + 2 };
    }
    // Build blocks: every leader starts a block; a block ends at a control op or the next leader.
    let mut all_code: BTreeSet<u16> = reachable.clone();
    for &s in &orphan_starts {
        let mut cur = s;
        loop {
            all_code.insert(cur);
            let op = decode(word(rom, cur).unwrap());
            cur += 2;
            if op.is_control() || cur + 1 >= end || reachable.contains(&cur) {
                break;
            }
        }
    }
    let leaders: Vec<u16> = leaders.into_iter().filter(|l| all_code.contains(l)).collect();
    let mut block_of: BTreeMap<u16, usize> = BTreeMap::new();
    for (i, &l) in leaders.iter().enumerate() {
        block_of.insert(l, i);
    }
    let halt_id = BlockId(leaders.len());
    let mut blocks: Vec<Block> = Vec::new();
    for (i, &start) in leaders.iter().enumerate() {
        let mut insts = Vec::new();
        let mut cur = start;
        let term;
        loop {
            let op = decode(word(rom, cur).unwrap());
            let next = cur + 2;
            if let Some(inst) = inst_of(cur, op) {
                insts.push(inst);
            }
            let target = |t: u16| block_of.get(&t).map(|&b| BlockId(b)).unwrap_or(halt_id);
            if op.is_control() {
                term = match op {
                    Op::Jp(t) => Terminator::Jump(target(t)),
                    Op::Call(t) => Terminator::Call { target: target(t), ret: target(next) },
                    Op::Ret => Terminator::Return,
                    Op::JpV0(_) | Op::Unknown(_) => Terminator::Halt,
                    _ => Terminator::Branch { cond: branch_cond(op), then_to: target(next + 2), else_to: target(next) },
                };
                break;
            }
            if next + 1 >= end {
                term = Terminator::Halt;
                break;
            }
            if block_of.contains_key(&next) {
                term = Terminator::Jump(target(next));
                break;
            }
            cur = next;
        }
        let label = if orphan_starts.contains(&start) { format!("orphan_{start:04X}") } else { format!("L{start:04X}") };
        blocks.push(Block { id: BlockId(i), label, insts, term, span: span_of(start, (cur + 2 - start) as usize) });
    }
    blocks.push(Block { id: halt_id, label: "HALT".into(), insts: vec![], term: Terminator::Halt, span: Span::default() });
    let mut variables: Vec<VarDecl> = (0..16)
        .map(|x| VarDecl { name: vname(x), int_digits: 3, scale: 0, signed: false, is_input: false, initial: 0, span: Span::default() })
        .collect();
    for extra in ["I", "DT", "ST", "KEYS", "RNG", "MEM"] {
        variables.push(VarDecl { name: extra.into(), int_digits: 4, scale: 0, signed: false, is_input: extra == "KEYS", initial: 0, span: Span::default() });
    }
    (
        NecroIR { name: name.to_string(), origin: "chip8".into(), entry: BlockId(0), blocks, variables, working_scale: 0, paragraphs: vec![] },
        warnings,
    )
}
