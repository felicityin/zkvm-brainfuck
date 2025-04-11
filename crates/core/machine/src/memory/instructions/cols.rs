use std::mem::size_of;

use bf_derive::AlignedBorrow;
use bf_stark::Word;

use crate::operations::KoalaBearWordRangeChecker;

pub const NUM_MEMORY_INSTRUCTIONS_COLS: usize = size_of::<MemoryInstructionsCols<u8>>();

/// The column layout for memory.
#[derive(AlignedBorrow, Default, Debug, Clone, Copy)]
#[repr(C)]
pub struct MemoryInstructionsCols<T> {
    /// The program counter of the instruction.
    pub pc: T,

    /// The clock cycle number.
    pub clk: T,

    /// The memory pointer.
    pub mp: Word<T>,
    pub mp_range_checker: KoalaBearWordRangeChecker<T>,

    /// The next memory pointer.
    pub next_mp: Word<T>,
    pub next_mp_range_checker: KoalaBearWordRangeChecker<T>,

    /// Whether this is `>`.
    pub is_step_forward: T,
    /// Whether this is `<`.
    pub is_step_backward: T,

    /// Selector to label whether this row is a non padded row.
    pub is_real: T,
}
