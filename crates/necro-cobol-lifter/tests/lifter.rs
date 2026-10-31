use necro_cobol_lifter::{emit_wat, exec, lift_source};
use necro_core::{scan, PathologyKind, StoreMode, VarDecl};
use proptest::prelude::*;

fn sample(name: &str) -> String {
    std::fs::read_to_string(format!("{}/../../samples/cobol/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

fn run_wasm(wat: &str, ir: &necro_core::NecroIR, raw: &[i64], fuel: u64) -> exec::Outcome {
    let engine = wasmtime::Engine::default();
    let module = wasmtime::Module::new(&engine, wat).expect("valid wat");
    let mut store = wasmtime::Store::new(&engine, ());
    let inst = wasmtime::Instance::new(&mut store, &module, &[]).unwrap();
    let mem = inst.get_memory(&mut store, "mem").unwrap();
    for (i, v) in raw.iter().enumerate() {
        mem.write(&mut store, 8 * i, &v.to_le_bytes()).unwrap();
    }
    let run = inst.get_typed_func::<i64, i32>(&mut store, "run").unwrap();
    let status = match run.call(&mut store, fuel as i64) {
        Ok(c) => exec::Status::from_code(c),
        Err(_) => exec::Status::Trap,
    };
    let mut vars = Vec::new();
    for i in 0..ir.variables.len() {
        let mut b = [0u8; 8];
        mem.read(&store, 8 * i, &mut b).unwrap();
        vars.push(i64::from_le_bytes(b));
    }
    exec::Outcome { status, vars, blocks_executed: 0 }
}

#[test]
fn bank_interest_lifts_and_has_a_ghost() {
    let (ir, warnings) = lift_source("BANKINT", &sample("BANK-INTEREST-1978.cob")).unwrap();
    let report = scan(&ir, warnings);
    assert!(report.pathologies.iter().any(|p| p.kind == PathologyKind::Ghost && p.block.starts_with("AUDIT-TRAIL")), "{report:#?}");
    assert!(report.lifter_warnings.iter().any(|w| w.contains("two-digit year")));
    let out = exec::run_ir(&ir, exec::initial_values(&ir), 100_000);
    assert_eq!(out.status, exec::Status::Halted);
    let bal = ir.var_index("WS-BALANCE").unwrap();
    assert!(out.vars[bal] > 200_000, "balance {} should pass the 2000.00 target", out.vars[bal]);
    let yrs = ir.var_index("WS-YEARS").unwrap();
    assert_eq!(out.vars[yrs], 52, "26 - 78 = -52 stored unsigned in PIC 99");
}

#[test]
fn payroll_has_zombie_and_ghost() {
    let (ir, w) = lift_source("PAYBATCH", &sample("PAYROLL-BATCH-1982.cob")).unwrap();
    let report = scan(&ir, w);
    assert_eq!(report.count(PathologyKind::Zombie), 1, "{report:#?}");
    assert!(report.count(PathologyKind::Ghost) >= 1);
    let out = exec::run_ir(&ir, exec::initial_values(&ir), 5_000);
    assert_eq!(out.status, exec::Status::FuelExhausted);
}

#[test]
fn wire_clearing_has_vampire() {
    let (ir, w) = lift_source("WIRECLR", &sample("WIRE-CLEARING-1985.cob")).unwrap();
    let report = scan(&ir, w);
    let v = report.pathologies.iter().find(|p| p.kind == PathologyKind::Vampire).expect("vampire");
    assert_eq!(v.variable.as_deref(), Some("WS-QUEUE-DEPTH"));
    assert!(report.requirements_markdown.contains("THE SYSTEM SHALL"));
    // Edge input: batch size 99 never terminates because WS-SEQ wraps at 100.
    let mut raw = exec::initial_values(&ir);
    raw[ir.var_index("WS-BATCH-SIZE").unwrap()] = 99;
    let out = exec::run_ir(&ir, raw, 20_000);
    assert_eq!(out.status, exec::Status::FuelExhausted);
}

#[test]
fn interpreter_and_wasm_agree_on_all_samples() {
    for name in ["BANK-INTEREST-1978.cob", "PAYROLL-BATCH-1982.cob", "WIRE-CLEARING-1985.cob"] {
        let (ir, _) = lift_source(name, &sample(name)).unwrap();
        let wat = emit_wat(&ir);
        let raw = exec::initial_values(&ir);
        let a = exec::run_ir(&ir, raw.clone(), 3_000);
        let b = run_wasm(&wat, &ir, &raw, 3_000);
        assert_eq!(a.status, b.status, "{name}");
        assert_eq!(a.vars, b.vars, "{name}");
    }
}

#[test]
fn healed_payroll_terminates() {
    let (ir, w) = lift_source("PAYBATCH", &sample("PAYROLL-BATCH-1982.cob")).unwrap();
    let report = scan(&ir, w);
    let healed = necro_core::heal(&ir, &report);
    let out = exec::run_ir(&healed, exec::initial_values(&healed), 10_000_000);
    assert_eq!(out.status, exec::Status::Halted);
    let again = scan(&healed, vec![]);
    assert_eq!(again.count(PathologyKind::Zombie), 0, "{again:#?}");
    assert_eq!(again.count(PathologyKind::Ghost), 0, "{:?}", again.pathologies.iter().map(|p| (&p.block, &p.detail)).collect::<Vec<_>>());
}

proptest! {
    #[test]
    fn store_never_exceeds_pic_width(v in -10_000_000_000i64..10_000_000_000, int_digits in 1u32..9, scale in 0u32..4, signed: bool) {
        let d = VarDecl { name: "X".into(), int_digits, scale, signed, is_input: false, initial: 0, span: Default::default() };
        let raw = necro_cobol_lifter::fixed::store(v, &d, StoreMode::Truncate);
        prop_assert!(raw.abs() < 10i64.pow(int_digits + scale));
        prop_assert!(signed || raw >= 0);
    }

    #[test]
    fn literal_roundtrip(ip in 0i64..1_000_000, fp in 0i64..10_000) {
        let text = format!("{ip}.{fp:04}");
        let v = necro_cobol_lifter::fixed::literal(&text).unwrap();
        prop_assert_eq!(v, ip * 10_000 + fp);
    }
}
