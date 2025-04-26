use std::borrow::BorrowMut;

use hashbrown::HashMap;
use itertools::Itertools;
use p3_field::{PrimeField, PrimeField32};
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::prelude::{ParallelBridge, ParallelIterator, ParallelSlice};
use tracing::instrument;

use bf_core_executor::{
    events::{ByteLookupEvent, ByteRecord, CpuEvent, MemoryRecordEnum},
    ExecutionRecord, Instruction, Program,
};
use bf_stark::air::MachineAir;

use super::{cols::NUM_CPU_COLS, CpuChip};
use crate::{cpu::cols::CpuCols, memory::MemoryCols, utils::zeroed_f_vec};

impl<F: PrimeField32> MachineAir<F> for CpuChip {
    type Record = ExecutionRecord;

    type Program = Program;

    fn name(&self) -> String {
        "Cpu".to_string()
    }

    fn generate_trace(
        &self,
        input: &ExecutionRecord,
        _: &mut ExecutionRecord,
    ) -> RowMajorMatrix<F> {
        let padded_nb_rows = input.cpu_events.len().next_power_of_two();
        let mut values = zeroed_f_vec(padded_nb_rows * NUM_CPU_COLS);

        let chunk_size = std::cmp::max(input.cpu_events.len() / num_cpus::get(), 1);
        values.chunks_mut(chunk_size * NUM_CPU_COLS).enumerate().par_bridge().for_each(
            |(i, rows)| {
                rows.chunks_mut(NUM_CPU_COLS).enumerate().for_each(|(j, row)| {
                    let idx = i * chunk_size + j;
                    let cols: &mut CpuCols<F> = row.borrow_mut();

                    if idx < input.cpu_events.len() {
                        let mut byte_lookup_events = Vec::new();
                        let event = &input.cpu_events[idx];
                        let instruction = &input.program.fetch(event.pc);
                        self.event_to_row(event, cols, &mut byte_lookup_events, instruction);
                    }
                });
            },
        );

        // Convert the trace to a row major matrix.
        RowMajorMatrix::new(values, NUM_CPU_COLS)
    }

    #[instrument(name = "generate cpu dependencies", level = "debug", skip_all)]
    fn generate_dependencies(&self, input: &ExecutionRecord, output: &mut ExecutionRecord) {
        // Generate the trace rows for each event.
        let chunk_size = std::cmp::max(input.cpu_events.len() / num_cpus::get(), 1);

        let blu_events: Vec<_> = input
            .cpu_events
            .par_chunks(chunk_size)
            .map(|ops: &[CpuEvent]| {
                // The blu map stores shard -> map(byte lookup event -> multiplicity).
                let mut blu: HashMap<ByteLookupEvent, usize> = HashMap::new();
                ops.iter().for_each(|op| {
                    let mut row = [F::ZERO; NUM_CPU_COLS];
                    let cols: &mut CpuCols<F> = row.as_mut_slice().borrow_mut();
                    let instruction = &input.program.fetch(op.pc);
                    self.event_to_row::<F>(op, cols, &mut blu, instruction);
                });
                blu
            })
            .collect::<Vec<_>>();

        output.add_byte_lookup_events_from_maps(blu_events.iter().collect_vec());
    }

    fn included(&self, record: &Self::Record) -> bool {
        !record.cpu_events.is_empty()
    }
}

impl CpuChip {
    /// Create a row from an event.
    fn event_to_row<F: PrimeField32>(
        &self,
        event: &CpuEvent,
        cols: &mut CpuCols<F>,
        blu_events: &mut impl ByteRecord,
        instruction: &Instruction,
    ) {
        // Populate clk columns.
        self.populate_clk(cols, event, blu_events);

        // Populate basic fields.
        cols.pc = F::from_canonical_u32(event.pc);
        cols.next_pc = F::from_canonical_u32(event.next_pc);
        cols.instruction.populate(instruction);
        cols.mp = F::from_canonical_u32(event.mp);
        cols.next_mp = F::from_canonical_u32(event.next_mp);

        cols.mv = F::from_canonical_u8(event.mv);
        cols.next_mv = F::from_canonical_u8(event.next_mv);
        *cols.mv_access.value_mut() = cols.mv;
        *cols.next_mv_access.value_mut() = cols.next_mv;

        // Populate memory accesses.
        if let Some(record) = event.mv_access {
            cols.mv_access.populate(record, blu_events);
        }

        if let Some(MemoryRecordEnum::Write(record)) = event.next_mv_access {
            cols.next_mv_access.populate(record, blu_events);
        }

        // Populate range checks for mv.
        // blu_events.add_u8_range_check(cols.mv_access.access.value.as_canonical_u32() as u8);

        cols.is_mv_immutable = F::from_bool(instruction.is_mv_immutable());

        cols.is_alu = F::from_bool(instruction.is_alu_instruction());
        cols.is_jump = F::from_bool(instruction.is_jump_instruction());
        cols.is_memory_instr = F::from_bool(instruction.is_memory_instruction());
        cols.is_io = F::from_bool(instruction.is_io_instruction());

        // Assert that the instruction is not a no-op.
        cols.is_real = cols.is_alu + cols.is_jump + cols.is_memory_instr + cols.is_io;
    }

    /// Populates the shard and clk related rows.
    fn populate_clk<F: PrimeField>(
        &self,
        cols: &mut CpuCols<F>,
        event: &CpuEvent,
        _blu_events: &mut impl ByteRecord,
    ) {
        let clk_16bit_limb = (event.clk & 0xffff) as u16;
        let clk_8bit_limb = ((event.clk >> 16) & 0xff) as u8;
        cols.clk_16bit_limb = F::from_canonical_u16(clk_16bit_limb);
        cols.clk_8bit_limb = F::from_canonical_u8(clk_8bit_limb);

        // blu_events.add_u16_range_check(clk_16bit_limb);
        // blu_events.add_u8_range_check(clk_8bit_limb);
    }
}
