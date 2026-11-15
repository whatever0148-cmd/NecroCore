//! necro-daemon: the lean template that every NecroCore app is built on. A zero-dependency HTTP and
//! Server-Sent-Events daemon on 127.0.0.1. It serves the Observatory page and answers its buttons by
//! calling the same library crates the command line uses.
//!
//! Routes
//!   GET  /                 the Observatory page
//!   GET  /api/bootstrap    samples, ROM modules, latest benchmark, kiro-cli availability
//!   POST /api/autopsy      static scan: report, control-flow graph, source lines
//!   POST /api/exorcise     SSE: scan, Kiro turn, heal, rescan, WebAssembly diff
//!   POST /api/prove        SSE: seeded vectors in batches through the interpreter and wasmtime
//!   POST /api/spec         writes .kiro/specs/autopsy-NAME/{requirements,design,tasks}.md
//!   POST /api/mod          natural-language mod: kiro-cli (or the offline mapper) -> patch values
//!   POST /api/drop         file drop from the notch: saves it, runs the on-legacy-drop hook command
//!
//! The listener binds to 127.0.0.1 only. Every request must carry a loopback Host header and a
//! POST may only come from a loopback Origin (or none, as from a native app).
use necro_core::{heal, scan, NecroAutopsyReport, NecroIR, PathologyKind};
use necro_kiro_bridge::{KiroBridge, SpecProposal};
use necro_prover::{prove_chip8_at, prove_cobol_at};
use necro_rom_lifter::{lift::lift_rom, wat as romwat, PatchField, PatchTable};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// The one UI shell. An [`AppSpec`] decides which views of it an app shows.
pub const SHELL: &str = include_str!("ui.html");

/// What makes one app different from another: a name, the views it shows and its default port.
#[derive(Clone, Copy, Debug)]
pub struct AppSpec {
    pub id: &'static str,
    pub title: &'static str,
    /// Views of the shell to show: "crypt" (COBOL), "arcade" (ROMs), "proof" (evidence).
    pub tabs: &'static [&'static str],
    pub port: u16,
}

impl AppSpec {
    /// Both apps in one page, the way the notch pill and the combined Observatory use it.
    pub const fn observatory() -> AppSpec {
        AppSpec { id: "observatory", title: "NecroCore", tabs: &["crypt", "arcade", "proof"], port: 4300 }
    }
    pub fn json(&self) -> String {
        json!({"id": self.id, "title": self.title, "tabs": self.tabs}).to_string()
    }
}

/// Fills the shell for one app. `data` is the embedded JSON for a saved static copy, or "null".
pub fn render_page(spec: &AppSpec, data: &str) -> String {
    SHELL.replace("__APP__", &spec.json()).replace("__TITLE__", spec.title).replace("__DATA__", data)
}

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

/// Standard base64 decoder (padding optional, whitespace ignored).
pub fn unb64(s: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let (mut acc, mut bits) = (0u32, 0u32);
    for c in s.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' | b' ' | b'\n' | b'\r' => continue,
            _ => return Err("invalid base64".into()),
        };
        acc = acc << 6 | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    Ok(out)
}
const MAX_BODY: usize = 2 * 1024 * 1024;
const PROVE_BATCHES: usize = 10;
const PROVE_SEED: u64 = 7;
const COBOL_FUEL: u64 = 10_000;
const ROM_STEPS: usize = 300;

struct State {
    root: PathBuf,
    port: u16,
    page: String,
    /// Last proposal per program, so "Generate spec" can reuse the Kiro text from "Exorcise".
    proposals: Mutex<HashMap<String, SpecProposal>>,
}

struct Req {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

pub fn serve(root: PathBuf, spec: &AppSpec, port: u16) -> Result<(), String> {
    let root = root.canonicalize().map_err(|e| format!("{}: {e}", root.display()))?;
    if !root.join("samples").is_dir() {
        return Err(format!("{} has no samples/ folder; pass --root to the NecroCore checkout", root.display()));
    }
    let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|e| format!("cannot bind 127.0.0.1:{port}: {e}"))?;
    let state = Arc::new(State { root, port, page: render_page(spec, "null"), proposals: Mutex::new(HashMap::new()) });
    println!("{} on http://127.0.0.1:{port} (root {})", spec.title, state.root.display());
    for conn in listener.incoming() {
        let Ok(stream) = conn else { continue };
        let st = Arc::clone(&state);
        std::thread::spawn(move || {
            let _ = handle(stream, &st);
        });
    }
    Ok(())
}

fn read_req(stream: &mut TcpStream) -> Result<Req, String> {
    stream.set_read_timeout(Some(Duration::from_secs(20))).ok();
    let mut rd = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);
    let mut line = String::new();
    rd.read_line(&mut line).map_err(|e| e.to_string())?;
    let mut parts = line.split_whitespace();
    let method = parts.next().ok_or("empty request")?.to_string();
    let path = parts.next().ok_or("no path")?.to_string();
    let mut headers = HashMap::new();
    loop {
        let mut h = String::new();
        rd.read_line(&mut h).map_err(|e| e.to_string())?;
        let h = h.trim_end();
        if h.is_empty() {
            break;
        }
        if let Some((k, v)) = h.split_once(':') {
            headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
        }
    }
    let len: usize = headers.get("content-length").and_then(|v| v.parse().ok()).unwrap_or(0);
    if len > MAX_BODY {
        return Err("body too large".into());
    }
    let mut body = vec![0u8; len];
    rd.read_exact(&mut body).map_err(|e| e.to_string())?;
    Ok(Req { method, path, headers, body })
}

