use core::borrow::BorrowMut;
use hashbrown::HashMap;
use itertools::Itertools;
use p3_field::{PrimeField, PrimeField32};
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::prelude::{ParallelBridge, ParallelIterator};

use bf_core_executor::{
    events::{ByteLookupEvent, ByteRecord, JumpEvent},
    ExecutionRecord, Opcode, Program,
};
use bf_stark::air::MachineAir;

use crate::utils::{next_power_of_two, zeroed_f_vec};

use super::{JumpChip, JumpCols, NUM_JUMP_COLS};

impl<F: PrimeField32> MachineAir<F> for JumpChip {
    type Record = ExecutionRecord;

    type Program = Program;

    fn name(&self) -> String {
        "Jump".to_string()
    }

    fn num_rows(&self, input: &Self::Record) -> Option<usize> {
        let nb_rows = next_power_of_two(input.jump_events.len());
        Some(nb_rows)
    }

    fn generate_trace(
        &self,
        input: &ExecutionRecord,
        output: &mut ExecutionRecord,
    ) -> RowMajorMatrix<F> {
        // Generate the rows for the trace.
        let chunk_size = std::cmp::max((input.jump_events.len()) / num_cpus::get(), 1);
        let padded_nb_rows = <JumpChip as MachineAir<F>>::num_rows(self, input).unwrap();
        let mut values = zeroed_f_vec(padded_nb_rows * NUM_JUMP_COLS);

        let blu_events = values
            .chunks_mut(chunk_size * NUM_JUMP_COLS)
            .enumerate()
            .par_bridge()
            .map(|(i, rows)| {
                let mut blu: HashMap<ByteLookupEvent, usize> = HashMap::new();
                rows.chunks_mut(NUM_JUMP_COLS).enumerate().for_each(|(j, row)| {
                    let idx = i * chunk_size + j;
                    let cols: &mut JumpCols<F> = row.borrow_mut();

                    if idx < input.jump_events.len() {
                        let event = &input.jump_events[idx];
                        self.event_to_row(event, cols, &mut blu);
                    }
                });
                blu
            })
            .collect::<Vec<_>>();

        output.add_byte_lookup_events_from_maps(blu_events.iter().collect_vec());

        // Convert the trace to a row major matrix.
        RowMajorMatrix::new(values, NUM_JUMP_COLS)
    }

    fn included(&self, record: &Self::Record) -> bool {
        !record.jump_events.is_empty()
    }

    fn local_only(&self) -> bool {
        true
    }
}

impl JumpChip {
    /// Create a row from an event.
    fn event_to_row<F: PrimeField>(
        &self,
        event: &JumpEvent,
        cols: &mut JumpCols<F>,
        _blu: &mut impl ByteRecord,
    ) {
        cols.pc = event.pc.into();
        cols.pc_range_checker.populate(event.pc);

        cols.next_pc = event.next_pc.into();
        cols.next_pc_range_checker.populate(event.next_pc);

        cols.is_loop_start = F::from_bool(matches!(event.opcode, Opcode::LoopStart));
        cols.is_loop_end = F::from_bool(matches!(event.opcode, Opcode::LoopEnd));

        cols.dst = event.dst.into();
        cols.mv = F::from_canonical_u8(event.mv);
        cols.is_mv_zero.populate_from_field_element(cols.mv);
    }
}
