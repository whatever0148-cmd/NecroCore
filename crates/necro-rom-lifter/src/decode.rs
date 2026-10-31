use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Op {
    Cls,
    Ret,
    Jp(u16),
    Call(u16),
    SeKk(u8, u8),
    SneKk(u8, u8),
    SeVy(u8, u8),
    LdKk(u8, u8),
    AddKk(u8, u8),
    LdVy(u8, u8),
    Or(u8, u8),
    And(u8, u8),
    Xor(u8, u8),
    AddVy(u8, u8),
    Sub(u8, u8),
    Shr(u8),
    Subn(u8, u8),
    Shl(u8),
    SneVy(u8, u8),
    LdI(u16),
    JpV0(u16),
    Rnd(u8, u8),
    Drw(u8, u8, u8),
    Skp(u8),
    Sknp(u8),
    LdVxDt(u8),
    LdVxK(u8),
    LdDtVx(u8),
    LdStVx(u8),
    AddIVx(u8),
    LdFVx(u8),
    LdBVx(u8),
    LdIVx(u8),
    LdVxI(u8),
    Unknown(u16),
}

pub fn decode(w: u16) -> Op {
    let x = ((w >> 8) & 0xF) as u8;
    let y = ((w >> 4) & 0xF) as u8;
    let kk = (w & 0xFF) as u8;
    let n = (w & 0xF) as u8;
    let nnn = w & 0xFFF;
    match w >> 12 {
        0x0 => match w {
            0x00E0 => Op::Cls,
            0x00EE => Op::Ret,
            _ => Op::Unknown(w),
        },
        0x1 => Op::Jp(nnn),
        0x2 => Op::Call(nnn),
        0x3 => Op::SeKk(x, kk),
        0x4 => Op::SneKk(x, kk),
        0x5 if n == 0 => Op::SeVy(x, y),
        0x6 => Op::LdKk(x, kk),
        0x7 => Op::AddKk(x, kk),
        0x8 => match n {
            0 => Op::LdVy(x, y),
            1 => Op::Or(x, y),
            2 => Op::And(x, y),
            3 => Op::Xor(x, y),
            4 => Op::AddVy(x, y),
            5 => Op::Sub(x, y),
            6 => Op::Shr(x),
            7 => Op::Subn(x, y),
            0xE => Op::Shl(x),
            _ => Op::Unknown(w),
        },
        0x9 if n == 0 => Op::SneVy(x, y),
        0xA => Op::LdI(nnn),
        0xB => Op::JpV0(nnn),
        0xC => Op::Rnd(x, kk),
        0xD => Op::Drw(x, y, n),
        0xE => match kk {
            0x9E => Op::Skp(x),
            0xA1 => Op::Sknp(x),
            _ => Op::Unknown(w),
        },
        0xF => match kk {
            0x07 => Op::LdVxDt(x),
            0x0A => Op::LdVxK(x),
            0x15 => Op::LdDtVx(x),
            0x18 => Op::LdStVx(x),
            0x1E => Op::AddIVx(x),
            0x29 => Op::LdFVx(x),
            0x33 => Op::LdBVx(x),
            0x55 => Op::LdIVx(x),
            0x65 => Op::LdVxI(x),
            _ => Op::Unknown(w),
        },
        _ => Op::Unknown(w),
    }
}

impl Op {
    pub fn mnemonic(&self) -> String {
        match self {
            Op::Cls => "CLS".into(),
            Op::Ret => "RET".into(),
            Op::Jp(a) => format!("JP {a:#05X}"),
            Op::Call(a) => format!("CALL {a:#05X}"),
            Op::SeKk(x, k) => format!("SE V{x:X}, {k:#04X}"),
            Op::SneKk(x, k) => format!("SNE V{x:X}, {k:#04X}"),
            Op::SeVy(x, y) => format!("SE V{x:X}, V{y:X}"),
            Op::LdKk(x, k) => format!("LD V{x:X}, {k:#04X}"),
            Op::AddKk(x, k) => format!("ADD V{x:X}, {k:#04X}"),
            Op::LdVy(x, y) => format!("LD V{x:X}, V{y:X}"),
            Op::Or(x, y) => format!("OR V{x:X}, V{y:X}"),
            Op::And(x, y) => format!("AND V{x:X}, V{y:X}"),
            Op::Xor(x, y) => format!("XOR V{x:X}, V{y:X}"),
            Op::AddVy(x, y) => format!("ADD V{x:X}, V{y:X}"),
            Op::Sub(x, y) => format!("SUB V{x:X}, V{y:X}"),
            Op::Shr(x) => format!("SHR V{x:X}"),
            Op::Subn(x, y) => format!("SUBN V{x:X}, V{y:X}"),
            Op::Shl(x) => format!("SHL V{x:X}"),
            Op::SneVy(x, y) => format!("SNE V{x:X}, V{y:X}"),
            Op::LdI(a) => format!("LD I, {a:#05X}"),
            Op::JpV0(a) => format!("JP V0, {a:#05X}"),
            Op::Rnd(x, k) => format!("RND V{x:X}, {k:#04X}"),
            Op::Drw(x, y, n) => format!("DRW V{x:X}, V{y:X}, {n}"),
            Op::Skp(x) => format!("SKP V{x:X}"),
            Op::Sknp(x) => format!("SKNP V{x:X}"),
            Op::LdVxDt(x) => format!("LD V{x:X}, DT"),
            Op::LdVxK(x) => format!("LD V{x:X}, K"),
            Op::LdDtVx(x) => format!("LD DT, V{x:X}"),
            Op::LdStVx(x) => format!("LD ST, V{x:X}"),
            Op::AddIVx(x) => format!("ADD I, V{x:X}"),
            Op::LdFVx(x) => format!("LD F, V{x:X}"),
            Op::LdBVx(x) => format!("LD B, V{x:X}"),
            Op::LdIVx(x) => format!("LD [I], V{x:X}"),
            Op::LdVxI(x) => format!("LD V{x:X}, [I]"),
            Op::Unknown(w) => format!("DW {w:#06X}"),
        }
    }
    /// True for instructions that end a basic block.
    pub fn is_control(&self) -> bool {
        matches!(
            self,
            Op::Ret | Op::Jp(_) | Op::Call(_) | Op::JpV0(_) | Op::SeKk(..) | Op::SneKk(..) | Op::SeVy(..) | Op::SneVy(..) | Op::Skp(_) | Op::Sknp(_) | Op::Unknown(_)
        )
    }
}
