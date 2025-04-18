mod air;
pub mod cols;
mod trace;

pub use cols::*;

/// A chip that implements the CPU.
#[derive(Default)]
pub struct CpuChip;
