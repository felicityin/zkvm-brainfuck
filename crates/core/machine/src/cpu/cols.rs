use std::mem::{size_of, transmute};

use p3_field::PrimeField;
use p3_util::indices_arr;

use bf_core_executor::Instruction;
use bf_derive::AlignedBorrow;
use bf_stark::Word;

use crate::memory::{MemoryCols, MemoryReadCols, MemoryReadWriteCols};

pub const NUM_CPU_COLS: usize = size_of::<CpuCols<u8>>();
pub const NUM_INSTRUCTION_COLS: usize = size_of::<InstructionCols<u8>>();
pub const CPU_COL_MAP: CpuCols<usize> = make_col_map();

/// The column layout for the CPU.
#[derive(AlignedBorrow, Default, Debug, Clone, Copy)]
#[repr(C)]
pub struct CpuCols<T: Copy> {
    /// The least significant 16 bit limb of clk.
    pub clk_16bit_limb: T,
    /// The most significant 8 bit limb of clk.
    pub clk_8bit_limb: T,

    /// The clk to send to the opcode specific tables.  This should be 0 for all instructions other
    /// than the syscall and memory instructions.
    pub clk_to_send: T,

    /// The program counter value.
    pub pc: T,

    /// The expected next program counter value.
    pub next_pc: T,

    /// Columns related to the instruction.
    pub instruction: InstructionCols<T>,

    /// Whether this is a memory instruction.
    pub is_memory: T,

    /// Operand values, either from registers or immediate values.
    pub op_a_value: Word<T>,
    pub op_a_access: MemoryReadWriteCols<T>,
    pub op_b_access: MemoryReadCols<T>,

    /// Selector to label whether this row is a non padded row.
    pub is_real: T,
}

impl<T: Copy> CpuCols<T> {
    /// Gets the value of the first operand.
    pub fn op_a_val(&self) -> Word<T> {
        *self.op_a_access.value()
    }

    /// Gets the value of the second operand.
    pub fn op_b_val(&self) -> Word<T> {
        *self.op_b_access.value()
    }

    /// Gets the value of the third operand.
    pub fn op_c_val(&self) -> Word<T> {
        *self.op_c_access.value()
    }
}

/// Creates the column map for the CPU.
const fn make_col_map() -> CpuCols<usize> {
    let indices_arr = indices_arr::<NUM_CPU_COLS>();
    unsafe { transmute::<[usize; NUM_CPU_COLS], CpuCols<usize>>(indices_arr) }
}

/// The column layout for instructions.
#[derive(AlignedBorrow, Clone, Copy, Default, Debug)]
#[repr(C)]
pub struct InstructionCols<T> {
    /// The opcode for this cycle.
    pub opcode: T,

    /// The first operand for this instruction.
    pub op_a: Word<T>,

    /// The second operand for this instruction.
    pub op_b: Word<T>,

    /// The third operand for this instruction.
    pub op_c: Word<T>,
}

impl<F: PrimeField> InstructionCols<F> {
    pub fn populate(&mut self, instruction: &Instruction) {
        self.opcode = instruction.opcode.as_field::<F>();
        self.op_a = (instruction.op_a as u32).into();
        self.op_b = instruction.op_b.into();
        self.op_c = instruction.op_c.into();
    }
}

impl<T> IntoIterator for InstructionCols<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        once(self.opcode)
            .chain(self.op_a)
            .chain(self.op_b)
            .chain(self.op_c)
            .collect::<Vec<_>>()
            .into_iter()
    }
}