fn respond(s: &mut TcpStream, code: u16, ctype: &str, body: &[u8]) -> std::io::Result<()> {
    let reason = match code {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        _ => "Error",
    };
    write!(s, "HTTP/1.1 {code} {reason}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n", body.len())?;
    s.write_all(body)?;
    s.flush()
}

fn json_resp(s: &mut TcpStream, code: u16, v: &Value) -> std::io::Result<()> {
    respond(s, code, "application/json", v.to_string().as_bytes())
}

struct Sse<'a>(&'a mut TcpStream);
impl Sse<'_> {
    fn start(s: &mut TcpStream) -> std::io::Result<Sse<'_>> {
        write!(s, "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n")?;
        s.flush()?;
        Ok(Sse(s))
    }
    /// Returns false once the browser has gone away.
    fn event(&mut self, name: &str, data: &Value) -> bool {
        write!(self.0, "event: {name}\ndata: {data}\n\n").and_then(|_| self.0.flush()).is_ok()
    }
}

fn loopback(host: &str, port: u16) -> bool {
    host == format!("127.0.0.1:{port}") || host == format!("localhost:{port}")
}

fn handle(mut stream: TcpStream, st: &State) -> std::io::Result<()> {
    let req = match read_req(&mut stream) {
        Ok(r) => r,
        Err(e) => return json_resp(&mut stream, 400, &json!({"error": e})),
    };
    if !req.headers.get("host").map(|h| loopback(h, st.port)).unwrap_or(false) {
        return json_resp(&mut stream, 403, &json!({"error": "loopback Host header required"}));
    }
    if req.method == "POST" {
        if let Some(o) = req.headers.get("origin") {
            if !(o == &format!("http://127.0.0.1:{}", st.port) || o == &format!("http://localhost:{}", st.port)) {
                return json_resp(&mut stream, 403, &json!({"error": "cross-origin POST refused"}));
            }
        }
    }
    let path = req.path.split('?').next().unwrap_or("/").to_string();
    let body: Value = if req.body.is_empty() { Value::Null } else { serde_json::from_slice(&req.body).unwrap_or(Value::Null) };
    match (req.method.as_str(), path.as_str()) {
        ("GET", "/") | ("GET", "/index.html") => respond(&mut stream, 200, "text/html; charset=utf-8", st.page.as_bytes()),
        ("GET", "/api/bootstrap") => match bootstrap(st) {
            Ok(v) => json_resp(&mut stream, 200, &v),
            Err(e) => json_resp(&mut stream, 400, &json!({"error": e})),
        },
        ("POST", "/api/autopsy") => reply(&mut stream, autopsy(st, &body)),
        ("POST", "/api/spec") => reply(&mut stream, spec(st, &body)),
        ("POST", "/api/mod") => reply(&mut stream, modbox(st, &body)),
        ("POST", "/api/drop") => reply(&mut stream, drop_file(st, &body)),
        ("POST", "/api/exorcise") => {
            let mut sse = Sse::start(&mut stream)?;
            if let Err(e) = exorcise(st, &body, &mut sse) {
                sse.event("fail", &json!({"error": e}));
            }
            Ok(())
        }
        ("POST", "/api/prove") => {
            let mut sse = Sse::start(&mut stream)?;
            if let Err(e) = prove(st, &body, &mut sse) {
                sse.event("fail", &json!({"error": e}));
            }
            Ok(())
        }
        _ => json_resp(&mut stream, 404, &json!({"error": "no such route"})),
    }
}

fn reply(s: &mut TcpStream, r: Result<Value, String>) -> std::io::Result<()> {
    match r {
        Ok(v) => json_resp(s, 200, &v),
        Err(e) => json_resp(s, 400, &json!({"error": e})),
    }
}

// ---------- programs ----------

fn safe_name(s: &str) -> Option<String> {
    let ok = !s.is_empty() && s.len() <= 64 && !s.starts_with('.') && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.') && !s.contains("..");
    ok.then(|| s.to_string())
}

enum Prog {
    Cobol { name: String, src: String },
    Rom { name: String, rom: Vec<u8>, patches: PatchTable },
}

impl Prog {
    fn name(&self) -> &str {
        match self {
            Prog::Cobol { name, .. } | Prog::Rom { name, .. } => name,
        }
    }
}

fn read_patches(path: &Path, name: &str) -> PatchTable {
    std::fs::read_to_string(path.with_extension("patch.json")).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(PatchTable { rom: name.to_string(), entries: vec![] })
}

fn load_prog(st: &State, body: &Value) -> Result<Prog, String> {
    let name = body["name"].as_str().and_then(safe_name);
    if let Some(src) = body["source"].as_str() {
        return Ok(Prog::Cobol { name: name.unwrap_or_else(|| "CUSTOM".into()), src: src.to_string() });
    }
    let name = name.ok_or("missing or unsafe program name")?;
    for d in ["samples/cobol", "samples/dropped"] {
        let p = st.root.join(d).join(format!("{name}.cob"));
        if p.is_file() {
            return Ok(Prog::Cobol { src: std::fs::read_to_string(&p).map_err(|e| e.to_string())?, name });
        }
    }
    for d in ["samples/roms", "samples/dropped"] {
        let p = st.root.join(d).join(format!("{name}.ch8"));
        if p.is_file() {
            let rom = std::fs::read(&p).map_err(|e| e.to_string())?;
            let patches = read_patches(&p, &name);
            return Ok(Prog::Rom { name, rom, patches });
        }
    }
    Err(format!("no sample named {name}"))
}

