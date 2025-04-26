use std::{iter::once, mem::size_of, vec::IntoIter};

use p3_field::PrimeField;

use bf_core_executor::Instruction;
use bf_derive::AlignedBorrow;
use bf_stark::Word;

use crate::memory::{MemoryCols, MemoryReadWriteCols, MemoryWriteCols};

pub const NUM_CPU_COLS: usize = size_of::<CpuCols<u8>>();
pub const NUM_INSTRUCTION_COLS: usize = size_of::<InstructionCols<u8>>();
// pub const CPU_COL_MAP: CpuCols<usize> = make_col_map();

/// The column layout for instructions.
#[derive(AlignedBorrow, Clone, Copy, Default, Debug)]
#[repr(C)]
pub struct InstructionCols<T> {
    /// The opcode for this cycle.
    pub opcode: T,

    /// The first operand for this instruction.
    pub op_a: Word<T>,
}

/// The column layout for the CPU.
#[derive(AlignedBorrow, Default, Debug, Clone, Copy)]
#[repr(C)]
pub struct CpuCols<T: Copy> {
    /// The least significant 16 bit limb of clk.
    pub clk_16bit_limb: T,
    /// The most significant 8 bit limb of clk.
    pub clk_8bit_limb: T,

    /// The program counter value.
    pub pc: T,

    /// The next program counter value.
    pub next_pc: T,

    /// The memory pointer.
    pub mp: T,

    /// The next memory pointer.
    pub next_mp: T,

    /// The memory value.
    pub mv: T,

    /// The next memory value.
    pub next_mv: T,

    /// Columns related to the instruction.
    pub instruction: InstructionCols<T>,

    pub mv_access: MemoryReadWriteCols<T>,
    pub next_mv_access: MemoryWriteCols<T>,

    pub is_mv_immutable: T,

    pub is_alu: T,
    pub is_jump: T,
    pub is_io: T,
    pub is_memory_instr: T,

    /// Selector to label whether this row is a non padded row.
    pub is_real: T,
}

impl<T: Copy> CpuCols<T> {
    /// Gets the value of mv.
    pub fn mv_val(&self) -> T {
        *self.mv_access.value()
    }
}

impl<F: PrimeField> InstructionCols<F> {
    pub fn populate(&mut self, instruction: &Instruction) {
        self.opcode = instruction.opcode.as_field::<F>();
        self.op_a = instruction.op_a.into();
    }
}

impl<T> IntoIterator for InstructionCols<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        once(self.opcode).chain(self.op_a).collect::<Vec<_>>().into_iter()
    }
}
