use necro_core::{scan, PathologyKind};
use necro_rom_lifter::{asm, decode, layout, lift::lift_rom, wat, Chip8, PatchTable, StepStatus};

fn rom(name: &str) -> (Vec<u8>, PatchTable) {
    let dir = format!("{}/../../samples/roms", env!("CARGO_MANIFEST_DIR"));
    let src = std::fs::read_to_string(format!("{dir}/{name}.asm")).unwrap();
    let a = asm::assemble(name, &src).unwrap();
    // The committed binary must match the source.
    let committed = std::fs::read(format!("{dir}/{name}.ch8")).unwrap();
    assert_eq!(a.bytes, committed, "{name}.ch8 is stale; run necro-cli assemble");
    (a.bytes, a.patches)
}

fn run_wasm(name: &str, bytes: &[u8], patches: &PatchTable, seed: u32, steps: i32, keys: u16) -> (StepStatus, Vec<u8>) {
    let text = wat::emit_wat(name, bytes, patches);
    let engine = wasmtime::Engine::default();
    let module = wasmtime::Module::new(&engine, &text).expect("valid wat");
    let mut store = wasmtime::Store::new(&engine, ());
    let inst = wasmtime::Instance::new(&mut store, &module, &[]).unwrap();
    let mem = inst.get_memory(&mut store, "mem").unwrap();
    let m = Chip8::new(bytes, seed);
    mem.write(&mut store, 0, &m.image).unwrap();
    mem.write(&mut store, layout::KEYS, &(keys as u32).to_le_bytes()).unwrap();
    let step = inst.get_typed_func::<i32, i32>(&mut store, "step").unwrap();
    let code = step.call(&mut store, steps).unwrap();
    let mut img = vec![0u8; layout::IMAGE_SIZE];
    mem.read(&store, 0, &mut img).unwrap();
    (StepStatus::from_code(code), img)
}

#[test]
fn games_run_and_draw() {
    for name in ["PONG-STYLE", "INVADERS-STYLE", "BREAKOUT-STYLE"] {
        let (bytes, _) = rom(name);
        let mut m = Chip8::new(&bytes, 1);
        let mut max_lit = 0;
        for n in 0..400 {
            let st = m.step();
            assert_eq!(st, StepStatus::Ok, "{name} stopped after {n} steps at pc {:#05X}", m.pc());
            max_lit = max_lit.max(m.lit_pixels());
        }
        assert!(max_lit > 4, "{name} drew nothing (max {max_lit} pixels lit)");
    }
}

#[test]
fn pong_ball_moves_and_paddle_follows_keys() {
    let (bytes, _) = rom("PONG-STYLE");
    let mut m = Chip8::new(&bytes, 3);
    m.run(60);
    let x0 = m.v(4);
    m.run(120);
    assert_ne!(m.v(4), x0, "ball x did not change");
    let mut held = Chip8::new(&bytes, 3);
    held.set_keys(1 << 4); // key 4: left paddle down
    held.run(300);
    assert!(held.v(2) > 12, "left paddle should have moved down from 12, is {}", held.v(2));
}

#[test]
fn invaders_march_and_shot_fires() {
    let (bytes, _) = rom("INVADERS-STYLE");
    let mut m = Chip8::new(&bytes, 9);
    m.run(100);
    let base0 = m.v(7);
    m.run(1500);
    assert_ne!(m.v(7), base0, "invaders did not move");
    let mut f = Chip8::new(&bytes, 9);
    f.set_keys(1 << 5);
    f.run(200);
    assert_eq!(f.v(6), 1, "shot should be active after pressing 5");
}

#[test]
fn breakout_destroys_a_brick_eventually() {
    let (bytes, _) = rom("BREAKOUT-STYLE");
    let mut m = Chip8::new(&bytes, 5);
    m.run(5000);
    assert!(m.v(8) != 0xFF || m.v(9) != 0xFF, "no brick was hit in 5000 steps (pc {:#05X})", m.pc());
}

#[test]
fn hot_patch_changes_one_byte_and_keeps_state() {
    let (bytes, patches) = rom("PONG-STYLE");
    let mut m = Chip8::new(&bytes, 2);
    m.run(100);
    let before = m.image.clone();
    let addr = patches.apply(&mut m.image, "PADDLE_H", 9).unwrap();
    let diffs: Vec<usize> = before.iter().zip(&m.image).enumerate().filter(|(_, (a, b))| a != b).map(|(i, _)| i).collect();
    assert_eq!(diffs, vec![addr]);
    assert_eq!(patches.read(&m.image, "PADDLE_H"), Some(9));
    let (st, _) = m.run(300);
    assert_eq!(st, StepStatus::Ok);
}

