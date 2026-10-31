//! necro-prover: differential shadow execution.
//!
//! Every vector is derived from one seed, runs through the reference interpreter
//! and the lifted WebAssembly module (wasmtime, Cranelift), and the complete
//! observable state is compared. Timings are wall-clock microseconds measured
//! around the execution calls only (module compilation is reported separately).
use necro_cobol_lifter::exec::{self, Outcome, Status};
use necro_core::NecroIR;
use necro_rom_lifter::{layout, Chip8, PatchTable, StepStatus};
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Divergence {
    pub index: usize,
    pub seed: u64,
    pub inputs: Vec<(String, i64)>,
    pub interpreter: String,
    pub wasm: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParityResult {
    pub target: String,
    pub vectors: usize,
    pub matched: usize,
    pub parity_pct: f64,
    pub divergences: Vec<Divergence>,
    pub interp_total_us: f64,
    pub wasm_total_us: f64,
    pub interp_per_vector_us: f64,
    pub wasm_per_vector_us: f64,
    pub speedup: f64,
    pub compile_us: f64,
    /// Vectors whose run ended by fuel exhaustion (COBOL) or a non-Ok status (CHIP-8), identically on both sides.
    pub non_halting: usize,
    pub status_histogram: Vec<(String, usize)>,
}

/// SplitMix64: one seed -> a deterministic stream.
#[derive(Clone)]
pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed)
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    pub fn below(&mut self, n: u64) -> u64 {
        if n == 0 {
            0
        } else {
            self.next_u64() % n
        }
    }
}

fn histogram(statuses: &[String]) -> Vec<(String, usize)> {
    let mut h: Vec<(String, usize)> = Vec::new();
    for s in statuses {
        match h.iter_mut().find(|(k, _)| k == s) {
            Some((_, c)) => *c += 1,
            None => h.push((s.clone(), 1)),
        }
    }
    h.sort();
    h
}

/// Seeds the level-77 inputs of a COBOL program. Values stay inside each PIC
/// width, with a bias toward edge values (0, 1, max, max-1).
pub fn cobol_vector(ir: &NecroIR, seed: u64) -> Vec<i64> {
    let mut rng = Rng::new(seed);
    let mut raw = exec::initial_values(ir);
    for (i, v) in ir.variables.iter().enumerate() {
        if !v.is_input {
            continue;
        }
        let max = 10i64.pow((v.int_digits + v.scale).min(18)) - 1;
        let r = rng.below(8);
        raw[i] = match r {
            0 => 0,
            1 => 1,
            2 => max,
            3 => max - 1,
            _ => rng.below(max as u64 + 1) as i64,
        };
        if v.signed && rng.below(2) == 1 {
            raw[i] = -raw[i];
        }
    }
    raw
}

pub struct CobolModule {
    engine: wasmtime::Engine,
    module: wasmtime::Module,
    pub compile_us: f64,
}

impl CobolModule {
    pub fn compile(wat: &str) -> Result<CobolModule, String> {
        let engine = wasmtime::Engine::default();
        let t = Instant::now();
        let module = wasmtime::Module::new(&engine, wat).map_err(|e| e.to_string())?;
        Ok(CobolModule { engine, module, compile_us: t.elapsed().as_secs_f64() * 1e6 })
    }
}

/// Runs `n` seeded vectors through the COBOL interpreter and the wasm module.
pub fn prove_cobol(ir: &NecroIR, wat: &str, n: usize, seed: u64, fuel: u64) -> Result<ParityResult, String> {
    prove_cobol_at(ir, wat, 0, n, seed, fuel)
}

