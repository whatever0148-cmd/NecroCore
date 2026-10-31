//! Reference interpreter over NecroIR with COBOL fixed-point semantics.
use crate::fixed;
use necro_core::{BinOp, Expr, Inst, NecroIR, Terminator};
use serde::{Deserialize, Serialize};

pub const STACK_LIMIT: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Halted = 0,
    Trap = 1,
    FuelExhausted = 2,
    BadBlock = 3,
    StackOverflow = 4,
}

impl Status {
    pub fn from_code(c: i32) -> Status {
        match c {
            0 => Status::Halted,
            1 => Status::Trap,
            2 => Status::FuelExhausted,
            4 => Status::StackOverflow,
            _ => Status::BadBlock,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outcome {
    pub status: Status,
    /// Raw storage values of every variable, in declaration order.
    pub vars: Vec<i64>,
    pub blocks_executed: u64,
}

fn eval(ir: &NecroIR, raw: &[i64], e: &Expr) -> Option<i64> {
    Some(match e {
        Expr::Const(c) => *c,
        Expr::Var(v) => {
            let i = ir.var_index(v)?;
            fixed::load(raw[i], &ir.variables[i])
        }
        Expr::Bin(l, op, r) => {
            let a = eval(ir, raw, l)?;
            let b = eval(ir, raw, r)?;
            match op {
                BinOp::Add => a.wrapping_add(b),
                BinOp::Sub => a.wrapping_sub(b),
                BinOp::Mul => fixed::mul(a, b),
                BinOp::Div => fixed::div(a, b)?,
                BinOp::Eq => (a == b) as i64,
                BinOp::Ne => (a != b) as i64,
                BinOp::Lt => (a < b) as i64,
                BinOp::Le => (a <= b) as i64,
                BinOp::Gt => (a > b) as i64,
                BinOp::Ge => (a >= b) as i64,
                BinOp::And => ((a != 0) && (b != 0)) as i64,
                BinOp::Or => ((a != 0) || (b != 0)) as i64,
            }
        }
    })
}

/// Runs the program from its entry with the given raw variable values and fuel
/// (one unit per block entered).
pub fn run_ir(ir: &NecroIR, mut raw: Vec<i64>, fuel: u64) -> Outcome {
    let mut fuel_left = fuel as i64;
    let mut stack: Vec<(usize, usize)> = Vec::new();
    let mut pc = ir.entry.0;
    let mut executed = 0u64;
    loop {
        if pc >= ir.blocks.len() {
            return Outcome { status: Status::BadBlock, vars: raw, blocks_executed: executed };
        }
        if fuel_left <= 0 {
            return Outcome { status: Status::FuelExhausted, vars: raw, blocks_executed: executed };
        }
        fuel_left -= 1;
        executed += 1;
        let b = &ir.blocks[pc];
        for inst in &b.insts {
            match inst {
                Inst::Assign { var, expr, mode, .. } => {
                    let v = match eval(ir, &raw, expr) {
                        Some(v) => v,
                        None => return Outcome { status: Status::Trap, vars: raw, blocks_executed: executed },
                    };
                    let i = ir.var_index(var).expect("lowering validated the variable");
                    raw[i] = fixed::store(v, &ir.variables[i], *mode);
                }
                Inst::Append { .. } | Inst::Effect { .. } => {}
            }
        }
        match &b.term {
            Terminator::Jump(t) => pc = t.0,
            Terminator::Branch { cond, then_to, else_to } => {
                let c = match eval(ir, &raw, cond) {
                    Some(c) => c,
                    None => return Outcome { status: Status::Trap, vars: raw, blocks_executed: executed },
                };
                pc = if c != 0 { then_to.0 } else { else_to.0 };
            }
            Terminator::Perform { entry, end_para, ret } => {
                if stack.len() >= STACK_LIMIT {
                    return Outcome { status: Status::StackOverflow, vars: raw, blocks_executed: executed };
                }
                stack.push((*end_para, ret.0));
                pc = entry.0;
            }
            Terminator::ParaEnd { para, next } => match stack.last() {
                Some(&(end, ret)) if end == *para => {
                    stack.pop();
                    pc = ret;
                }
                _ => pc = next.0,
            },
            Terminator::Halt => return Outcome { status: Status::Halted, vars: raw, blocks_executed: executed },
            Terminator::Call { .. } | Terminator::Return => {
                return Outcome { status: Status::BadBlock, vars: raw, blocks_executed: executed }
            }
        }
    }
}

pub fn initial_values(ir: &NecroIR) -> Vec<i64> {
    ir.variables.iter().map(|v| v.initial).collect()
}
