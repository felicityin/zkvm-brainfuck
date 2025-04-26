use std::{
    borrow::{Borrow, BorrowMut},
    mem::size_of,
};

use p3_air::{Air, BaseAir};
use p3_field::PrimeField32;
use p3_matrix::{dense::RowMajorMatrix, Matrix};
use p3_maybe_rayon::prelude::{ParallelBridge, ParallelIterator};

use bf_core_executor::{ExecutionRecord, Program};
use bf_derive::AlignedBorrow;
use bf_stark::air::{BfAirBuilder, MachineAir};
use bf_stark::{AirLookup, LookupKind};

use crate::utils::{next_power_of_two, zeroed_f_vec};

pub const NUM_MEMORY_ENTRIES_PER_ROW: usize = 2;

pub(crate) const NUM_MEMORY_INIT_COLS: usize = size_of::<MemCols<u8>>();

#[derive(AlignedBorrow, Debug, Clone, Copy)]
#[repr(C)]
struct SingleMemoryLocal<T> {
    /// The address of the memory access.
    pub addr: T,

    /// The initial clk of the memory access.
    pub initial_clk: T,

    /// The final clk of the memory access.
    pub final_clk: T,

    /// The initial value of the memory access.
    pub initial_value: T,

    /// The final value of the memory access.
    pub final_value: T,

    /// Whether the memory access is a real access.
    pub is_real: T,
}

#[derive(AlignedBorrow, Debug, Clone, Copy)]
#[repr(C)]
pub struct MemCols<T> {
    memory_entries: [SingleMemoryLocal<T>; NUM_MEMORY_ENTRIES_PER_ROW],
}

pub struct MemoryChip {}

impl Default for MemoryChip {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryChip {
    /// Creates a new memory chip with a certain type.
    pub const fn new() -> Self {
        Self {}
    }
}

impl<F> BaseAir<F> for MemoryChip {
    fn width(&self) -> usize {
        NUM_MEMORY_INIT_COLS
    }
}

impl<F: PrimeField32> MachineAir<F> for MemoryChip {
    type Record = ExecutionRecord;

    type Program = Program;

    fn name(&self) -> String {
        "Memory".to_string()
    }

    fn generate_dependencies(&self, _input: &ExecutionRecord, _output: &mut ExecutionRecord) {
        // Do nothing since this chip has no dependencies.
    }

    fn generate_trace(
        &self,
        input: &ExecutionRecord,
        _output: &mut ExecutionRecord,
    ) -> RowMajorMatrix<F> {
        // Generate the trace rows for each event.
        let nb_rows = input.cpu_memory_access.len().div_ceil(NUM_MEMORY_ENTRIES_PER_ROW);
        let padded_nb_rows = next_power_of_two(nb_rows);
        let mut values = zeroed_f_vec(padded_nb_rows * NUM_MEMORY_INIT_COLS);
        let chunk_size = std::cmp::max((nb_rows + 1) / num_cpus::get(), 1);

        values.chunks_mut(chunk_size * NUM_MEMORY_INIT_COLS).enumerate().par_bridge().for_each(
            |(i, rows)| {
                rows.chunks_mut(NUM_MEMORY_INIT_COLS).enumerate().for_each(|(j, row)| {
                    let idx = (i * chunk_size + j) * NUM_MEMORY_ENTRIES_PER_ROW;
                    let cols: &mut MemCols<F> = row.borrow_mut();
                    for k in 0..NUM_MEMORY_ENTRIES_PER_ROW {
                        let cols = &mut cols.memory_entries[k];
                        if idx + k < input.cpu_memory_access.len() {
                            let event = &input.cpu_memory_access[idx + k];
                            cols.addr = F::from_canonical_u32(event.addr);
                            cols.initial_clk =
                                F::from_canonical_u32(event.initial_mem_access.timestamp);
                            cols.final_clk =
                                F::from_canonical_u32(event.final_mem_access.timestamp);
                            cols.initial_value =
                                F::from_canonical_u8(event.initial_mem_access.value);
                            cols.final_value = F::from_canonical_u8(event.final_mem_access.value);
                            cols.is_real = F::ONE;
                        }
                    }
                });
            },
        );

        // Convert the trace to a row major matrix.
        RowMajorMatrix::new(values, NUM_MEMORY_INIT_COLS)
    }

    fn included(&self, shard: &Self::Record) -> bool {
        !shard.cpu_memory_access.is_empty()
    }
}

impl<AB> Air<AB> for MemoryChip
where
    AB: BfAirBuilder,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.row_slice(0);
        let local: &MemCols<AB::Var> = (*local).borrow();

        for local in local.memory_entries.iter() {
            let values =
                vec![local.initial_clk.into(), local.addr.into(), local.initial_value.into()];
            builder.receive(AirLookup::new(values, local.is_real.into(), LookupKind::Memory));

            let values = vec![local.final_clk.into(), local.addr.into(), local.final_value.into()];
            builder.send(AirLookup::new(values, local.is_real.into(), LookupKind::Memory));
        }
    }
}
