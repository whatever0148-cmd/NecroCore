//! `necro-mcp`: a Model Context Protocol server over stdio (newline-delimited
//! JSON-RPC 2.0) exposing three tools to Kiro:
//! * `necro_autopsy`: static autopsy of a .cob or .ch8,
//! * `necro_parity`: differential parity of interpreter vs lifted wasm module,
//! * `necro_hotpatch`: run a ROM, hot-patch one value, keep running, report state.
use necro_core::scan;
use necro_prover::{prove_chip8, prove_cobol};
use necro_rom_lifter::{lift::lift_rom, wat as romwat, Chip8, PatchTable};
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::path::Path;

pub const PROTOCOL: &str = "2025-06-18";

fn tools() -> Value {
    json!([
        {
            "name": "necro_autopsy",
            "description": "Static autopsy of a legacy program (.cob COBOL subset or .ch8 CHIP-8 ROM): CFG, Ghost/Vampire/Zombie pathologies with byte and line offsets, and EARS requirements text.",
            "inputSchema": {"type": "object", "properties": {"file": {"type": "string", "description": "Path to a .cob or .ch8 file"}}, "required": ["file"]}
        },
        {
            "name": "necro_parity",
            "description": "Differential shadow execution: seeded vectors through the reference interpreter and the lifted wasmtime module. Returns parity percent, divergences and microsecond timings.",
            "inputSchema": {"type": "object", "properties": {"file": {"type": "string"}, "vectors": {"type": "integer", "default": 1000}, "seed": {"type": "integer", "default": 7}}, "required": ["file"]}
        },
        {
            "name": "necro_hotpatch",
            "description": "Run a CHIP-8 ROM for steps_before steps, write one patch point (from its .patch.json) to a new value without resetting, run steps_after more steps, and report the machine state.",
            "inputSchema": {"type": "object", "properties": {"rom": {"type": "string"}, "patch": {"type": "string"}, "value": {"type": "integer"}, "steps_before": {"type": "integer", "default": 200}, "steps_after": {"type": "integer", "default": 200}}, "required": ["rom", "patch", "value"]}
        }
    ])
}

fn read_rom(file: &Path) -> Result<(Vec<u8>, PatchTable), String> {
    let rom = std::fs::read(file).map_err(|e| format!("{}: {e}", file.display()))?;
    let sidecar = file.with_extension("patch.json");
    let patches = match std::fs::read_to_string(&sidecar) {
        Ok(s) => serde_json::from_str(&s).map_err(|e| format!("{}: {e}", sidecar.display()))?,
        Err(_) => PatchTable::default(),
    };
    Ok((rom, patches))
}

fn is_rom(p: &Path) -> bool {
    p.extension().map(|e| e == "ch8").unwrap_or(false)
}

fn stem(p: &Path) -> String {
    p.file_stem().and_then(|s| s.to_str()).unwrap_or("program").to_string()
}

pub fn call_tool(name: &str, args: &Value) -> Result<Value, String> {
    let s = |k: &str| args.get(k).and_then(|v| v.as_str()).map(|v| v.to_string());
    let u = |k: &str, d: u64| args.get(k).and_then(|v| v.as_u64()).unwrap_or(d);
    match name {
        "necro_autopsy" => {
            let file = s("file").ok_or("file is required")?;
            let p = Path::new(&file);
            let report = if is_rom(p) {
                let (rom, _) = read_rom(p)?;
                let (ir, w) = lift_rom(&stem(p), &rom);
                scan(&ir, w)
            } else {
                let src = std::fs::read_to_string(p).map_err(|e| e.to_string())?;
                let (ir, w) = necro_cobol_lifter::lift_source(&stem(p), &src)?;
                scan(&ir, w)
            };
            serde_json::to_value(report).map_err(|e| e.to_string())
        }
        "necro_parity" => {
            let file = s("file").ok_or("file is required")?;
            let p = Path::new(&file);
            let n = u("vectors", 1000) as usize;
            let seed = u("seed", 7);
            let r = if is_rom(p) {
                let (rom, patches) = read_rom(p)?;
                let wat = romwat::emit_wat(&stem(p), &rom, &patches);
                prove_chip8(&stem(p), &rom, &patches, &wat, n, seed, 300)?
            } else {
                let src = std::fs::read_to_string(p).map_err(|e| e.to_string())?;
                let (ir, _) = necro_cobol_lifter::lift_source(&stem(p), &src)?;
                let wat = necro_cobol_lifter::emit_wat(&ir);
                prove_cobol(&ir, &wat, n, seed, 10_000)?
            };
            serde_json::to_value(r).map_err(|e| e.to_string())
        }
        "necro_hotpatch" => {
            let rom_path = s("rom").ok_or("rom is required")?;
            let patch = s("patch").ok_or("patch is required")?;
            let value = u("value", 0) as u8;
            let before = u("steps_before", 200) as usize;
            let after = u("steps_after", 200) as usize;
            let (rom, patches) = read_rom(Path::new(&rom_path))?;
            let mut m = Chip8::new(&rom, 1);
            let (st1, ran1) = m.run(before);
            let regs_before: Vec<u8> = (0..16).map(|x| m.v(x)).collect();
            let pc_before = m.pc();
            let addr = patches.apply(&mut m.image, &patch, value).ok_or_else(|| format!("no patch point named {patch}; known: {:?}", patches.entries.iter().map(|e| e.name.clone()).collect::<Vec<_>>()))?;
            let (st2, ran2) = m.run(after);
            Ok(json!({
                "patch": patch, "written_address": addr, "value": value,
                "before": {"status": format!("{st1:?}"), "steps": ran1, "pc": pc_before, "registers": regs_before},
                "after": {"status": format!("{st2:?}"), "steps": ran2, "pc": m.pc(), "registers": (0..16).map(|x| m.v(x)).collect::<Vec<u8>>(), "lit_pixels": m.lit_pixels()},
                "state_kept": "only the patched byte changed between the two runs; registers, stack and display carried over"
            }))
        }
        other => Err(format!("unknown tool {other}")),
    }
}