/// Same as `prove_cobol` but runs vectors `start..start+n` of the seeded sequence,
/// so ten batches of 1,000 cover exactly the vectors one batch of 10,000 would.
pub fn prove_cobol_at(ir: &NecroIR, wat: &str, start: usize, n: usize, seed: u64, fuel: u64) -> Result<ParityResult, String> {
    let m = CobolModule::compile(wat)?;
    let mut store = wasmtime::Store::new(&m.engine, ());
    let inst = wasmtime::Instance::new(&mut store, &m.module, &[]).map_err(|e| e.to_string())?;
    let mem = inst.get_memory(&mut store, "mem").ok_or("no memory export")?;
    let run = inst.get_typed_func::<i64, i32>(&mut store, "run").map_err(|e| e.to_string())?;
    let reset = inst.get_typed_func::<(), ()>(&mut store, "reset_stack").map_err(|e| e.to_string())?;
    let nv = ir.variables.len();
    let mut matched = 0usize;
    let mut divergences = Vec::new();
    let mut interp_us = 0f64;
    let mut wasm_us = 0f64;
    let mut statuses = Vec::with_capacity(n);
    let mut non_halting = 0usize;
    for k in start..start + n {
        let vseed = seed.wrapping_mul(1_000_003).wrapping_add(k as u64);
        let raw = cobol_vector(ir, vseed);
        let t = Instant::now();
        let a: Outcome = exec::run_ir(ir, raw.clone(), fuel);
        interp_us += t.elapsed().as_secs_f64() * 1e6;

        let t = Instant::now();
        for (i, v) in raw.iter().enumerate() {
            mem.write(&mut store, 8 * i, &v.to_le_bytes()).map_err(|e| e.to_string())?;
        }
        reset.call(&mut store, ()).map_err(|e| e.to_string())?;
        let status = match run.call(&mut store, fuel as i64) {
            Ok(c) => Status::from_code(c),
            Err(_) => Status::Trap,
        };
        let mut vars = Vec::with_capacity(nv);
        let mut buf = [0u8; 8];
        for i in 0..nv {
            mem.read(&store, 8 * i, &mut buf).map_err(|e| e.to_string())?;
            vars.push(i64::from_le_bytes(buf));
        }
        wasm_us += t.elapsed().as_secs_f64() * 1e6;

        statuses.push(format!("{:?}", a.status));
        if a.status != Status::Halted {
            non_halting += 1;
        }
        if a.status == status && a.vars == vars {
            matched += 1;
        } else if divergences.len() < 50 {
            let inputs = ir.variables.iter().enumerate().filter(|(_, v)| v.is_input).map(|(i, v)| (v.name.clone(), raw[i])).collect();
            divergences.push(Divergence {
                index: k,
                seed: vseed,
                inputs,
                interpreter: format!("{:?} {:?}", a.status, a.vars),
                wasm: format!("{status:?} {vars:?}"),
            });
        }
    }
    Ok(finish(ir.name.clone(), n, matched, divergences, interp_us, wasm_us, m.compile_us, non_halting, histogram(&statuses)))
}

#[allow(clippy::too_many_arguments)]
fn finish(target: String, n: usize, matched: usize, divergences: Vec<Divergence>, interp_us: f64, wasm_us: f64, compile_us: f64, non_halting: usize, status_histogram: Vec<(String, usize)>) -> ParityResult {
    ParityResult {
        target,
        vectors: n,
        matched,
        parity_pct: if n == 0 { 0.0 } else { 100.0 * matched as f64 / n as f64 },
        divergences,
        interp_total_us: interp_us,
        wasm_total_us: wasm_us,
        interp_per_vector_us: if n == 0 { 0.0 } else { interp_us / n as f64 },
        wasm_per_vector_us: if n == 0 { 0.0 } else { wasm_us / n as f64 },
        speedup: if wasm_us > 0.0 { interp_us / wasm_us } else { 0.0 },
        compile_us,
        non_halting,
        status_histogram,
    }
}

/// One CHIP-8 vector: an RNG seed, a key schedule (the key mask changes every
/// `KEY_PERIOD` steps) and, halfway through, a hot patch of every patch point to
/// a seeded value. Both machines must stay byte-identical through all of it.
pub const KEY_PERIOD: usize = 25;

pub fn prove_chip8(name: &str, rom: &[u8], patches: &PatchTable, wat: &str, n: usize, seed: u64, steps: usize) -> Result<ParityResult, String> {
    prove_chip8_at(name, rom, patches, wat, 0, n, seed, steps)
}