fn analyse(p: &Prog) -> Result<(NecroIR, NecroAutopsyReport), String> {
    match p {
        Prog::Cobol { name, src } => {
            let (ir, w) = necro_cobol_lifter::lift_source(name, src)?;
            let r = scan(&ir, w);
            Ok((ir, r))
        }
        Prog::Rom { name, rom, .. } => {
            let (ir, w) = lift_rom(name, rom);
            let r = scan(&ir, w);
            Ok((ir, r))
        }
    }
}

fn counts(r: &NecroAutopsyReport) -> Value {
    json!({"ghost": r.count(PathologyKind::Ghost), "vampire": r.count(PathologyKind::Vampire), "zombie": r.count(PathologyKind::Zombie), "total": r.pathologies.len()})
}

fn line_end(src: &str, start: usize, end: usize, line: usize) -> usize {
    match src.get(start..end.min(src.len())) {
        Some(t) => line + t.trim_end().matches('\n').count(),
        None => line,
    }
}

fn autopsy_json(p: &Prog) -> Result<Value, String> {
    let (ir, report) = analyse(p)?;
    let src = if let Prog::Cobol { src, .. } = p { Some(src.as_str()) } else { None };
    let mut kinds: HashMap<String, Vec<&str>> = HashMap::new();
    for x in &report.pathologies {
        let k = match x.kind {
            PathologyKind::Ghost => "Ghost",
            PathologyKind::Vampire => "Vampire",
            PathologyKind::Zombie => "Zombie",
        };
        let e = kinds.entry(x.block.clone()).or_default();
        if !e.contains(&k) {
            e.push(k);
        }
        if x.kind != PathologyKind::Ghost {
            for b in &x.blocks {
                let e = kinds.entry(b.clone()).or_default();
                if !e.contains(&k) {
                    e.push(k);
                }
            }
        }
    }
    let nodes: Vec<Value> = ir
        .blocks
        .iter()
        .map(|b| {
            let le = src.map(|s| line_end(s, b.span.byte_start, b.span.byte_end, b.span.line)).unwrap_or(b.span.line);
            json!({"id": b.label, "line": b.span.line, "line_end": le, "kinds": kinds.get(&b.label).cloned().unwrap_or_default()})
        })
        .collect();
    let pathologies: Vec<Value> = report
        .pathologies
        .iter()
        .map(|x| {
            let le = src.map(|s| line_end(s, x.span.byte_start, x.span.byte_end, x.span.line)).unwrap_or(x.span.line);
            let mut v = serde_json::to_value(x).unwrap_or(Value::Null);
            v["line_end"] = json!(le);
            v["technical"] = json!(x.kind.technical());
            v
        })
        .collect();
    Ok(json!({
        "name": p.name(),
        "kind": if src.is_some() { "cobol" } else { "chip8" },
        "source": src,
        "counts": counts(&report),
        "block_count": report.block_count,
        "edge_count": report.edge_count,
        "nodes": nodes,
        "edges": report.cfg_edges,
        "pathologies": pathologies,
        "warnings": report.lifter_warnings,
        "requirements_markdown": report.requirements_markdown,
    }))
}

/// The autopsy view for one COBOL program, exactly as `/api/autopsy` returns it.
/// The static Observatory embeds these so the hosted copy shows the real analysis.
pub fn cobol_autopsy(name: &str, src: &str) -> Result<Value, String> {
    autopsy_json(&Prog::Cobol { name: name.to_string(), src: src.to_string() })
}

fn autopsy(st: &State, body: &Value) -> Result<Value, String> {
    autopsy_json(&load_prog(st, body)?)
}

// ---------- bootstrap ----------

fn list(dir: &Path, ext: &str) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir).into_iter().flatten().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map(|x| x == ext).unwrap_or(false)).collect();
    v.sort();
    v
}

fn latest_bench(root: &Path) -> Option<(String, Value)> {
    let mut v: Vec<PathBuf> = list(&root.join("benchmarks"), "json").into_iter().filter(|p| p.file_name().map(|n| n.to_string_lossy().contains("phase")).unwrap_or(false)).collect();
    v.sort();
    let p = v.pop()?;
    let val = serde_json::from_str(&std::fs::read_to_string(&p).ok()?).ok()?;
    Some((p.strip_prefix(root).unwrap_or(&p).display().to_string(), val))
}