/// Handles one JSON-RPC request; `None` for notifications.
pub fn handle(req: &Value) -> Option<Value> {
    let id = req.get("id").cloned();
    let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let params = req.get("params").cloned().unwrap_or(Value::Null);
    let result: Result<Value, (i64, String)> = match method {
        "initialize" => Ok(json!({"protocolVersion": PROTOCOL, "capabilities": {"tools": {}}, "serverInfo": {"name": "necro-mcp", "version": env!("CARGO_PKG_VERSION")}})),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({"tools": tools()})),
        "tools/call" => {
            let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            match call_tool(name, &args) {
                Ok(v) => Ok(json!({"content": [{"type": "text", "text": serde_json::to_string_pretty(&v).unwrap()}], "isError": false})),
                Err(e) => Ok(json!({"content": [{"type": "text", "text": e}], "isError": true})),
            }
        }
        m if m.starts_with("notifications/") => return None,
        other => Err((-32601, format!("method not found: {other}"))),
    };
    id.as_ref()?;
    Some(match result {
        Ok(r) => json!({"jsonrpc": "2.0", "id": id, "result": r}),
        Err((code, msg)) => json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": msg}}),
    })
}

pub fn serve() -> Result<(), String> {
    let stdin = std::io::stdin();
    let mut out = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let req: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                let err = json!({"jsonrpc": "2.0", "id": Value::Null, "error": {"code": -32700, "message": format!("parse error: {e}")}});
                writeln!(out, "{err}").map_err(|e| e.to_string())?;
                continue;
            }
        };
        if let Some(resp) = handle(&req) {
            writeln!(out, "{resp}").map_err(|e| e.to_string())?;
            out.flush().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_three_tools_and_answers_initialize() {
        let r = handle(&json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}})).unwrap();
        assert_eq!(r["result"]["protocolVersion"], PROTOCOL);
        let r = handle(&json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"})).unwrap();
        assert_eq!(r["result"]["tools"].as_array().unwrap().len(), 3);
        assert!(handle(&json!({"jsonrpc": "2.0", "method": "notifications/initialized"})).is_none());
    }

    #[test]
    fn hotpatch_tool_runs_a_sample_rom() {
        let rom = format!("{}/../samples/roms/PONG-STYLE.ch8", env!("CARGO_MANIFEST_DIR"));
        let r = handle(&json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "necro_hotpatch", "arguments": {"rom": rom, "patch": "PADDLE_H", "value": 8}}})).unwrap();
        assert_eq!(r["result"]["isError"], false, "{r}");
        let text = r["result"]["content"][0]["text"].as_str().unwrap();
        let v: Value = serde_json::from_str(text).unwrap();
        assert_eq!(v["after"]["status"], "Ok");
    }

    #[test]
    fn unknown_method_is_an_error() {
        let r = handle(&json!({"jsonrpc": "2.0", "id": 4, "method": "nope"})).unwrap();
        assert_eq!(r["error"]["code"], -32601);
    }
}
