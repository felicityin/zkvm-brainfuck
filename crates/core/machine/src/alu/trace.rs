use core::borrow::BorrowMut;
use hashbrown::HashMap;
use itertools::Itertools;
use p3_field::{PrimeField, PrimeField32};
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::prelude::{ParallelBridge, ParallelIterator};

use bf_core_executor::{
    events::{AluEvent, ByteLookupEvent, ByteRecord},
    ExecutionRecord, Opcode, Program,
};
use bf_stark::air::MachineAir;

use crate::utils::{next_power_of_two, zeroed_f_vec};

use super::{AddSubChip, AddSubCols,  NUM_ADD_SUB_COLS};

impl<F: PrimeField32> MachineAir<F> for AddSubChip {
    type Record = ExecutionRecord;

    type Program = Program;

    fn name(&self) -> String {
        "AddSub".to_string()
    }

    fn num_rows(&self, input: &Self::Record) -> Option<usize> {
        let nb_rows = next_power_of_two(
            input.add_events.len() + input.sub_events.len(),
        );
        Some(nb_rows)
    }

    fn generate_trace(
        &self,
        input: &ExecutionRecord,
        _: &mut ExecutionRecord,
    ) -> RowMajorMatrix<F> {
        // Generate the rows for the trace.
        let chunk_size =
            std::cmp::max((input.add_events.len() + input.sub_events.len()) / num_cpus::get(), 1);
        let merged_events =
            input.add_events.iter().chain(input.sub_events.iter()).collect::<Vec<_>>();
        let padded_nb_rows = <AddSubChip as MachineAir<F>>::num_rows(self, input).unwrap();
        let mut values = zeroed_f_vec(padded_nb_rows * NUM_ADD_SUB_COLS);

        values.chunks_mut(chunk_size * NUM_ADD_SUB_COLS).enumerate().par_bridge().for_each(
            |(i, rows)| {
                rows.chunks_mut(NUM_ADD_SUB_COLS).enumerate().for_each(|(j, row)| {
                    let idx = i * chunk_size + j;
                    let cols: &mut AddSubCols<F> = row.borrow_mut();

                    if idx < merged_events.len() {
                        let mut byte_lookup_events = Vec::new();
                        let event = &merged_events[idx];
                        self.event_to_row(event, cols, &mut byte_lookup_events);
                    }
                });
            },
        );

        // Convert the trace to a row major matrix.
        RowMajorMatrix::new(values, NUM_ADD_SUB_COLS)
    }

    fn generate_dependencies(&self, input: &Self::Record, output: &mut Self::Record) {
        let chunk_size =
            std::cmp::max((input.add_events.len() + input.sub_events.len()) / num_cpus::get(), 1);

        let event_iter =
            input.add_events.chunks(chunk_size).chain(input.sub_events.chunks(chunk_size));

        let blu_batches = event_iter
            .par_bridge()
            .map(|events| {
                let mut blu: HashMap<ByteLookupEvent, usize> = HashMap::new();
                events.iter().for_each(|event| {
                    let mut row = [F::ZERO; NUM_ADD_SUB_COLS];
                    let cols: &mut AddSubCols<F> = row.as_mut_slice().borrow_mut();
                    self.event_to_row(event, cols, &mut blu);
                });
                blu
            })
            .collect::<Vec<_>>();

        output.add_byte_lookup_events_from_maps(blu_batches.iter().collect_vec());
    }

    fn included(&self, shard: &Self::Record) -> bool {
        !shard.add_events.is_empty() || !shard.sub_events.is_empty()
    }

    fn local_only(&self) -> bool {
        true
    }
}

impl AddSubChip {
    /// Create a row from an event.
    fn event_to_row<F: PrimeField>(
        &self,
        event: &AluEvent,
        cols: &mut AddSubCols<F>,
        blu: &mut impl ByteRecord,
    ) {
        cols.pc = F::from_canonical_u32(event.pc);

        cols.is_add = F::from_bool(matches!(event.opcode, Opcode::Add));
        cols.is_sub = F::from_bool(matches!(event.opcode, Opcode::Sub));

        let is_add = event.opcode == Opcode::Add;
        let operand_1 = if is_add { event.mv_next } else { event.mv };
        let operand_2 = 1;

        cols.add_operation.populate(blu, operand_1, operand_2);
        cols.next_mv = F::from_canonical_u8(operand_1);
        cols.mv = F::from_canonical_u8(operand_2);
    }
}
