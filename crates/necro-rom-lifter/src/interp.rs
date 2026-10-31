//! Reference CHIP-8 interpreter over a flat machine image.
use crate::decode::{decode, Op};
use crate::layout::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepStatus {
    Ok = 0,
    WaitingForKey = 1,
    BadPc = 3,
    UnknownOpcode = 5,
    StackFault = 6,
}

impl StepStatus {
    pub fn from_code(c: i32) -> StepStatus {
        match c {
            0 => StepStatus::Ok,
            1 => StepStatus::WaitingForKey,
            5 => StepStatus::UnknownOpcode,
            6 => StepStatus::StackFault,
            _ => StepStatus::BadPc,
        }
    }
}

#[derive(Clone)]
pub struct Chip8 {
    pub image: Vec<u8>,
    pub rom_len: usize,
}

impl Chip8 {
    pub fn new(rom: &[u8], seed: u32) -> Chip8 {
        let mut image = vec![0u8; IMAGE_SIZE];
        image[FONT_BASE..FONT_BASE + 80].copy_from_slice(&FONT);
        image[ROM_BASE..ROM_BASE + rom.len()].copy_from_slice(rom);
        let mut m = Chip8 { image, rom_len: rom.len() };
        m.set32(PC, ROM_BASE as u32);
        m.set32(RNG, if seed == 0 { 0x9E37_79B9 } else { seed });
        m
    }
    fn get32(&self, a: usize) -> u32 {
        u32::from_le_bytes(self.image[a..a + 4].try_into().unwrap())
    }
    fn set32(&mut self, a: usize, v: u32) {
        self.image[a..a + 4].copy_from_slice(&v.to_le_bytes());
    }
    pub fn pc(&self) -> u16 {
        self.get32(PC) as u16
    }
    pub fn v(&self, x: u8) -> u8 {
        self.image[V + x as usize]
    }
    pub fn set_v(&mut self, x: u8, val: u8) {
        self.image[V + x as usize] = val;
    }
    pub fn i(&self) -> u16 {
        self.get32(I) as u16
    }
    pub fn set_keys(&mut self, mask: u16) {
        self.set32(KEYS, mask as u32);
    }
    pub fn keys(&self) -> u16 {
        self.get32(KEYS) as u16
    }
    pub fn hires(&self) -> bool {
        self.image[HIRES] != 0
    }
    fn dims(&self) -> (usize, usize) {
        if self.hires() {
            (128, 64)
        } else {
            (64, 32)
        }
    }
    pub fn display(&self) -> &[u8] {
        let (w, h) = self.dims();
        &self.image[DISPLAY..DISPLAY + w * h / 8]
    }
    pub fn pixel(&self, x: usize, y: usize) -> bool {
        let (w, h) = self.dims();
        let idx = (y % h) * w + (x % w);
        self.image[DISPLAY + idx / 8] & (0x80 >> (idx % 8)) != 0
    }
    pub fn lit_pixels(&self) -> usize {
        self.display().iter().map(|b| b.count_ones() as usize).sum()
    }
    pub fn dt(&self) -> u8 {
        self.image[DT]
    }

    fn clear_display(&mut self) {
        for b in &mut self.image[DISPLAY..DISPLAY + DISPLAY_BYTES] {
            *b = 0;
        }
    }

    fn rnd(&mut self) -> u8 {
        let mut x = self.get32(RNG);
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.set32(RNG, x);
        (x & 0xFF) as u8
    }

    fn tick(&mut self) {
        let d = self.get32(TIMER_DIV) + 1;
        if d >= TIMER_DIVISOR {
            self.set32(TIMER_DIV, 0);
            if self.image[DT] > 0 {
                self.image[DT] -= 1;
            }
            if self.image[ST] > 0 {
                self.image[ST] -= 1;
            }
        } else {
            self.set32(TIMER_DIV, d);
        }
    }

