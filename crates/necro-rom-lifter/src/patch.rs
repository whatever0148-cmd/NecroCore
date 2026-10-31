use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatchField {
    /// The low byte of an instruction (`kk` in 6xkk, 7xkk, 3xkk, 4xkk, Cxkk).
    Kk,
    /// The low nibble of an instruction (`n` in Dxyn).
    N,
    /// A raw data byte (sprite rows).
    Byte,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchEntry {
    pub name: String,
    /// Address of the instruction or data byte in CHIP-8 memory.
    pub addr: u16,
    pub field: PatchField,
    pub default: u8,
    pub description: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchTable {
    pub rom: String,
    pub entries: Vec<PatchEntry>,
}

impl PatchTable {
    pub fn get(&self, name: &str) -> Option<&PatchEntry> {
        self.entries.iter().find(|e| e.name == name)
    }
    /// True when the instruction at `addr` has a patchable field.
    pub fn field_at(&self, addr: u16) -> Option<PatchField> {
        self.entries.iter().find(|e| e.addr == addr).map(|e| e.field)
    }
    /// Writes `value` into a machine image. Returns the byte address written.
    pub fn apply(&self, image: &mut [u8], name: &str, value: u8) -> Option<usize> {
        let e = self.get(name)?;
        let a = match e.field {
            PatchField::Kk => e.addr as usize + 1,
            PatchField::N => e.addr as usize + 1,
            PatchField::Byte => e.addr as usize,
        };
        match e.field {
            PatchField::N => image[a] = (image[a] & 0xF0) | (value & 0x0F),
            _ => image[a] = value,
        }
        Some(a)
    }
    /// Reads the current value of a patch point from a machine image.
    pub fn read(&self, image: &[u8], name: &str) -> Option<u8> {
        let e = self.get(name)?;
        Some(match e.field {
            PatchField::Kk => image[e.addr as usize + 1],
            PatchField::N => image[e.addr as usize + 1] & 0x0F,
            PatchField::Byte => image[e.addr as usize],
        })
    }
}
