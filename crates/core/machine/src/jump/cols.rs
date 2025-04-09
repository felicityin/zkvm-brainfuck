use bf_derive::AlignedBorrow;
use bf_stark::Word;

use crate::operations::KoalaBearWordRangeChecker;

/// The number of main trace columns for `JumpChip`.
pub const NUM_JUMP_COLS: usize = size_of::<JumpCols<u8>>();

/// The column layout for the chip.
#[derive(AlignedBorrow, Default, Clone, Copy)]
#[repr(C)]
pub struct JumpCols<T> {
    /// The current program counter.
    pub pc: Word<T>,
    pub pc_range_checker: KoalaBearWordRangeChecker<T>,

    /// The next program counter.
    pub next_pc: Word<T>,
    pub next_pc_range_checker: KoalaBearWordRangeChecker<T>,

    /// The destination.
    pub dst: Word<T>,
    /// The memory value, value of the memory pointed at by mp.
    pub mv: T,
    /// Whether the mv is zero.
    pub is_mv_zero: T,

    /// Jump Instructions.
    pub is_loop_start: T,
    pub is_loop_end: T,
}