fn bootstrap(st: &State) -> Result<Value, String> {
    let mut cobol = Vec::new();
    for d in ["samples/cobol", "samples/dropped"] {
        for f in list(&st.root.join(d), "cob") {
            let name = f.file_stem().unwrap().to_string_lossy().to_string();
            cobol.push(json!({"name": name, "dropped": d.ends_with("dropped"), "source": std::fs::read_to_string(&f).unwrap_or_default()}));
        }
    }
    let mut roms = Vec::new();
    for d in ["samples/roms", "samples/dropped"] {
        for f in list(&st.root.join(d), "ch8") {
            let name = f.file_stem().unwrap().to_string_lossy().to_string();
            let rom = std::fs::read(&f).map_err(|e| e.to_string())?;
            let patches = read_patches(&f, &name);
            let wasm = wat::parse_str(romwat::emit_wat(&name, &rom, &patches)).map_err(|e| e.to_string())?;
            roms.push(json!({"name": name, "dropped": d.ends_with("dropped"), "wasm": b64(&wasm), "wasm_bytes": wasm.len(), "image": b64(&necro_rom_lifter::Chip8::new(&rom, 1).image), "patches": patches.entries}));
        }
    }
    let mut reports = Vec::new();
    for c in &cobol {
        if let Ok(prog) = load_prog(st, &json!({"name": c["name"]})) {
            if let Ok((_, r)) = analyse(&prog) {
                reports.push(json!({"name": c["name"], "kind": "cobol", "report": r}));
            }
        }
    }
    for c in &roms {
        if let Ok(prog) = load_prog(st, &json!({"name": c["name"]})) {
            if let Ok((_, r)) = analyse(&prog) {
                reports.push(json!({"name": c["name"], "kind": "chip8", "report": r}));
            }
        }
    }
    let bench = latest_bench(&st.root);
    let trace = std::fs::read_to_string(st.root.join("benchmarks/live-kiro-trace.json")).ok().and_then(|s| serde_json::from_str::<Value>(&s).ok());
    Ok(json!({
        "cobol": cobol,
        "roms": roms,
        "reports": reports,
        "bench": bench.as_ref().map(|b| b.1.clone()),
        "bench_source": bench.map(|b| b.0),
        "kiro_available": KiroBridge::offline().cli_available(),
        "live_trace": trace.map(|t| json!({"live_turns": t["live_turns"], "kiro_cli_version": t["kiro_cli_version"], "turns": t["turns"].as_array().map(|a| a.iter().map(|x| json!({"program": x["program"], "elapsed_ms": x["trace"]["elapsed_ms"], "response": x["trace"]["response"]})).collect::<Vec<_>>())})),
    }))
}

// ---------- exorcise ----------

fn specify(live: bool, report: &NecroAutopsyReport) -> SpecProposal {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let bridge = if live { KiroBridge::live(Duration::from_secs(150)) } else { KiroBridge::offline() };
    rt.block_on(bridge.specify(report))
}

fn diff_lines(a: &str, b: &str) -> Vec<Value> {
    let x: Vec<&str> = a.lines().collect();
    let y: Vec<&str> = b.lines().collect();
    let mut pre = 0;
    while pre < x.len() && pre < y.len() && x[pre] == y[pre] {
        pre += 1;
    }
    let mut suf = 0;
    while suf < x.len() - pre && suf < y.len() - pre && x[x.len() - 1 - suf] == y[y.len() - 1 - suf] {
        suf += 1;
    }
    let (mx, my) = (&x[pre..x.len() - suf], &y[pre..y.len() - suf]);
    let mut ops: Vec<(char, &str)> = Vec::new();
    if mx.len() * my.len() <= 16_000_000 {
        let (n, m) = (mx.len(), my.len());
        let mut t = vec![0u32; (n + 1) * (m + 1)];
        for i in (0..n).rev() {
            for j in (0..m).rev() {
                t[i * (m + 1) + j] = if mx[i] == my[j] { t[(i + 1) * (m + 1) + j + 1] + 1 } else { t[(i + 1) * (m + 1) + j].max(t[i * (m + 1) + j + 1]) };
            }
        }
        let (mut i, mut j) = (0, 0);
        while i < n && j < m {
            if mx[i] == my[j] {
                ops.push((' ', mx[i]));
                i += 1;
                j += 1;
            } else if t[(i + 1) * (m + 1) + j] >= t[i * (m + 1) + j + 1] {
                ops.push(('-', mx[i]));
                i += 1;
            } else {
                ops.push(('+', my[j]));
                j += 1;
            }
        }
        ops.extend(mx[i..].iter().map(|l| ('-', *l)));
        ops.extend(my[j..].iter().map(|l| ('+', *l)));
    } else {
        ops.extend(mx.iter().map(|l| ('-', *l)));
        ops.extend(my.iter().map(|l| ('+', *l)));
    }
    let mut out: Vec<Value> = Vec::new();
    let ctx = 3usize;
    let mut all: Vec<(char, &str)> = Vec::new();
    all.extend(x[pre.saturating_sub(ctx)..pre].iter().map(|l| (' ', *l)));
    all.extend(ops);
    all.extend(x[x.len() - suf..(x.len() - suf + ctx).min(x.len())].iter().map(|l| (' ', *l)));
    let mut run = 0usize;
    let flush = |run: &mut usize, out: &mut Vec<Value>| {
        if *run > 0 {
            out.push(json!({"op": "skip", "n": *run}));
            *run = 0;
        }
    };
    let changed: Vec<bool> = all.iter().map(|(o, _)| *o != ' ').collect();
    for (k, (o, l)) in all.iter().enumerate() {
        let near = (k.saturating_sub(ctx)..=(k + ctx).min(all.len() - 1)).any(|q| changed[q]);
        if *o != ' ' || near {
            flush(&mut run, &mut out);
            out.push(json!({"op": o.to_string(), "text": l}));
        } else {
            run += 1;
        }
    }
    flush(&mut run, &mut out);
    out
}

