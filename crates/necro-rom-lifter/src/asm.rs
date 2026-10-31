//! A small two-pass CHIP-8 assembler.
//!
//! Syntax: one instruction per line, `;` comments, `label:` definitions,
//! `DB b, b, ...` data bytes, registers `V0`..`VF`, numbers decimal or `0x..`.
//! A line `.patch NAME description` marks the next instruction's immediate
//! (or the next DB line's first byte) as a hot-patch point.
use crate::patch::{PatchEntry, PatchField, PatchTable};
use std::collections::HashMap;

pub struct Assembled {
    pub bytes: Vec<u8>,
    pub patches: PatchTable,
    pub labels: HashMap<String, u16>,
}

fn reg(t: &str) -> Result<u8, String> {
    let t = t.trim();
    if t.len() == 2 && t.starts_with('V') {
        return u8::from_str_radix(&t[1..], 16).map_err(|_| format!("bad register {t}"));
    }
    Err(format!("bad register {t}"))
}

fn num(t: &str, labels: &HashMap<String, u16>) -> Result<u16, String> {
    let t = t.trim();
    if let Some(h) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        return u16::from_str_radix(h, 16).map_err(|_| format!("bad number {t}"));
    }
    if let Ok(v) = t.parse::<u16>() {
        return Ok(v);
    }
    labels.get(t).copied().ok_or_else(|| format!("unknown label {t}"))
}