#[test]
fn interpreter_and_wasm_agree_with_and_without_keys() {
    for name in ["PONG-STYLE", "INVADERS-STYLE", "BREAKOUT-STYLE"] {
        let (bytes, patches) = rom(name);
        for keys in [0u16, 1 << 4, (1 << 5) | (1 << 6), 0xFFFF] {
            let mut m = Chip8::new(&bytes, 42);
            m.set_keys(keys);
            let (st, _) = m.run(500);
            let (ws, img) = run_wasm(name, &bytes, &patches, 42, 500, keys);
            assert_eq!(st, ws, "{name} keys {keys:#06X}");
            if let Some(off) = m.image.iter().zip(&img).position(|(a, b)| a != b) {
                panic!("{name} keys {keys:#06X}: first differing byte at {off:#06X} (interp {:#04X}, wasm {:#04X}, pc {:#05X})", m.image[off], img[off], m.pc());
            }
        }
    }
}

#[test]
fn scanner_finds_the_dropped_subroutine_and_the_spin_halt() {
    let (bytes, _) = rom("PONG-STYLE");
    let (ir, w) = lift_rom("PONG-STYLE", &bytes);
    let report = scan(&ir, w);
    assert!(report.pathologies.iter().any(|p| p.kind == PathologyKind::Ghost && p.block.starts_with("orphan_")), "{report:#?}");
    assert!(report.pathologies.iter().any(|p| p.kind == PathologyKind::Zombie && p.detail.contains("spin halt")), "{report:#?}");
}

#[test]
fn assembler_roundtrips_through_the_decoder() {
    let src = "a:\n CLS\n RET\n JP a\n CALL a\n SE V1, 0x2A\n SNE V1, 7\n SE V1, V2\n LD V3, 200\n ADD V3, 1\n LD V3, V4\n OR V3, V4\n AND V3, V4\n XOR V3, V4\n ADD V3, V4\n SUB V3, V4\n SHR V3\n SUBN V3, V4\n SHL V3\n SNE V3, V4\n LD I, a\n JP V0, a\n RND V5, 0x0F\n DRW V1, V2, 5\n SKP V6\n SKNP V6\n LD V7, DT\n LD V7, K\n LD DT, V7\n LD ST, V7\n ADD I, V7\n LD F, V7\n LD B, V7\n LD [I], V7\n LD V7, [I]\n";
    let a = asm::assemble("t", src).unwrap();
    let expected = [
        0x00E0, 0x00EE, 0x1200, 0x2200, 0x312A, 0x4107, 0x5120, 0x63C8, 0x7301, 0x8340, 0x8341, 0x8342, 0x8343, 0x8344, 0x8345, 0x8306, 0x8347, 0x830E, 0x9340, 0xA200, 0xB200, 0xC50F, 0xD125, 0xE69E, 0xE6A1, 0xF707, 0xF70A, 0xF715, 0xF718, 0xF71E, 0xF729, 0xF733, 0xF755, 0xF765,
    ];
    for (i, w) in expected.iter().enumerate() {
        let got = ((a.bytes[2 * i] as u16) << 8) | a.bytes[2 * i + 1] as u16;
        assert_eq!(got, *w, "instruction {i}");
        assert!(!matches!(decode(got), necro_rom_lifter::Op::Unknown(_)));
    }
}

#[test]
fn pong_second_ball_switches_on_live_and_wasm_agrees() {
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../samples/roms/PONG-STYLE.asm")).unwrap();
    let a = asm::assemble("PONG-STYLE", &src).unwrap();
    assert!(a.patches.get("BALL2_ON").is_some() && a.patches.get("FRAME_DELAY").is_some());
    let lit = |on: bool| {
        let mut c = Chip8::new(&a.bytes, 3);
        let mut most = 0usize;
        for k in 0..2000 {
            if k == 100 && on {
                a.patches.apply(&mut c.image, "BALL2_ON", 1);
            }
            c.step();
            let px: usize = c.image[layout::DISPLAY..layout::DISPLAY + 256].iter().map(|b| b.count_ones() as usize).sum();
            most = most.max(px);
        }
        most
    };
    assert!(lit(true) > lit(false), "second ball adds lit pixels");
}