fn exorcise(st: &State, body: &Value, sse: &mut Sse) -> Result<(), String> {
    let prog = load_prog(st, body)?;
    let Prog::Cobol { name, src } = &prog else { return Err("Exorcise heals COBOL programs. ROMs are changed with the hot-patch table.".into()) };
    let live = body["live"].as_bool().unwrap_or(true) && KiroBridge::offline().cli_available();
    sse.event("stage", &json!({"stage": "scan", "text": format!("Static scan of {name}")}));
    let (ir, report) = analyse(&prog)?;
    let before = counts(&report);
    sse.event("stage", &json!({"stage": "scanned", "counts": before}));
    sse.event("stage", &json!({"stage": "kiro", "text": if live { "Asking kiro-cli for requirements and patch proposals" } else { "Building the offline proposal" }}));
    let t = Instant::now();
    let prop = specify(live, &report);
    sse.event("stage", &json!({"stage": "kiro_done", "source": prop.source, "ms": t.elapsed().as_millis()}));
    sse.event("stage", &json!({"stage": "heal", "text": "Inserting guards and detaching dead code"}));
    let healed = heal(&ir, &report);
    let again = scan(&healed, vec![]);
    let wat_before = necro_cobol_lifter::emit_wat(&ir);
    let wat_after = necro_cobol_lifter::emit_wat(&healed);
    // Check that the healed module still agrees with its own interpreter on a few hundred vectors.
    let check = prove_cobol_at(&healed, &wat_after, 0, 500, PROVE_SEED, COBOL_FUEL).map(|r| json!({"vectors": r.vectors, "matched": r.matched, "non_halting": r.non_halting, "original_non_halting": prove_cobol_at(&ir, &wat_before, 0, 500, PROVE_SEED, COBOL_FUEL).map(|o| o.non_halting).unwrap_or(0)}));
    st.proposals.lock().unwrap().insert(name.clone(), prop.clone());
    let remaining: Vec<Value> = again.pathologies.iter().map(|p| json!({"label": p.label, "block": p.block, "line": p.span.line})).collect();
    sse.event(
        "result",
        &json!({
            "name": name,
            "before": before,
            "after": counts(&again),
            "remaining": remaining,
            "kiro_source": prop.source,
            "kiro_ms": prop.elapsed_ms,
            "requirements_md": prop.requirements_md,
            "patches": prop.patches,
            "wat_diff": diff_lines(&wat_before, &wat_after),
            "wat_lines": [wat_before.lines().count(), wat_after.lines().count()],
            "healed_check": check.unwrap_or(Value::Null),
            "source_lines": src.lines().count(),
        }),
    );
    Ok(())
}

// ---------- prove ----------

fn prove(st: &State, body: &Value, sse: &mut Sse) -> Result<(), String> {
    let prog = load_prog(st, body)?;
    let total = body["vectors"].as_u64().unwrap_or(10_000).clamp(PROVE_BATCHES as u64, 50_000) as usize;
    let per = total / PROVE_BATCHES;
    let total = per * PROVE_BATCHES;
    sse.event("start", &json!({"name": prog.name(), "vectors": total, "batches": PROVE_BATCHES, "seed": PROVE_SEED}));
    let (mut matched, mut vectors, mut interp, mut wasm, mut nonhalt, mut compile) = (0usize, 0usize, 0f64, 0f64, 0usize, 0f64);
    let mut divs: Vec<Value> = Vec::new();
    let mut hist: HashMap<String, usize> = HashMap::new();
    let prepared = match &prog {
        Prog::Cobol { name, src } => {
            let (ir, _) = necro_cobol_lifter::lift_source(name, src)?;
            let wat = necro_cobol_lifter::emit_wat(&ir);
            (Some((ir, wat)), None)
        }
        Prog::Rom { name, rom, patches } => (None, Some(romwat::emit_wat(name, rom, patches))),
    };
    for b in 0..PROVE_BATCHES {
        let r = match (&prog, &prepared) {
            (Prog::Cobol { .. }, (Some((ir, wat)), _)) => prove_cobol_at(ir, wat, b * per, per, PROVE_SEED, COBOL_FUEL)?,
            (Prog::Rom { name, rom, patches }, (_, Some(wat))) => prove_chip8_at(name, rom, patches, wat, b * per, per, PROVE_SEED, ROM_STEPS)?,
            _ => unreachable!(),
        };
        matched += r.matched;
        vectors += r.vectors;
        interp += r.interp_total_us;
        wasm += r.wasm_total_us;
        nonhalt += r.non_halting;
        compile += r.compile_us;
        for (k, n) in r.status_histogram {
            *hist.entry(k).or_default() += n;
        }
        for d in r.divergences {
            if divs.len() < 5 {
                divs.push(serde_json::to_value(d).unwrap_or(Value::Null));
            }
        }
        let ok = sse.event(
            "batch",
            &json!({
                "batch": b + 1, "batches": PROVE_BATCHES, "vectors": vectors, "matched": matched,
                "parity_pct": 100.0 * matched as f64 / vectors as f64,
                "interp_us_per_vec": interp / vectors as f64, "wasm_us_per_vec": wasm / vectors as f64,
                "speedup": if wasm > 0.0 { interp / wasm } else { 0.0 }, "non_halting": nonhalt,
            }),
        );
        if !ok {
            return Ok(());
        }
    }
    let mut h: Vec<(String, usize)> = hist.into_iter().collect();
    h.sort();
    sse.event(
        "done",
        &json!({
            "name": prog.name(), "vectors": vectors, "matched": matched,
            "parity_pct": 100.0 * matched as f64 / vectors as f64,
            "interp_us_per_vec": interp / vectors as f64, "wasm_us_per_vec": wasm / vectors as f64,
            "speedup": if wasm > 0.0 { interp / wasm } else { 0.0 },
            "non_halting": nonhalt, "compile_us_per_batch": compile / PROVE_BATCHES as f64,
            "status_histogram": h, "divergences": divs, "seed": PROVE_SEED,
            "target": if matches!(prog, Prog::Cobol { .. }) { "cobol" } else { "chip8" },
            "rom_steps": ROM_STEPS, "cobol_fuel": COBOL_FUEL,
        }),
    );
    Ok(())
}

