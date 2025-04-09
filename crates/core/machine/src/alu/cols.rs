use bf_derive::AlignedBorrow;

use crate::operations::AddOperation;

/// The number of main trace columns for `AddSubChip`.
pub const NUM_ADD_SUB_COLS: usize = size_of::<AddSubCols<u8>>();

/// The column layout for the chip.
#[derive(AlignedBorrow, Default, Clone, Copy)]
#[repr(C)]
pub struct AddSubCols<T> {
    /// The program counter.
    pub pc: T,

    /// Instance of `AddOperation` to handle addition logic in `AddSubChip`'s ALU operations.
    /// It's result will be `mv_next` for the add operation and `mv` for the sub operation.
    pub add_operation: AddOperation<T>,

    /// The first input operand.
    pub mv_next: T,

    /// The second input operand.
    pub mv: T,

    /// Boolean to indicate whether the row is for an add operation.
    pub is_add: T,

    /// Boolean to indicate whether the row is for a sub operation.
    pub is_sub: T,
}