const SCHIP_PROBE: &str = "
    HIGH
    LD V0, 3
    LD V1, 5
    LD I, spr
    DRW V0, V1, 0
    SCD 4
    SCR
    SCL
    SCL
    LD V2, 0x11
    LD V3, 0x22
    LD V4, 0x33
    LD R, V4
    LD V2, 0
    LD V3, 0
    LD V4, 0
    LD V4, R
    LD VE, 1
    SCD 63
    LOW
    LD I, spr
    DRW V0, V1, 0
    DRW V0, V1, 3
    HIGH
    LD V0, 120
    LD V1, 60
    DRW V0, V1, 0
done:
    JP done
spr:
    DB 0xFF, 0xFF, 0x80, 0x01, 0xBF, 0xFD, 0xA0, 0x05
    DB 0xAF, 0xF5, 0xA8, 0x15, 0xAB, 0xD5, 0xAA, 0x55
    DB 0xAA, 0x55, 0xAB, 0xD5, 0xA8, 0x15, 0xAF, 0xF5
    DB 0xA0, 0x05, 0xBF, 0xFD, 0x80, 0x01, 0xFF, 0xFF
";

#[test]
fn schip_opcodes_agree_between_interpreter_and_wasm() {
    let a = asm::assemble("SCHIP-PROBE", SCHIP_PROBE).unwrap();
    let mut m = Chip8::new(&a.bytes, 5);
    let (st, _) = m.run(100);
    assert_eq!(st, StepStatus::Ok);
    assert!(m.hires(), "the probe ends in high-res mode");
    assert_eq!((m.v(2), m.v(3), m.v(4)), (0x11, 0x22, 0x33), "Fx75 then Fx85 restores V0..V4");
    let (st2, img) = run_wasm("SCHIP-PROBE", &a.bytes, &a.patches, 5, 100, 0);
    assert_eq!(st2, StepStatus::Ok);
    assert_eq!(m.image, img, "interpreter and wasm images must match byte for byte");
    // The wrapped 16x16 sprite near the bottom right corner lit pixels in the 128x64 plane.
    assert!(m.lit_pixels() > 0);
}

#[test]
fn schip_clears_on_mode_switch_and_low_res_ignores_dxy0() {
    let src = "
        HIGH
        LD I, spr
        DRW V0, V0, 0
        LOW
        DRW V0, V0, 0
        done:
        JP done
        spr:
        DB 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF
        DB 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF
    ";
    let a = asm::assemble("T", src).unwrap();
    let mut m = Chip8::new(&a.bytes, 1);
    m.run(3);
    assert_eq!(m.lit_pixels(), 256, "16x16 sprite in high-res lights 256 pixels");
    m.run(3);
    assert!(!m.hires());
    assert_eq!(m.lit_pixels(), 0, "LOW clears the screen and Dxy0 draws nothing in low-res");
}

#[test]
fn necrorunner_runs_in_high_res_and_jumps() {
    let (bytes, patches) = rom("NECRORUNNER-1991");
    assert!(patches.get("OBST_SPEED").is_some() && patches.get("JUMP_STEP").is_some() && patches.get("FRAME_DELAY").is_some());
    let mut m = Chip8::new(&bytes, 9);
    m.run(2000);
    assert!(m.hires());
    assert!(m.lit_pixels() > 100, "the ground and the sprites are on the 128x64 screen");
    let mut j = Chip8::new(&bytes, 9);
    j.set_keys(1 << 5);
    let mut min_y = 255u8;
    for _ in 0..1500 {
        j.step();
        min_y = min_y.min(j.v(2));
    }
    assert!(min_y < 40, "holding key 5 lifts the runner above the ground (lowest y reached {min_y})");
    let report = scan(&lift_rom("NECRORUNNER-1991", &bytes).0, vec![]);
    for k in [PathologyKind::Ghost, PathologyKind::Zombie] {
        assert!(report.count(k) >= 1, "{k:?} expected");
    }
    let (st, img) = run_wasm("NECRORUNNER-1991", &bytes, &patches, 9, 3000, 1 << 5);
    let mut r = Chip8::new(&bytes, 9);
    r.set_keys(1 << 5);
    let (rs, _) = r.run(3000);
    assert_eq!(st, rs);
    assert_eq!(r.image, img);
}