// ---------- spec ----------

fn spec(st: &State, body: &Value) -> Result<Value, String> {
    let prog = load_prog(st, body)?;
    let (_, report) = analyse(&prog)?;
    let name = prog.name().to_string();
    let cached = st.proposals.lock().unwrap().get(&name).cloned();
    let prop = cached.unwrap_or_else(|| KiroBridge::offline_proposal(&report));
    let dir = st.root.join(".kiro/specs").join(format!("autopsy-{name}"));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let requirements = if prop.source == "kiro-cli" {
        format!("# Requirements: {name}\n\n_Written by kiro-cli ({} ms) from the static autopsy, then saved by the NecroCore daemon._\n\n{}\n\n## Static findings (EARS)\n\n{}\n", prop.elapsed_ms, prop.requirements_md.trim(), report.requirements_markdown.trim())
    } else {
        format!("# Requirements: {name}\n\n_Generated offline from the static autopsy. Run Exorcise with kiro-cli available to add Kiro's text._\n\n{}\n", report.requirements_markdown.trim())
    };
    let mut design = format!("# Design: {name}\n\nOrigin: {}. Blocks: {}. Edges: {}.\n\n## Pathologies\n\n", report.origin, report.block_count, report.edge_count);
    if report.pathologies.is_empty() {
        design.push_str("None found.\n");
    }
    for (i, p) in prop.patches.iter().enumerate() {
        let x = &report.pathologies[p.pathology_index.min(report.pathologies.len() - 1)];
        design.push_str(&format!("{}. **{}** ({}) at line {}, block `{}`\n   - {}\n   - Proposed change: {}\n", i + 1, x.label, x.kind.technical(), x.span.line, x.block, p.rationale, p.proposed_change));
    }
    let mut tasks = format!("# Tasks: {name}\n\n");
    for p in &prop.patches {
        tasks.push_str(&format!("- [ ] {}\n", p.title));
    }
    tasks.push_str("- [ ] Rerun `necro-cli benchmark` and confirm parity stays at 100%\n");
    let files = [("requirements.md", &requirements), ("design.md", &design), ("tasks.md", &tasks)];
    let mut written = Vec::new();
    for (f, text) in files {
        std::fs::write(dir.join(f), text).map_err(|e| e.to_string())?;
        written.push(json!({"path": format!(".kiro/specs/autopsy-{name}/{f}"), "bytes": text.len(), "text": text}));
    }
    Ok(json!({"name": name, "source": prop.source, "files": written}))
}

// ---------- natural-language mod box ----------

fn field_range(f: PatchField) -> (u8, u8) {
    match f {
        PatchField::N => (0, 15),
        _ => (0, 255),
    }
}

fn offline_mod(prompt: &str, patches: &PatchTable) -> (HashMap<String, u8>, String) {
    let p = prompt.to_lowercase();
    let has = |ws: &[&str]| ws.iter().any(|w| p.contains(w));
    let mut v: HashMap<String, u8> = HashMap::new();
    let mut why: Vec<String> = Vec::new();
    let set = |n: &str, x: u8, w: &str, v: &mut HashMap<String, u8>, why: &mut Vec<String>| {
        if patches.get(n).is_some() {
            v.insert(n.into(), x);
            why.push(w.into());
        }
    };
    if has(&["double", "faster", "speed up", "quick", "twice"]) && has(&["speed", "fast", "ball"]) {
        set("FRAME_DELAY", 1, "double speed: wait 1 tick per frame instead of 2", &mut v, &mut why);
    }
    if has(&["slow"]) {
        set("FRAME_DELAY", 4, "slow motion: wait 4 ticks per frame", &mut v, &mut why);
    }
    if has(&["widen", "wider", "bigger paddle", "taller", "longer paddle", "large paddle"]) {
        let cur = patches.get("PADDLE_H").map(|e| e.default).unwrap_or(4);
        set("PADDLE_H", (cur * 2).min(15), "paddle height doubled", &mut v, &mut why);
    }
    if has(&["narrow", "shrink", "smaller paddle", "tiny paddle"]) {
        set("PADDLE_H", 2, "paddle height cut to 2 rows", &mut v, &mut why);
    }
    if has(&["multi", "second ball", "two ball", "extra ball", "another ball", "more ball"]) {
        set("BALL2_ON", 1, "second ball switched on", &mut v, &mut why);
    }
    if has(&["zero-gravity", "zero gravity", "no gravity", "float", "straight line", "no bounce"]) {
        set("BALL_DY", 0, "vertical velocity 0 so the ball floats straight across", &mut v, &mut why);
    }
    if has(&["reverse", "other way", "flip direction"]) {
        set("BALL_DX", 0xFF, "horizontal velocity reversed", &mut v, &mut why);
    }
    let text = if why.is_empty() { "No patch point matched that request.".to_string() } else { why.join("; ") };
    (v, text)
}

