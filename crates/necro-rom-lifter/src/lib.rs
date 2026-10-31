//! necro-rom-lifter: retro bytecode emulation and lifting for CHIP-8.
//!
//! * [`decode`]: the 35-opcode instruction set.
//! * [`interp`]: a reference interpreter whose whole state is one flat byte image
//!   with the same layout the lifted WebAssembly module uses, so parity is a byte
//!   comparison.
//! * [`asm`]: a two-pass assembler with `.patch NAME` directives that mark
//!   immediates as hot-patchable.
//! * [`patch`]: the hot-patch table. Patching writes bytes into the machine image;
//!   nothing else changes, so a running game keeps its state.
//! * [`lift`]: CHIP-8 -> NecroIR basic blocks for the static scanner.
//! * [`wat`]: CHIP-8 -> WebAssembly text, one compiled case per ROM address,
//!   with patchable immediates read from memory at run time.

pub mod asm;
pub mod decode;
pub mod interp;
pub mod layout;
pub mod lift;
pub mod patch;
pub mod wat;

pub use decode::{decode, Op};
pub use interp::{Chip8, StepStatus};
pub use patch::{PatchEntry, PatchField, PatchTable};
