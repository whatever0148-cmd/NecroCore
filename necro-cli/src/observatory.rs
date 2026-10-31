//! `necro-cli observatory`: writes one self-contained HTML page. It embeds every
//! autopsy report, the benchmark JSON, and each ROM as a compiled wasm binary, so
//! the page runs the lifted modules in the browser and hot-patches them live.
use necro_core::scan;
use necro_rom_lifter::{lift::lift_rom, wat as romwat, Chip8, PatchTable};
use serde_json::{json, Value};
use std::path::Path;

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn b64(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for c in data.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        out.push(B64[(n >> 18) as usize & 63] as char);
        out.push(B64[(n >> 12) as usize & 63] as char);
        out.push(if c.len() > 1 { B64[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if c.len() > 2 { B64[n as usize & 63] as char } else { '=' });
    }
    out
}

fn files(dir: &Path, ext: &str) -> Result<Vec<std::path::PathBuf>, String> {
    let mut v: Vec<_> = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map(|x| x == ext).unwrap_or(false)).collect();
    v.sort();
    Ok(v)
}

pub fn build(samples: &Path, bench: Option<&Path>) -> Result<String, String> {
    let mut reports = Vec::new();
    for f in files(&samples.join("cobol"), "cob")? {
        let name = f.file_stem().unwrap().to_string_lossy().to_string();
        let (ir, w) = necro_cobol_lifter::lift_source(&name, &std::fs::read_to_string(&f).map_err(|e| e.to_string())?)?;
        reports.push(json!({"name": name, "kind": "cobol", "report": scan(&ir, w)}));
    }
    let mut roms = Vec::new();
    for f in files(&samples.join("roms"), "ch8")? {
        let name = f.file_stem().unwrap().to_string_lossy().to_string();
        let rom = std::fs::read(&f).map_err(|e| e.to_string())?;
        let patches: PatchTable = serde_json::from_str(&std::fs::read_to_string(f.with_extension("patch.json")).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        let (ir, w) = lift_rom(&name, &rom);
        reports.push(json!({"name": name, "kind": "chip8", "report": scan(&ir, w)}));
        let wasm = wat::parse_str(romwat::emit_wat(&name, &rom, &patches)).map_err(|e| e.to_string())?;
        roms.push(json!({"name": name, "wasm": b64(&wasm), "wasm_bytes": wasm.len(), "image": b64(&Chip8::new(&rom, 1).image), "patches": patches.entries}));
    }
    let bench_val: Value = match bench {
        Some(p) => serde_json::from_str(&std::fs::read_to_string(p).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?,
        None => Value::Null,
    };
    let data = json!({"reports": reports, "roms": roms, "bench": bench_val, "bench_source": bench.map(|p| p.display().to_string())});
    let text = serde_json::to_string(&data).map_err(|e| e.to_string())?.replace("</", "<\\/");
    Ok(include_str!("observatory.html").replace("__DATA__", &text))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn base64_matches_known_vectors() {
        assert_eq!(b64(b""), "");
        assert_eq!(b64(b"f"), "Zg==");
        assert_eq!(b64(b"fo"), "Zm8=");
        assert_eq!(b64(b"foobar"), "Zm9vYmFy");
    }
    #[test]
    fn page_embeds_every_sample() {
        let s = build(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../samples"), None).unwrap();
        for n in ["BANK-INTEREST-1978", "PAYROLL-BATCH-1982", "WIRE-CLEARING-1985", "PONG-STYLE", "INVADERS-STYLE", "BREAKOUT-STYLE"] {
            assert!(s.contains(n), "{n}");
        }
        assert!(!s.contains("__DATA__"));
    }
}