fn modbox(st: &State, body: &Value) -> Result<Value, String> {
    let prompt = body["prompt"].as_str().ok_or("missing prompt")?.chars().take(400).collect::<String>();
    let prog = load_prog(st, &json!({"name": body["rom"]}))?;
    let Prog::Rom { name, patches, .. } = prog else { return Err("the mod box works on ROMs".into()) };
    let live = body["live"].as_bool().unwrap_or(true) && KiroBridge::offline().cli_available();
    let mut table = String::new();
    for e in &patches.entries {
        let (lo, hi) = field_range(e.field);
        table.push_str(&format!("- {} (range {lo}..{hi}, default {}): {}\n", e.name, e.default, e.description));
    }
    let ask = format!(
        "You edit a running CHIP-8 game through named patch points. Patch points of {name}:\n{table}\nVelocity points only work with 1 (forward), 255 (backward) and 0 (none). \
         The player asked: \"{prompt}\"\nReply with ONE line of JSON and nothing else, in the form {{\"values\":{{\"NAME\":number}},\"explanation\":\"one short sentence\"}}. \
         Hints: pace is set by FRAME_DELAY (lower is faster). A ball with no vertical pull has BALL_DY 0. \
         Use only the names listed. Leave out any patch point that the request does not change."
    );
    let t = Instant::now();
    let mut source = "offline".to_string();
    let mut chosen: Option<(HashMap<String, u8>, String)> = None;
    let mut transcript = Value::Null;
    if live {
        let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
        match rt.block_on(KiroBridge::live(Duration::from_secs(120)).ask(&ask)) {
            Ok(tr) => {
                if let Some((vals, why, repaired)) = parse_mod_reply(&tr.response, &patches) {
                    source = if repaired { "kiro-cli (closing brace added)".into() } else { "kiro-cli".into() };
                    chosen = Some((vals, why));
                } else {
                    source = "offline (kiro-cli reply was not usable JSON)".into();
                }
                transcript = json!({"prompt": tr.prompt, "response": tr.response, "elapsed_ms": tr.elapsed_ms, "started_unix_ms": tr.started_unix_ms, "command": tr.command, "exit_code": tr.exit_code});
            }
            Err(e) => source = format!("offline ({e})"),
        }
    }
    let (values, explanation) = chosen.unwrap_or_else(|| offline_mod(&prompt, &patches));
    let mut list: Vec<Value> = values.iter().map(|(k, v)| json!({"name": k, "value": v})).collect();
    list.sort_by_key(|x| x["name"].as_str().unwrap_or("").to_string());
    if source.starts_with("kiro-cli") {
        // Keep every real mod-box turn as evidence, next to the benchmark transcripts.
        let path = st.root.join("benchmarks/live-mod-trace.json");
        let mut all: Vec<Value> = std::fs::read_to_string(&path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
        all.push(json!({"rom": name, "user_prompt": prompt, "values_applied": list, "explanation": explanation, "elapsed_ms": t.elapsed().as_millis(), "kiro_turn": transcript}));
        let _ = std::fs::write(&path, serde_json::to_string_pretty(&all).unwrap_or_default());
    }
    Ok(json!({"rom": name, "prompt": prompt, "source": source, "values": list, "explanation": explanation, "elapsed_ms": t.elapsed().as_millis(), "transcript": transcript}))
}

/// Reads Kiro's one-line JSON reply. One real reply arrived without its last closing
/// brace, so a reply that only parses after adding braces is accepted and labelled as repaired.
fn parse_mod_reply(text: &str, patches: &PatchTable) -> Option<(HashMap<String, u8>, String, bool)> {
    let a = text.find('{')?;
    let whole = text.get(a..)?.trim_end();
    let mut attempts: Vec<(String, bool)> = Vec::new();
    if let Some(b) = text.rfind('}') {
        attempts.push((text.get(a..=b)?.to_string(), false));
    }
    attempts.push((format!("{whole}}}"), true));
    attempts.push((format!("{whole}}}}}"), true));
    for (cand, repaired) in attempts {
        let Ok(v) = serde_json::from_str::<Value>(&cand) else { continue };
        let Some(obj) = v["values"].as_object() else { continue };
        let mut out = HashMap::new();
        let mut ok = true;
        for (k, x) in obj {
            let (Some(e), Some(n)) = (patches.get(k), x.as_i64().or_else(|| x.as_f64().map(|f| f as i64))) else {
                ok = false;
                break;
            };
            let (lo, hi) = field_range(e.field);
            out.insert(k.clone(), n.clamp(lo as i64, hi as i64) as u8);
        }
        if ok && !out.is_empty() {
            return Some((out, v["explanation"].as_str().unwrap_or("").chars().take(300).collect(), repaired));
        }
    }
    None
}

// ---------- file drop from the notch ----------

fn run_hook(root: &Path, rel: &str) -> Value {
    let hook: Value = std::fs::read_to_string(root.join(".kiro/hooks/on-legacy-drop.json")).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(Value::Null);
    let Some(cmd) = hook["hooks"][0]["action"]["command"].as_str() else { return json!({"ran": false, "reason": "hook file missing"}) };
    let t = Instant::now();
    let child = std::process::Command::new("sh").arg("-c").arg(cmd).current_dir(root).stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::piped()).spawn();
    let Ok(mut child) = child else { return json!({"ran": false, "reason": "could not start sh"}) };
    if let Some(mut si) = child.stdin.take() {
        let _ = si.write_all(json!({"file_path": rel}).to_string().as_bytes());
    }
    let deadline = Instant::now() + Duration::from_secs(150);
    let code = loop {
        match child.try_wait() {
            Ok(Some(s)) => break s.code(),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(50)),
            _ => {
                let _ = child.kill();
                break None;
            }
        }
    };
    json!({"ran": true, "name": hook["hooks"][0]["name"], "trigger": hook["hooks"][0]["trigger"], "file_path": rel, "exit_code": code, "ms": t.elapsed().as_millis()})
}