fn split_operands(rest: &str) -> Vec<String> {
    rest.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

fn encode(mn: &str, ops: &[String], labels: &HashMap<String, u16>) -> Result<u16, String> {
    let n = |i: usize| num(&ops[i], labels);
    let r = |i: usize| reg(&ops[i]);
    let need = |k: usize| if ops.len() == k { Ok(()) } else { Err(format!("{mn} takes {k} operands")) };
    Ok(match mn {
        "CLS" => 0x00E0,
        "SCD" => {
            need(1)?;
            0x00C0 | (n(0)? & 0xF)
        }
        "SCR" => 0x00FB,
        "SCL" => 0x00FC,
        "LOW" => 0x00FE,
        "HIGH" => 0x00FF,
        "RET" => 0x00EE,
        "JP" => {
            if ops.len() == 2 && ops[0] == "V0" {
                0xB000 | (n(1)? & 0xFFF)
            } else {
                need(1)?;
                0x1000 | (n(0)? & 0xFFF)
            }
        }
        "CALL" => {
            need(1)?;
            0x2000 | (n(0)? & 0xFFF)
        }
        "SE" => {
            need(2)?;
            if ops[1].starts_with('V') {
                0x5000 | ((r(0)? as u16) << 8) | ((r(1)? as u16) << 4)
            } else {
                0x3000 | ((r(0)? as u16) << 8) | (n(1)? & 0xFF)
            }
        }
        "SNE" => {
            need(2)?;
            if ops[1].starts_with('V') {
                0x9000 | ((r(0)? as u16) << 8) | ((r(1)? as u16) << 4)
            } else {
                0x4000 | ((r(0)? as u16) << 8) | (n(1)? & 0xFF)
            }
        }
        "LD" => {
            need(2)?;
            match (ops[0].as_str(), ops[1].as_str()) {
                ("I", b) => 0xA000 | (num(b, labels)? & 0xFFF),
                ("DT", b) => 0xF015 | ((reg(b)? as u16) << 8),
                ("ST", b) => 0xF018 | ((reg(b)? as u16) << 8),
                ("F", b) => 0xF029 | ((reg(b)? as u16) << 8),
                ("B", b) => 0xF033 | ((reg(b)? as u16) << 8),
                ("[I]", b) => 0xF055 | ((reg(b)? as u16) << 8),
                ("R", b) => 0xF075 | ((reg(b)? as u16) << 8),
                (a, "R") => 0xF085 | ((reg(a)? as u16) << 8),
                (a, "DT") => 0xF007 | ((reg(a)? as u16) << 8),
                (a, "K") => 0xF00A | ((reg(a)? as u16) << 8),
                (a, "[I]") => 0xF065 | ((reg(a)? as u16) << 8),
                (a, b) if b.starts_with('V') => 0x8000 | ((reg(a)? as u16) << 8) | ((reg(b)? as u16) << 4),
                (a, b) => 0x6000 | ((reg(a)? as u16) << 8) | (num(b, labels)? & 0xFF),
            }
        }
        "ADD" => {
            need(2)?;
            match (ops[0].as_str(), ops[1].as_str()) {
                ("I", b) => 0xF01E | ((reg(b)? as u16) << 8),
                (a, b) if b.starts_with('V') => 0x8004 | ((reg(a)? as u16) << 8) | ((reg(b)? as u16) << 4),
                (a, b) => 0x7000 | ((reg(a)? as u16) << 8) | (num(b, labels)? & 0xFF),
            }
        }
        "OR" => 0x8001 | ((r(0)? as u16) << 8) | ((r(1)? as u16) << 4),
        "AND" => 0x8002 | ((r(0)? as u16) << 8) | ((r(1)? as u16) << 4),
        "XOR" => 0x8003 | ((r(0)? as u16) << 8) | ((r(1)? as u16) << 4),
        "SUB" => 0x8005 | ((r(0)? as u16) << 8) | ((r(1)? as u16) << 4),
        "SHR" => 0x8006 | ((r(0)? as u16) << 8),
        "SUBN" => 0x8007 | ((r(0)? as u16) << 8) | ((r(1)? as u16) << 4),
        "SHL" => 0x800E | ((r(0)? as u16) << 8),
        "RND" => 0xC000 | ((r(0)? as u16) << 8) | (n(1)? & 0xFF),
        "DRW" => {
            need(3)?;
            0xD000 | ((r(0)? as u16) << 8) | ((r(1)? as u16) << 4) | (n(2)? & 0xF)
        }
        "SKP" => 0xE09E | ((r(0)? as u16) << 8),
        "SKNP" => 0xE0A1 | ((r(0)? as u16) << 8),
        other => return Err(format!("unknown mnemonic {other}")),
    })
}

pub fn assemble(name: &str, src: &str) -> Result<Assembled, String> {
    // Pass 1: labels and sizes.
    let mut labels: HashMap<String, u16> = HashMap::new();
    let mut addr: u16 = 0x200;
    let lines: Vec<(usize, String)> = src
        .lines()
        .enumerate()
        .map(|(i, l)| (i + 1, l.split(';').next().unwrap_or("").trim().to_string()))
        .filter(|(_, l)| !l.is_empty())
        .collect();
    for (ln, l) in &lines {
        if let Some(lab) = l.strip_suffix(':') {
            labels.insert(lab.trim().to_string(), addr);
            continue;
        }
        if l.starts_with(".patch") {
            continue;
        }
        let (mn, rest) = l.split_once(char::is_whitespace).unwrap_or((l.as_str(), ""));
        if mn.eq_ignore_ascii_case("DB") {
            addr += split_operands(rest).len() as u16;
        } else if addr & 1 == 1 {
            return Err(format!("line {ln}: instruction at odd address {addr:#X}; pad data with an extra byte"));
        } else {
            addr += 2;
        }
    }
    // Pass 2: encode.
    let mut bytes = Vec::new();
    let mut patches = PatchTable { rom: name.to_string(), entries: vec![] };
    let mut pending_patch: Option<(String, String)> = None;
    addr = 0x200;
    for (ln, l) in &lines {
        if l.ends_with(':') {
            continue;
        }
        if let Some(rest) = l.strip_prefix(".patch") {
            let rest = rest.trim();
            let (nm, desc) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
            pending_patch = Some((nm.to_string(), desc.trim().to_string()));
            continue;
        }
        let (mn, rest) = l.split_once(char::is_whitespace).unwrap_or((l.as_str(), ""));
        let mn_u = mn.to_ascii_uppercase();
        if mn_u == "DB" {
            let ops = split_operands(rest);
            if let Some((nm, desc)) = pending_patch.take() {
                patches.entries.push(PatchEntry { name: nm, addr, field: PatchField::Byte, default: num(&ops[0], &labels)? as u8, description: desc });
            }
            for o in &ops {
                bytes.push(num(o, &labels)? as u8);
                addr += 1;
            }
            continue;
        }
        let ops = split_operands(rest);
        let w = encode(&mn_u, &ops, &labels).map_err(|e| format!("line {ln}: {e}"))?;
        if let Some((nm, desc)) = pending_patch.take() {
            let field = match w >> 12 {
                0xD => PatchField::N,
                0x3 | 0x4 | 0x6 | 0x7 | 0xC => PatchField::Kk,
                _ => return Err(format!("line {ln}: .patch applies only to immediates (SE/SNE/LD/ADD/RND kk, DRW n, DB)")),
            };
            let default = match field {
                PatchField::N => (w & 0xF) as u8,
                _ => (w & 0xFF) as u8,
            };
            patches.entries.push(PatchEntry { name: nm, addr, field, default, description: desc });
        }
        bytes.push((w >> 8) as u8);
        bytes.push((w & 0xFF) as u8);
        addr += 2;
    }
    Ok(Assembled { bytes, patches, labels })
}
