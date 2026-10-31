//! necro-core: the unified intermediate representation (`NecroIR`), control-flow
//! graph construction, and the static pathology scanner.
//!
//! Three pathologies are detected on the CFG, with their UI labels:
//! * unreachable code (label "Ghost"),
//! * unbounded accumulators or buffer growth inside a loop (label "Vampire"),
//! * loops that cannot terminate because no exit condition depends on anything
//!   the loop writes (label "Zombie").
//!
//! Every finding carries byte and line offsets into the original source plus the
//! CFG edges involved, and the scanner emits an EARS requirements document that a
//! Kiro spec can consume directly.

pub mod cfg;
pub mod heal;
pub mod ir;
pub mod report;
pub mod scan;

pub use cfg::Cfg;
pub use heal::heal;
pub use ir::*;
pub use report::{NecroAutopsyReport, Pathology, PathologyKind};
pub use scan::scan;
