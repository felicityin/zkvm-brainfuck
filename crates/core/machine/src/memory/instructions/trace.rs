use std::borrow::BorrowMut;

use hashbrown::HashMap;
use itertools::Itertools;
use p3_field::PrimeField32;
use p3_matrix::dense::RowMajorMatrix;
use rayon::iter::{ParallelBridge, ParallelIterator};

use bf_core_executor::{
    events::{ByteLookupEvent, ByteRecord, MemInstrEvent},
    ExecutionRecord, Opcode, Program,
};
use bf_stark::air::MachineAir;

use super::{
    cols::{MemoryInstructionsCols, NUM_MEMORY_INSTRUCTIONS_COLS},
    MemoryInstructionsChip,
};
use crate::utils::{next_power_of_two, zeroed_f_vec};

impl<F: PrimeField32> MachineAir<F> for MemoryInstructionsChip {
    type Record = ExecutionRecord;

    type Program = Program;

    fn name(&self) -> String {
        "MemoryInstrs".to_string()
    }

    fn num_rows(&self, input: &Self::Record) -> Option<usize> {
        let nb_rows = next_power_of_two(input.memory_instr_events.len());
        Some(nb_rows)
    }

    fn generate_trace(
        &self,
        input: &ExecutionRecord,
        output: &mut ExecutionRecord,
    ) -> RowMajorMatrix<F> {
        let chunk_size = std::cmp::max((input.memory_instr_events.len()) / num_cpus::get(), 1);
        let padded_nb_rows =
            <MemoryInstructionsChip as MachineAir<F>>::num_rows(self, input).unwrap();
        let mut values = zeroed_f_vec(padded_nb_rows * NUM_MEMORY_INSTRUCTIONS_COLS);

        let blu_events: Vec<HashMap<ByteLookupEvent, usize>> = values
            .chunks_mut(chunk_size * NUM_MEMORY_INSTRUCTIONS_COLS)
            .enumerate()
            .par_bridge()
            .map(|(i, rows)| {
                let mut blu: HashMap<ByteLookupEvent, usize> = HashMap::new();
                rows.chunks_mut(NUM_MEMORY_INSTRUCTIONS_COLS).enumerate().for_each(|(j, row)| {
                    let idx = i * chunk_size + j;
                    let cols: &mut MemoryInstructionsCols<F> = row.borrow_mut();

                    if idx < input.memory_instr_events.len() {
                        let event = &input.memory_instr_events[idx];
                        self.event_to_row(event, cols, &mut blu);
                    }
                });
                blu
            })
            .collect::<Vec<_>>();

        output.add_byte_lookup_events_from_maps(blu_events.iter().collect_vec());

        // Convert the trace to a row major matrix.
        RowMajorMatrix::new(values, NUM_MEMORY_INSTRUCTIONS_COLS)
    }

    fn included(&self, record: &Self::Record) -> bool {
        !record.memory_instr_events.is_empty()
    }

    fn local_only(&self) -> bool {
        false
    }
}

impl MemoryInstructionsChip {
    fn event_to_row<F: PrimeField32>(
        &self,
        event: &MemInstrEvent,
        cols: &mut MemoryInstructionsCols<F>,
        _blu: &mut HashMap<ByteLookupEvent, usize>,
    ) {
        cols.clk = F::from_canonical_u32(event.clk);
        cols.pc = F::from_canonical_u32(event.pc);
        cols.mp = event.mp.into();
        cols.mp_range_checker.populate(event.mp);
        cols.next_mp = event.next_mp.into();
        cols.next_mp_range_checker.populate(event.next_mp);
        cols.is_step_forward = F::from_bool(matches!(event.opcode, Opcode::MemStepForward));
        cols.is_step_backward = F::from_bool(matches!(event.opcode, Opcode::MemStepBackward));
        // Assert that the instruction is not a no-op.
        cols.is_real = F::ONE;
    }
}