#[test]
fn step_batch_matches_step_with_no_keys() {
    for name in ["PONG-STYLE", "INVADERS-STYLE", "BREAKOUT-STYLE", "NECRORUNNER-1991"] {
        let (bytes, patches) = rom(name);
        // Interpreter: 400 steps with keys held at 0 (the all-zero key schedule).
        let mut m = Chip8::new(&bytes, 1);
        m.set_keys(0);
        m.run(400);
        // Module: step_batch(400, 25) with a 16-block schedule of zero masks at KEY_SCHEDULE.
        let text = wat::emit_wat(name, &bytes, &patches);
        let engine = wasmtime::Engine::default();
        let module = wasmtime::Module::new(&engine, &text).expect("valid wat");
        let mut store = wasmtime::Store::new(&engine, ());
        let inst = wasmtime::Instance::new(&mut store, &module, &[]).unwrap();
        let mem = inst.get_memory(&mut store, "mem").unwrap();
        let fresh = Chip8::new(&bytes, 1);
        mem.write(&mut store, 0, &fresh.image).unwrap();
        for i in 0..16 {
            mem.write(&mut store, layout::KEY_SCHEDULE + 4 * i, &0u32.to_le_bytes()).unwrap();
        }
        mem.write(&mut store, layout::KEY_SCHEDULE_BLOCK0, &0u32.to_le_bytes()).unwrap();
        let step_batch = inst.get_typed_func::<(i32, i32), i32>(&mut store, "step_batch").unwrap();
        let _ = step_batch.call(&mut store, (400, 25)).unwrap();
        let mut img = vec![0u8; layout::IMAGE_SIZE];
        mem.read(&store, 0, &mut img).unwrap();
        assert_eq!(m.image, img, "{name}: step_batch image differs from interpreter after 400 steps");
    }
}

#[test]
fn breakout_keeps_playing_after_a_miss() {
    // With no keys the paddle never moves, so the ball is missed. The game must serve
    // again and keep drawing, never freeze on a blank screen.
    let (bytes, _) = rom("BREAKOUT-STYLE");
    let mut m = Chip8::new(&bytes, 3);
    let (mut lit, mut samples, mut ys) = (0, 0, std::collections::BTreeSet::new());
    for _ in 0..400 {
        let (st, _) = m.run(500);
        assert_eq!(st, StepStatus::Ok);
        samples += 1;
        if m.lit_pixels() > 0 { lit += 1; }
        ys.insert(m.v(5));
    }
    // A frame is cleared before it is redrawn, so a few samples land between the two.
    assert!(lit * 10 >= samples * 7, "screen blank in {} of {} samples", samples - lit, samples);
    assert!(ys.len() > 10, "ball stopped moving: y values {ys:?}");
}

#[test]
fn breakout_gravity_patch_is_live() {
    // Without gravity dy is only ever -1 or +1. A live write to GRAVITY must make the
    // ball hover (dy 0) within a few frames, without a reset.
    let (bytes, patches) = rom("BREAKOUT-STYLE");
    let mut m = Chip8::new(&bytes, 4);
    for _ in 0..200 { m.run(100); assert_ne!(m.v(7), 0, "dy was 0 before the patch"); }
    patches.apply(&mut m.image, "GRAVITY", 1).unwrap();
    let mut hovered = false;
    for _ in 0..400 { m.run(50); hovered |= m.v(7) == 0; assert!(matches!(m.v(7), 0 | 1 | 0xFF), "dy left -1..1: {}", m.v(7)); }
    assert!(hovered, "GRAVITY = 1 never changed dy");
}

#[test]
fn breakout_draws_the_bricks_that_are_left() {
    // Clearing the first brick of row 0 must hide only that brick, not the whole row.
    let (bytes, _) = rom("BREAKOUT-STYLE");
    let mut m = Chip8::new(&bytes, 6);
    m.run(2000);
    let (mut seen_second, mut first_lit) = (false, false);
    for _ in 0..200 {
        m.set_v(8, 0xFE);
        m.run(37);
        if m.pixel(9, 3) { seen_second = true; if m.pixel(2, 3) { first_lit = true; } }
    }
    assert!(seen_second, "the second brick of row 0 was never drawn");
    assert!(!first_lit, "the cleared first brick was still drawn");
}
