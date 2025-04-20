mod air;
pub mod cols;
mod trace;

pub use cols::*;

/// The maximum log degree of the CPU chip to avoid lookup multiplicity overflow.
pub const MAX_CPU_LOG_DEGREE: usize = 22;

/// A chip that implements the CPU.
#[derive(Default)]
pub struct CpuChip;
