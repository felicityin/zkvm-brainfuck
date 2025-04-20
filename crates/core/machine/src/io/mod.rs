use std::{
    borrow::{Borrow, BorrowMut},
    mem::size_of,
};

use p3_air::{Air, BaseAir};
use p3_field::{PrimeField, PrimeField32};
use p3_matrix::{dense::RowMajorMatrix, Matrix};
use p3_maybe_rayon::prelude::{ParallelBridge, ParallelIterator};

use bf_core_executor::{events::IoEvent, ExecutionRecord, Opcode, Program};
use bf_derive::AlignedBorrow;
use bf_stark::air::{BfAirBuilder, MachineAir};

use crate::utils::{next_power_of_two, zeroed_f_vec};

pub(crate) const NUM_IO_COLS: usize = size_of::<IoCols<u8>>();

#[derive(AlignedBorrow, Debug, Clone, Copy)]
#[repr(C)]
struct IoCols<T> {
    /// The address of the memory access.
    pub pc: T,

    /// The memory pointer.
    pub mp: T,

    /// The memory value.
    pub mv: T,

    /// Boolean to indicate whether the row is for a input operation.
    pub is_input: T,

    /// Boolean to indicate whether the row is for an output operation.
    pub is_output: T,
}

pub struct IoChip;

impl Default for IoChip {
    fn default() -> Self {
        Self::new()
    }
}

impl IoChip {
    /// Creates a new memory chip with a certain type.
    pub const fn new() -> Self {
        Self {}
    }
}

impl<F> BaseAir<F> for IoChip {
    fn width(&self) -> usize {
        NUM_IO_COLS
    }
}

impl<F: PrimeField32> MachineAir<F> for IoChip {
    type Record = ExecutionRecord;

    type Program = Program;

    fn name(&self) -> String {
        "IO".to_string()
    }

    fn generate_dependencies(&self, _input: &ExecutionRecord, _output: &mut ExecutionRecord) {
        // Do nothing since this chip has no dependencies.
    }

    fn generate_trace(
        &self,
        input: &ExecutionRecord,
        _output: &mut ExecutionRecord,
    ) -> RowMajorMatrix<F> {
        // Generate the rows for the trace.
        let chunk_size = std::cmp::max((input.io_events.len()) / num_cpus::get(), 1);
        let padded_nb_rows = next_power_of_two(input.io_events.len());
        let mut values = zeroed_f_vec(padded_nb_rows * NUM_IO_COLS);

        values
            .chunks_mut(chunk_size * NUM_IO_COLS)
            .enumerate()
            .par_bridge()
            .map(|(i, rows)| {
                rows.chunks_mut(NUM_IO_COLS).enumerate().for_each(|(j, row)| {
                    let idx = i * chunk_size + j;
                    let cols: &mut IoCols<F> = row.borrow_mut();

                    if idx < input.io_events.len() {
                        let event = &input.io_events[idx];
                        self.event_to_row(event, cols);
                    }
                });
            })
            .collect::<Vec<_>>();

        // Convert the trace to a row major matrix.
        RowMajorMatrix::new(values, NUM_IO_COLS)
    }

    fn included(&self, record: &Self::Record) -> bool {
        !record.io_events.is_empty()
    }

    fn local_only(&self) -> bool {
        true
    }
}

impl IoChip {
    /// Create a row from an event.
    fn event_to_row<F: PrimeField>(&self, event: &IoEvent, cols: &mut IoCols<F>) {
        cols.pc = F::from_canonical_u32(event.pc);
        cols.mp = F::from_canonical_u32(event.mp);
        cols.mv = F::from_canonical_u8(event.mv);
        cols.is_input = F::from_bool(matches!(event.opcode, Opcode::Input));
        cols.is_output = F::from_bool(matches!(event.opcode, Opcode::Output));
    }
}

impl<AB> Air<AB> for IoChip
where
    AB: BfAirBuilder,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.row_slice(0);
        let local: &IoCols<AB::Var> = (*local).borrow();

        let is_real = local.is_input + local.is_output;
        builder.assert_bool(local.is_input);
        builder.assert_bool(local.is_output);
        builder.assert_bool(is_real.clone());

        let opcode = local.is_input * Opcode::Input.as_field::<AB::F>()
            + local.is_output * Opcode::Output.as_field::<AB::F>();

        builder.receive_io(local.pc, opcode, local.mp, local.mv, is_real);
    }
}