fn drop_file(st: &State, body: &Value) -> Result<Value, String> {
    let (rel, name) = if let Some(s) = body["sample"].as_str() {
        let n = safe_name(s).ok_or("unsafe sample name")?;
        let rel = ["samples/cobol/", "samples/roms/"]
            .iter()
            .flat_map(|d| ["cob", "ch8"].map(|e| format!("{d}{n}.{e}")))
            .find(|r| st.root.join(r).is_file())
            .ok_or(format!("no sample named {n}"))?;
        (rel, n)
    } else {
        let fname = body["filename"].as_str().and_then(safe_name).ok_or("missing or unsafe filename")?;
        let ext = fname.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
        if ext != "cob" && ext != "ch8" {
            return Err("only .cob and .ch8 files can be resurrected".into());
        }
        let data = unb64(body["b64"].as_str().ok_or("missing b64")?)?;
        if data.len() > 1_000_000 {
            return Err("file larger than 1 MB".into());
        }
        let dir = st.root.join("samples/dropped");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let stem = fname.rsplit_once('.').map(|x| x.0).unwrap_or(&fname).to_string();
        std::fs::write(dir.join(format!("{stem}.{ext}")), data).map_err(|e| e.to_string())?;
        (format!("samples/dropped/{stem}.{ext}"), stem)
    };
    let hook = run_hook(&st.root, &rel);
    let prog = load_prog(st, &json!({"name": name}))?;
    let (ir, report) = analyse(&prog)?;
    let healed = heal(&ir, &report);
    let again = scan(&healed, vec![]);
    let kind = if matches!(prog, Prog::Cobol { .. }) { "cobol" } else { "chip8" };
    Ok(json!({
        "name": name, "kind": kind, "path": rel, "before": counts(&report), "after": counts(&again), "hook": hook,
        "url": format!("http://127.0.0.1:{}/#{}/{}{}", st.port, if kind == "cobol" { "cryptledger" } else { "phantomarcade" }, name, if kind == "cobol" { "/healed" } else { "" }),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use necro_rom_lifter::PatchEntry;

    fn table() -> PatchTable {
        let e = |n: &str| PatchEntry { name: n.into(), addr: 0x200, field: PatchField::Kk, default: 1, description: String::new() };
        PatchTable { rom: "T".into(), entries: vec![e("FRAME_DELAY"), e("PADDLE_H"), e("BALL_DY")] }
    }
    #[test]
    fn reads_a_clean_reply() {
        let (v, why, repaired) = parse_mod_reply("{\"values\":{\"FRAME_DELAY\":1,\"BALL_DY\":0},\"explanation\":\"ok\"}\n", &table()).unwrap();
        assert_eq!((v["FRAME_DELAY"], v["BALL_DY"], why.as_str(), repaired), (1, 0, "ok", false));
    }
    #[test]
    fn repairs_a_reply_that_lost_its_last_brace() {
        let (v, _, repaired) = parse_mod_reply("{\"values\":{\"FRAME_DELAY\":1,\"PADDLE_H\":8,\"BALL_DY\":0},\"explanation\":\"no vertical pull.\"", &table()).unwrap();
        assert!(repaired);
        assert_eq!((v["FRAME_DELAY"], v["PADDLE_H"], v["BALL_DY"]), (1, 8, 0));
    }
    #[test]
    fn refuses_unknown_names_and_clamps_range() {
        assert!(parse_mod_reply("{\"values\":{\"NOPE\":1},\"explanation\":\"x\"}", &table()).is_none());
        let (v, _, _) = parse_mod_reply("{\"values\":{\"PADDLE_H\":999},\"explanation\":\"x\"}", &table()).unwrap();
        assert_eq!(v["PADDLE_H"], 255);
    }
}

#[cfg(test)]
mod shell_tests {
    use super::*;
    #[test]
    fn every_app_gets_its_own_page_from_the_one_shell() {
        let crypt = AppSpec { id: "cryptledger", title: "CryptLedger", tabs: &["crypt", "proof"], port: 4301 };
        let page = render_page(&crypt, "null");
        assert!(page.contains("<title>CryptLedger</title>") && page.contains("\"tabs\":[\"crypt\",\"proof\"]"));
        assert!(!page.contains("__APP__") && !page.contains("__TITLE__") && !page.contains("__DATA__"));
        assert!(render_page(&AppSpec::observatory(), "null").contains("\"arcade\""));
    }
    #[test]
    fn base64_round_trips() {
        for t in [&b""[..], b"f", b"fo", b"foo", b"foobar", &[0u8, 255, 128, 7, 9][..]] {
            assert_eq!(unb64(&b64(t)).unwrap(), t);
        }
        assert_eq!(b64(b"foobar"), "Zm9vYmFy");
    }
}
