//! Byte layout of the machine image shared by the interpreter and the WAT module.
pub const RAM: usize = 0x0000; // 4096 bytes
pub const V: usize = 0x1000; // 16 bytes
pub const I: usize = 0x1010; // i32
pub const PC: usize = 0x1014; // i32
pub const SP: usize = 0x1018; // i32
pub const STACK: usize = 0x1020; // 16 x i32
pub const DT: usize = 0x1060; // u8
pub const ST: usize = 0x1061; // u8
pub const KEYS: usize = 0x1064; // i32 bitmask
pub const RNG: usize = 0x1068; // u32 xorshift state
pub const TIMER_DIV: usize = 0x106C; // u32
pub const HIRES: usize = 0x1070; // u8, 1 while SUPER-CHIP high-resolution mode is on
pub const RPL: usize = 0x1078; // 8 bytes, SUPER-CHIP user flags (Fx75 and Fx85)
/// Low-res uses the first 256 bytes (64x32, 8 bytes per row); high-res uses all 1024 (128x64, 16 bytes per row).
pub const DISPLAY: usize = 0x1100;
pub const DISPLAY_BYTES: usize = 1024;
pub const IMAGE_SIZE: usize = 0x1500;
/// `step_batch` reads one u32 key mask per `period` instructions from this table.
/// It sits outside the 0x1500-byte machine image, so parity comparison is unchanged.
pub const KEY_SCHEDULE: usize = 0x2000;
/// u32: the schedule block index `step_batch` starts reading from. The prover sets it
/// so the second per-vector call (after the hot patch) resumes mid-table without
/// rewriting the schedule. Outside the compared machine image.
pub const KEY_SCHEDULE_BLOCK0: usize = 0x1FFC;
pub const ROM_BASE: usize = 0x200;
pub const FONT_BASE: usize = 0x50;
/// Timers decrement once every this many steps.
pub const TIMER_DIVISOR: u32 = 8;

pub const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0,
    0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40,
    0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0,
    0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
];