    fn draw(&mut self, x: u8, y: u8, n: u8) {
        let i = self.i() as usize;
        let (w, h) = self.dims();
        // n = 0 is a 16x16 sprite in high-res mode and draws nothing in low-res mode.
        let (rows, cols) = if n == 0 { (if self.hires() { 16 } else { 0 }, 16usize) } else { (n as usize, 8usize) };
        let mut vf = 0u8;
        for r in 0..rows {
            let bits: u16 = if cols == 16 {
                ((self.image[(i + 2 * r) & 0xFFF] as u16) << 8) | self.image[(i + 2 * r + 1) & 0xFFF] as u16
            } else {
                (self.image[(i + r) & 0xFFF] as u16) << 8
            };
            for c in 0..cols {
                if bits & (0x8000 >> c) == 0 {
                    continue;
                }
                let px = (x as usize + c) % w;
                let py = (y as usize + r) % h;
                let idx = py * w + px;
                let a = DISPLAY + idx / 8;
                let mask = 0x80u8 >> (idx % 8);
                if self.image[a] & mask != 0 {
                    vf = 1;
                }
                self.image[a] ^= mask;
            }
        }
        self.set_v(0xF, vf);
    }

    /// Executes one instruction. The status is also what the lifted module returns.
    pub fn step(&mut self) -> StepStatus {
        let pc = self.get32(PC) as usize;
        if pc & 1 == 1 || pc < ROM_BASE || pc + 1 >= ROM_BASE + self.rom_len {
            return StepStatus::BadPc;
        }
        let w = ((self.image[pc] as u16) << 8) | self.image[pc + 1] as u16;
        self.set32(PC, (pc + 2) as u32);
        let op = decode(w);
        match op {
            Op::Cls => self.clear_display(),
            Op::Scd(n) => {
                let (w, h) = self.dims();
                let stride = w / 8;
                let n = (n as usize).min(h);
                for y in (0..h).rev() {
                    for k in 0..stride {
                        self.image[DISPLAY + y * stride + k] = if y >= n { self.image[DISPLAY + (y - n) * stride + k] } else { 0 };
                    }
                }
            }
            Op::Scr => {
                let (w, h) = self.dims();
                let stride = w / 8;
                for y in 0..h {
                    for k in (0..stride).rev() {
                        let hi = if k > 0 { self.image[DISPLAY + y * stride + k - 1] << 4 } else { 0 };
                        self.image[DISPLAY + y * stride + k] = (self.image[DISPLAY + y * stride + k] >> 4) | hi;
                    }
                }
            }
            Op::Scl => {
                let (w, h) = self.dims();
                let stride = w / 8;
                for y in 0..h {
                    for k in 0..stride {
                        let lo = if k + 1 < stride { self.image[DISPLAY + y * stride + k + 1] >> 4 } else { 0 };
                        self.image[DISPLAY + y * stride + k] = (self.image[DISPLAY + y * stride + k] << 4) | lo;
                    }
                }
            }
            Op::Low => {
                self.image[HIRES] = 0;
                self.clear_display();
            }
            Op::High => {
                self.image[HIRES] = 1;
                self.clear_display();
            }
            Op::StRpl(x) => {
                for k in 0..=(x as usize).min(7) {
                    self.image[RPL + k] = self.image[V + k];
                }
            }
            Op::LdRpl(x) => {
                for k in 0..=(x as usize).min(7) {
                    self.image[V + k] = self.image[RPL + k];
                }
            }
            Op::Ret => {
                let sp = self.get32(SP);
                if sp == 0 {
                    self.set32(PC, pc as u32);
                    return StepStatus::StackFault;
                }
                self.set32(SP, sp - 1);
                let r = self.get32(STACK + 4 * (sp as usize - 1));
                self.set32(PC, r);
            }
            Op::Jp(a) => self.set32(PC, a as u32),
            Op::Call(a) => {
                let sp = self.get32(SP);
                if sp >= 16 {
                    self.set32(PC, pc as u32);
                    return StepStatus::StackFault;
                }
                self.set32(STACK + 4 * sp as usize, (pc + 2) as u32);
                self.set32(SP, sp + 1);
                self.set32(PC, a as u32);
            }
            Op::SeKk(x, kk) => {
                if self.v(x) == kk {
                    self.set32(PC, (pc + 4) as u32);
                }
            }
            Op::SneKk(x, kk) => {
                if self.v(x) != kk {
                    self.set32(PC, (pc + 4) as u32);
                }
            }
            Op::SeVy(x, y) => {
                if self.v(x) == self.v(y) {
                    self.set32(PC, (pc + 4) as u32);
                }
            }
            Op::SneVy(x, y) => {
                if self.v(x) != self.v(y) {
                    self.set32(PC, (pc + 4) as u32);
                }
            }
            Op::LdKk(x, kk) => self.set_v(x, kk),
            Op::AddKk(x, kk) => self.set_v(x, self.v(x).wrapping_add(kk)),
            Op::LdVy(x, y) => self.set_v(x, self.v(y)),
            Op::Or(x, y) => self.set_v(x, self.v(x) | self.v(y)),
            Op::And(x, y) => self.set_v(x, self.v(x) & self.v(y)),
            Op::Xor(x, y) => self.set_v(x, self.v(x) ^ self.v(y)),
            Op::AddVy(x, y) => {
                let (r, c) = self.v(x).overflowing_add(self.v(y));
                self.set_v(x, r);
                self.set_v(0xF, c as u8);
            }
            Op::Sub(x, y) => {
                let (a, b) = (self.v(x), self.v(y));
                self.set_v(x, a.wrapping_sub(b));
                self.set_v(0xF, (a >= b) as u8);
            }
            Op::Shr(x) => {
                let a = self.v(x);
                self.set_v(x, a >> 1);
                self.set_v(0xF, a & 1);
            }
            Op::Subn(x, y) => {
                let (a, b) = (self.v(x), self.v(y));
                self.set_v(x, b.wrapping_sub(a));
                self.set_v(0xF, (b >= a) as u8);
            }
            Op::Shl(x) => {
                let a = self.v(x);
                self.set_v(x, a << 1);
                self.set_v(0xF, a >> 7);
            }
            Op::LdI(a) => self.set32(I, a as u32),
            Op::JpV0(a) => self.set32(PC, (a as u32 + self.v(0) as u32) & 0xFFF),
            Op::Rnd(x, kk) => {
                let r = self.rnd();
                self.set_v(x, r & kk);
            }
            Op::Drw(x, y, n) => {
                let (vx, vy) = (self.v(x), self.v(y));
                self.draw(vx, vy, n);
            }
            Op::Skp(x) => {
                if self.keys() & (1 << (self.v(x) & 0xF)) != 0 {
                    self.set32(PC, (pc + 4) as u32);
                }
            }
            Op::Sknp(x) => {
                if self.keys() & (1 << (self.v(x) & 0xF)) == 0 {
                    self.set32(PC, (pc + 4) as u32);
                }
            }
            Op::LdVxDt(x) => self.set_v(x, self.image[DT]),
            Op::LdVxK(x) => {
                let k = self.keys();
                if k == 0 {
                    self.set32(PC, pc as u32);
                    return StepStatus::WaitingForKey;
                }
                self.set_v(x, k.trailing_zeros() as u8);
            }
            Op::LdDtVx(x) => self.image[DT] = self.v(x),
            Op::LdStVx(x) => self.image[ST] = self.v(x),
            Op::AddIVx(x) => self.set32(I, (self.i() as u32 + self.v(x) as u32) & 0xFFF),
            Op::LdFVx(x) => self.set32(I, (FONT_BASE as u32) + 5 * (self.v(x) & 0xF) as u32),
            Op::LdBVx(x) => {
                let v = self.v(x);
                let i = self.i() as usize;
                self.image[i & 0xFFF] = v / 100;
                self.image[(i + 1) & 0xFFF] = (v / 10) % 10;
                self.image[(i + 2) & 0xFFF] = v % 10;
            }
            Op::LdIVx(x) => {
                let i = self.i() as usize;
                for k in 0..=x as usize {
                    self.image[(i + k) & 0xFFF] = self.image[V + k];
                }
            }
            Op::LdVxI(x) => {
                let i = self.i() as usize;
                for k in 0..=x as usize {
                    self.image[V + k] = self.image[(i + k) & 0xFFF];
                }
            }
            Op::Unknown(_) => {
                self.set32(PC, pc as u32);
                return StepStatus::UnknownOpcode;
            }
        }
        self.tick();
        StepStatus::Ok
    }

    /// Runs up to `n` steps, stopping at the first non-Ok status.
    pub fn run(&mut self, n: usize) -> (StepStatus, usize) {
        for k in 0..n {
            let s = self.step();
            if s != StepStatus::Ok {
                return (s, k);
            }
        }
        (StepStatus::Ok, n)
    }
}