/// Vectors `start..start+n` of the seeded sequence (see `prove_cobol_at`).
#[allow(clippy::too_many_arguments)]
pub fn prove_chip8_at(name: &str, rom: &[u8], patches: &PatchTable, wat: &str, start: usize, n: usize, seed: u64, steps: usize) -> Result<ParityResult, String> {
    let engine = wasmtime::Engine::default();
    let t = Instant::now();
    let module = wasmtime::Module::new(&engine, wat).map_err(|e| e.to_string())?;
    let compile_us = t.elapsed().as_secs_f64() * 1e6;
    let mut store = wasmtime::Store::new(&engine, ());
    let inst = wasmtime::Instance::new(&mut store, &module, &[]).map_err(|e| e.to_string())?;
    let mem = inst.get_memory(&mut store, "mem").ok_or("no memory export")?;
    let step = inst.get_typed_func::<i32, i32>(&mut store, "step").map_err(|e| e.to_string())?;
    let mut matched = 0usize;
    let mut divergences = Vec::new();
    let mut interp_us = 0f64;
    let mut wasm_us = 0f64;
    let mut statuses = Vec::with_capacity(n);
    let mut non_halting = 0usize;
    let mut wasm_image = vec![0u8; layout::IMAGE_SIZE];
    for k in start..start + n {
        let vseed = seed.wrapping_mul(1_000_003).wrapping_add(k as u64);
        let mut rng = Rng::new(vseed);
        let rng_seed = rng.next_u64() as u32;
        let keys: Vec<u16> = (0..steps.div_ceil(KEY_PERIOD)).map(|_| (rng.next_u64() & 0xFFFF) as u16).collect();
        let patch_vals: Vec<u8> = patches.entries.iter().map(|_| (rng.next_u64() & 0xFF) as u8).collect();
        let patch_at = steps / 2;

        // Interpreter.
        let mut a = Chip8::new(rom, rng_seed);
        let t = Instant::now();
        let mut a_status = StepStatus::Ok;
        let mut done = 0usize;
        'outer: while done < steps {
            if done == patch_at {
                for (e, v) in patches.entries.iter().zip(&patch_vals) {
                    patches.apply(&mut a.image, &e.name, *v);
                }
            }
            a.set_keys(keys[done / KEY_PERIOD]);
            let chunk = KEY_PERIOD.min(steps - done).min(patch_at.saturating_sub(done).max(1));
            let chunk = if done < patch_at { chunk.min(patch_at - done) } else { chunk };
            for _ in 0..chunk {
                let s = a.step();
                done += 1;
                if s != StepStatus::Ok {
                    a_status = s;
                    break 'outer;
                }
            }
        }
        interp_us += t.elapsed().as_secs_f64() * 1e6;

        // WebAssembly: same schedule, applied to the module's memory.
        let fresh = Chip8::new(rom, rng_seed);
        let t = Instant::now();
        mem.write(&mut store, 0, &fresh.image).map_err(|e| e.to_string())?;
        let mut b_status = StepStatus::Ok;
        let mut done = 0usize;
        while done < steps {
            if done == patch_at {
                mem.read(&store, 0, &mut wasm_image).map_err(|e| e.to_string())?;
                for (e, v) in patches.entries.iter().zip(&patch_vals) {
                    if let Some(addr) = patches.apply(&mut wasm_image, &e.name, *v) {
                        mem.write(&mut store, addr, &wasm_image[addr..addr + 1]).map_err(|e| e.to_string())?;
                    }
                }
            }
            mem.write(&mut store, layout::KEYS, &(keys[done / KEY_PERIOD] as u32).to_le_bytes()).map_err(|e| e.to_string())?;
            let chunk = KEY_PERIOD.min(steps - done);
            let chunk = if done < patch_at { chunk.min(patch_at - done) } else { chunk };
            let code = step.call(&mut store, chunk as i32).map_err(|e| e.to_string())?;
            if code != 0 {
                b_status = StepStatus::from_code(code);
                // the module does not report how many steps ran; recover it from the interpreter side
                break;
            }
            done += chunk;
        }
        mem.read(&store, 0, &mut wasm_image).map_err(|e| e.to_string())?;
        wasm_us += t.elapsed().as_secs_f64() * 1e6;

        statuses.push(format!("{a_status:?}"));
        if a_status != StepStatus::Ok {
            non_halting += 1;
        }
        if a_status == b_status && a.image == wasm_image {
            matched += 1;
        } else if divergences.len() < 50 {
            let first_diff = a.image.iter().zip(&wasm_image).position(|(x, y)| x != y);
            divergences.push(Divergence {
                index: k,
                seed: vseed,
                inputs: vec![("rng_seed".into(), rng_seed as i64), ("first_key_mask".into(), keys[0] as i64)],
                interpreter: format!("{a_status:?} pc={:#05X}", a.pc()),
                wasm: format!("{b_status:?} first differing byte at {:?}", first_diff.map(|o| format!("{o:#06X}"))),
            });
        }
    }
    Ok(finish(name.to_string(), n, matched, divergences, interp_us, wasm_us, compile_us, non_halting, histogram(&statuses)))
}
