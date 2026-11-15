mod mcp;
mod observatory;

use clap::{Parser, Subcommand};
use necro_core::{heal, scan, PathologyKind};
use necro_kiro_bridge::KiroBridge;
use necro_prover::{prove_chip8, prove_cobol, ParityResult};
use necro_rom_lifter::{asm, lift::lift_rom, wat as romwat, PatchTable};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(name = "necro-cli", about = "NecroCore: legacy compiler translation and retro bytecode emulation on WebAssembly")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Static autopsy of a COBOL program (.cob) or CHIP-8 ROM (.ch8); prints the report as JSON.
    Autopsy { file: PathBuf, #[arg(long)] requirements: bool },
    /// Lift a .cob or .ch8 to WebAssembly text.
    Lift { file: PathBuf, #[arg(short, long)] out: Option<PathBuf> },
    /// Assemble a CHIP-8 .asm into .ch8 plus a .patch.json sidecar.
    Assemble { file: PathBuf },
    /// Write the Observatory: one self-contained HTML page that shows the autopsies and runs the lifted ROMs live.
    Observatory {
        #[arg(short, long, default_value = "observatory/index.html")]
        out: PathBuf,
        #[arg(long, default_value = "samples")]
        samples: PathBuf,
        #[arg(long)]
        bench: Option<PathBuf>,
        /// Run the local daemon (HTTP and SSE on 127.0.0.1) instead of writing a file.
        #[arg(long)]
        serve: bool,
        #[arg(long, default_value_t = 4300)]
        port: u16,
        /// NecroCore checkout the daemon reads samples from and writes .kiro/specs into.
        #[arg(long, default_value = ".")]
        root: PathBuf,
    },
    /// Serve the necro-mcp Model Context Protocol server over stdio (used by .kiro/settings/mcp.json).
    Mcp,
    /// End-to-end benchmark over samples/cobol and samples/roms.
    Benchmark {
        #[arg(long, default_value_t = 10_000)]
        vectors: usize,
        #[arg(long, default_value_t = 7)]
        seed: u64,
        #[arg(long, default_value_t = 300)]
        rom_steps: usize,
        #[arg(long, default_value_t = 10_000)]
        fuel: u64,
        /// Ask the local kiro-cli for spec text (needs a login); otherwise the offline bridge is used.
        #[arg(long)]
        live_kiro: bool,
        #[arg(long)]
        json: Option<PathBuf>,
        #[arg(long, default_value = "samples")]
        samples: PathBuf,
    },
}

#[derive(Serialize)]
struct BenchRow {
    target: String,
    kind: String,
    lift_us: f64,
    blocks: usize,
    edges: usize,
    ghosts: usize,
    vampires: usize,
    zombies: usize,
    warnings: usize,
    healed_remaining: usize,
    parity: ParityResult,
    bridge_source: String,
    bridge_patches: usize,
    bridge_ms: u128,
}

fn stem(p: &Path) -> String {
    p.file_stem().and_then(|s| s.to_str()).unwrap_or("program").to_string()
}

fn read_rom(file: &Path) -> Result<(Vec<u8>, PatchTable), String> {
    let rom = std::fs::read(file).map_err(|e| format!("{}: {e}", file.display()))?;
    let sidecar = file.with_extension("patch.json");
    let patches = match std::fs::read_to_string(&sidecar) {
        Ok(s) => serde_json::from_str(&s).map_err(|e| format!("{}: {e}", sidecar.display()))?,
        Err(_) => PatchTable { rom: stem(file), entries: vec![] },
    };
    Ok((rom, patches))
}

fn main() {
    if let Err(e) = real_main() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn real_main() -> Result<(), String> {
    match Cli::parse().cmd {
        Cmd::Autopsy { file, requirements } => {
            let report = if file.extension().map(|e| e == "ch8").unwrap_or(false) {
                let (rom, _) = read_rom(&file)?;
                let (ir, w) = lift_rom(&stem(&file), &rom);
                scan(&ir, w)
            } else {
                let src = std::fs::read_to_string(&file).map_err(|e| e.to_string())?;
                let (ir, w) = necro_cobol_lifter::lift_source(&stem(&file), &src)?;
                scan(&ir, w)
            };
            if requirements {
                println!("{}", report.requirements_markdown);
            } else {
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            }
        }
        Cmd::Lift { file, out } => {
            let wat = if file.extension().map(|e| e == "ch8").unwrap_or(false) {
                let (rom, patches) = read_rom(&file)?;
                romwat::emit_wat(&stem(&file), &rom, &patches)
            } else {
                let src = std::fs::read_to_string(&file).map_err(|e| e.to_string())?;
                let (ir, _) = necro_cobol_lifter::lift_source(&stem(&file), &src)?;
                necro_cobol_lifter::emit_wat(&ir)
            };
            match out {
                Some(p) => std::fs::write(&p, wat).map_err(|e| e.to_string())?,
                None => print!("{wat}"),
            }
        }
        Cmd::Assemble { file } => {
            let src = std::fs::read_to_string(&file).map_err(|e| e.to_string())?;
            let a = asm::assemble(&stem(&file), &src)?;
            let ch8 = file.with_extension("ch8");
            std::fs::write(&ch8, &a.bytes).map_err(|e| e.to_string())?;
            std::fs::write(file.with_extension("patch.json"), serde_json::to_string_pretty(&a.patches).unwrap()).map_err(|e| e.to_string())?;
            println!("{}: {} bytes, {} patch points, {} labels", ch8.display(), a.bytes.len(), a.patches.entries.len(), a.labels.len());
        }
        Cmd::Observatory { serve, port, root, .. } if serve => necro_daemon::serve(root, &necro_daemon::AppSpec::observatory(), port)?,
        Cmd::Observatory { out, samples, bench, .. } => {
            let html = observatory::build(&samples, bench.as_deref())?;
            if let Some(d) = out.parent() {
                std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
            }
            std::fs::write(&out, &html).map_err(|e| e.to_string())?;
            let fdir = out.parent().unwrap_or(std::path::Path::new(".")).join("fonts");
            std::fs::create_dir_all(&fdir).map_err(|e| e.to_string())?;
            for (name, bytes) in necro_daemon::FONTS {
                std::fs::write(fdir.join(name), bytes).map_err(|e| e.to_string())?;
            }
            println!("{}: {} bytes, fonts in {}", out.display(), html.len(), fdir.display());
        }
        Cmd::Mcp => mcp::serve()?,
        Cmd::Benchmark { vectors, seed, rom_steps, fuel, live_kiro, json, samples } => {
            benchmark(vectors, seed, rom_steps, fuel, live_kiro, json, samples)?;
        }
    }
    Ok(())
}

fn benchmark(vectors: usize, seed: u64, rom_steps: usize, fuel: u64, live_kiro: bool, json: Option<PathBuf>, samples: PathBuf) -> Result<(), String> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let bridge = if live_kiro { KiroBridge::live(Duration::from_secs(120)) } else { KiroBridge::offline() };
    let mut rows: Vec<BenchRow> = Vec::new();
    let mut traces: Vec<serde_json::Value> = Vec::new();
    let mut cob: Vec<PathBuf> = std::fs::read_dir(samples.join("cobol")).map_err(|e| e.to_string())?.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map(|x| x == "cob").unwrap_or(false)).collect();
    cob.sort();
    let mut roms: Vec<PathBuf> = std::fs::read_dir(samples.join("roms")).map_err(|e| e.to_string())?.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map(|x| x == "ch8").unwrap_or(false)).collect();
    roms.sort();
    println!("NecroCore benchmark: {vectors} vectors per target, seed {seed}, COBOL fuel {fuel} blocks, ROM {rom_steps} steps per vector");
    println!("host: {} {}, necro-cli {}, wasmtime 50 (Cranelift), release build", std::env::consts::OS, std::env::consts::ARCH, env!("CARGO_PKG_VERSION"));
    println!();
    for f in &cob {
        let name = stem(f);
        let src = std::fs::read_to_string(f).map_err(|e| e.to_string())?;
        let t = Instant::now();
        let (ir, w) = necro_cobol_lifter::lift_source(&name, &src)?;
        let wat = necro_cobol_lifter::emit_wat(&ir);
        let lift_us = t.elapsed().as_secs_f64() * 1e6;
        let report = scan(&ir, w);
        let healed = heal(&ir, &report);
        let again = scan(&healed, vec![]);
        let parity = prove_cobol(&ir, &wat, vectors, seed, fuel)?;
        let prop = rt.block_on(bridge.specify(&report));
        if let Some(t) = &prop.trace {
            traces.push(serde_json::json!({"program": name, "kind": "cobol", "pathologies": report.pathologies.len(), "trace": t}));
        }
        rows.push(BenchRow {
            target: name,
            kind: "cobol".into(),
            lift_us,
            blocks: report.block_count,
            edges: report.edge_count,
            ghosts: report.count(PathologyKind::Ghost),
            vampires: report.count(PathologyKind::Vampire),
            zombies: report.count(PathologyKind::Zombie),
            warnings: report.lifter_warnings.len(),
            healed_remaining: again.pathologies.len(),
            parity,
            bridge_source: prop.source,
            bridge_patches: prop.patches.len(),
            bridge_ms: prop.elapsed_ms,
        });
    }
    for f in &roms {
        let name = stem(f);
        let (rom, patches) = read_rom(f)?;
        let t = Instant::now();
        let (ir, w) = lift_rom(&name, &rom);
        let wat = romwat::emit_wat(&name, &rom, &patches);
        let lift_us = t.elapsed().as_secs_f64() * 1e6;
        let report = scan(&ir, w);
        let healed = heal(&ir, &report);
        let again = scan(&healed, vec![]);
        let parity = prove_chip8(&name, &rom, &patches, &wat, vectors, seed, rom_steps)?;
        let prop = rt.block_on(bridge.specify(&report));
        if let Some(t) = &prop.trace {
            traces.push(serde_json::json!({"program": name, "kind": "chip8", "pathologies": report.pathologies.len(), "trace": t}));
        }
        rows.push(BenchRow {
            target: name,
            kind: "chip8".into(),
            lift_us,
            blocks: report.block_count,
            edges: report.edge_count,
            ghosts: report.count(PathologyKind::Ghost),
            vampires: report.count(PathologyKind::Vampire),
            zombies: report.count(PathologyKind::Zombie),
            warnings: report.lifter_warnings.len(),
            healed_remaining: again.pathologies.len(),
            parity,
            bridge_source: prop.source,
            bridge_patches: prop.patches.len(),
            bridge_ms: prop.elapsed_ms,
        });
    }
    println!("| target | kind | lift us | blocks | edges | ghosts | vampires | zombies | warnings | left after heal | parity | interp us/vec | wasm us/vec | speedup | compile us | non-halting vecs | bridge |");
    println!("|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|");
    for r in &rows {
        println!(
            "| {} | {} | {:.1} | {} | {} | {} | {} | {} | {} | {} | {:.2}% ({}/{}) | {:.2} | {:.2} | {:.2}x | {:.0} | {} | {} ({} patches, {} ms) |",
            r.target, r.kind, r.lift_us, r.blocks, r.edges, r.ghosts, r.vampires, r.zombies, r.warnings, r.healed_remaining, r.parity.parity_pct, r.parity.matched, r.parity.vectors, r.parity.interp_per_vector_us, r.parity.wasm_per_vector_us, r.parity.speedup, r.parity.compile_us, r.parity.non_halting, r.bridge_source, r.bridge_patches, r.bridge_ms
        );
    }
    println!();
    let found: usize = rows.iter().map(|r| r.ghosts + r.vampires + r.zombies).sum();
    let left: usize = rows.iter().map(|r| r.healed_remaining).sum();
    let total_vec: usize = rows.iter().map(|r| r.parity.vectors).sum();
    let total_match: usize = rows.iter().map(|r| r.parity.matched).sum();
    println!("pathologies found: {found}, remaining after heal: {left}, healed: {}", found - left.min(found));
    println!("vectors: {total_vec}, matched: {total_match}, overall parity: {:.3}%", if total_vec > 0 { 100.0 * total_match as f64 / total_vec as f64 } else { 0.0 });
    for r in &rows {
        for (st, n) in &r.parity.status_histogram {
            println!("  {}: {st} x {n}", r.target);
        }
        for d in r.parity.divergences.iter().take(3) {
            println!("  DIVERGENCE {} #{} seed {} inputs {:?}: interp {} | wasm {}", r.target, d.index, d.seed, d.inputs, d.interpreter, d.wasm);
        }
    }
    if live_kiro {
        let live = rows.iter().filter(|r| r.bridge_source == "kiro-cli").count();
        println!("live kiro-cli turns: {live} of {} ({} ms in total)", rows.len(), rows.iter().filter(|r| r.bridge_source == "kiro-cli").map(|r| r.bridge_ms).sum::<u128>());
        let version = std::process::Command::new(&bridge.cli).arg("--version").output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
        let doc = serde_json::json!({
            "about": "Real kiro-cli transcripts captured by `necro-cli benchmark --live-kiro`. Each entry is one chat turn with tools disabled.",
            "kiro_cli_version": version,
            "host": format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
            "live_turns": live,
            "fell_back_to_offline": rows.len() - live,
            "turns": traces,
            "rows": rows.iter().map(|r| serde_json::json!({"target": r.target, "bridge": r.bridge_source, "ms": r.bridge_ms})).collect::<Vec<_>>(),
        });
        let tp = std::path::Path::new("benchmarks/live-kiro-trace.json");
        if let Some(d) = tp.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        std::fs::write(tp, serde_json::to_string_pretty(&doc).unwrap()).map_err(|e| e.to_string())?;
        println!("wrote {}", tp.display());
    }
    if let Some(p) = json {
        std::fs::write(&p, serde_json::to_string_pretty(&rows).unwrap()).map_err(|e| e.to_string())?;
        println!("wrote {}", p.display());
    }
    Ok(())
}
