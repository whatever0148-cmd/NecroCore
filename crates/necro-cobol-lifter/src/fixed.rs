//! Fixed-point arithmetic shared by the interpreter and the WAT emitter.
//! Every value in an expression is an i64 scaled by 10^W. All operations wrap
//! exactly like the WebAssembly i64 instructions they are compiled to.
use necro_core::{StoreMode, VarDecl};

/// Working scale: four decimal places.
pub const W: u32 = 4;

pub fn pow10(n: u32) -> i64 {
    10i64.pow(n.min(18))
}

/// Raw storage value of a variable -> working-scale value.
pub fn load(raw: i64, decl: &VarDecl) -> i64 {
    raw.wrapping_mul(pow10(W - decl.scale.min(W)))
}

/// Working-scale value -> raw storage value with COBOL truncation (or ROUNDED),
/// high-order truncation to the PIC width, and sign dropping for unsigned PICs.
pub fn store(val: i64, decl: &VarDecl, mode: StoreMode) -> i64 {
    let f = pow10(W - decl.scale.min(W));
    let mut x = val;
    if mode == StoreMode::Round && f > 1 {
        let half = f / 2;
        x = x.wrapping_add(if x >= 0 { half } else { -half });
    }
    x = x.wrapping_div(f);
    let digits = decl.int_digits + decl.scale;
    if digits < 19 {
        x = x.wrapping_rem(pow10(digits));
    }
    if !decl.signed {
        x = x.wrapping_abs();
    }
    x
}

pub fn mul(a: i64, b: i64) -> i64 {
    a.wrapping_mul(b).wrapping_div(pow10(W))
}

/// Returns None where WebAssembly `i64.div_s` would trap.
pub fn div(a: i64, b: i64) -> Option<i64> {
    a.wrapping_mul(pow10(W)).checked_div(b)
}

/// Parse a decimal literal such as `1978`, `0.0525` or `-5.5` into a working-scale value.
pub fn literal(text: &str) -> Option<i64> {
    let (neg, t) = match text.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    let (ip, fp) = match t.split_once('.') {
        Some((a, b)) => (a, b),
        None => (t, ""),
    };
    if ip.is_empty() && fp.is_empty() {
        return None;
    }
    if !ip.chars().all(|c| c.is_ascii_digit()) || !fp.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let mut v: i64 = if ip.is_empty() { 0 } else { ip.parse().ok()? };
    v = v.checked_mul(pow10(W))?;
    let mut frac = fp.to_string();
    frac.truncate(W as usize);
    while frac.len() < W as usize {
        frac.push('0');
    }
    let fv: i64 = if frac.is_empty() { 0 } else { frac.parse().ok()? };
    v = v.checked_add(fv)?;
    Some(if neg { -v } else { v })
}

/// Formats a raw storage value as a decimal string with the variable's scale.
pub fn format_raw(raw: i64, decl: &VarDecl) -> String {
    if decl.scale == 0 {
        return raw.to_string();
    }
    let m = pow10(decl.scale);
    let sign = if raw < 0 { "-" } else { "" };
    let a = raw.unsigned_abs();
    format!("{sign}{}.{:0width$}", a / m as u64, a % m as u64, width = decl.scale as usize)
}
