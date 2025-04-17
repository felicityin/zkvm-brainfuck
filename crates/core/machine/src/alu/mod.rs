use core::borrow::{Borrow, BorrowMut};
use hashbrown::HashMap;
use itertools::Itertools;
use p3_air::{Air, BaseAir};
use p3_field::{PrimeField, PrimeField32};
use p3_matrix::{dense::RowMajorMatrix, Matrix};
use p3_maybe_rayon::prelude::{ParallelBridge, ParallelIterator};

use bf_core_executor::{
    events::{AluEvent, ByteLookupEvent, ByteRecord},
    ExecutionRecord, Opcode, Program,
};
use bf_derive::AlignedBorrow;
use bf_stark::air::{BfAirBuilder, MachineAir};

use crate::operations::AddOperation;
use crate::utils::{next_power_of_two, zeroed_f_vec};

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

    /// The next memory value.
    pub next_mv: T,

    /// The memory value.
    pub mv: T,

    /// Boolean to indicate whether the row is for an add operation.
    pub is_add: T,

    /// Boolean to indicate whether the row is for a sub operation.
    pub is_sub: T,
}


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

    fn included(&self, record: &Self::Record) -> bool {
        !record.add_events.is_empty() || !record.sub_events.is_empty()
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

impl<F> BaseAir<F> for AddSubChip {
    fn width(&self) -> usize {
        NUM_ADD_SUB_COLS
    }
}

impl<AB> Air<AB> for AddSubChip
where
    AB: BfAirBuilder,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.row_slice(0);
        let local: &AddSubCols<AB::Var> = (*local).borrow();

        let is_real = local.is_add + local.is_sub;
        builder.assert_bool(local.is_add);
        builder.assert_bool(local.is_sub);
        builder.assert_bool(is_real);

        // Evaluate the addition operation.
        AddOperation::<AB::F>::eval(
            builder,
            local.next_mv,
            local.mv,
            local.add_operation,
            local.is_add + local.is_sub,
        );

        builder.receive_alu(
            local.pc,
            Opcode::Add.as_field::<AB::F>(),
            local.add_operation.value,
            local.mv,
            local.is_add,
        );

        builder.receive_alu(
            local.pc,
            Opcode::Add.as_field::<AB::F>(),
            local.mv,
            local.add_operation.value,
            local.is_sub,
        );
    }
}

#[derive(Default)]
pub struct AddSubChip;

#[cfg(test)]
mod tests {
    use p3_koala_bear::KoalaBear;
    use p3_matrix::dense::RowMajorMatrix;
    use rand::{rng, Rng};

    use bf_core_executor::{events::AluEvent, ExecutionRecord, Opcode};
    use bf_stark::{
        air::MachineAir, koala_bear_poseidon2::KoalaBearPoseidon2, StarkGenericConfig,
    };

    use crate::utils::{uni_stark_prove as prove, uni_stark_verify as verify};
    use super::AddSubChip;

    #[test]
    fn generate_trace() {
        let mut shard = ExecutionRecord::default();
        shard.add_events = vec![AluEvent::new(0, Opcode::Add, 11, 10)];
        let chip = AddSubChip::default();
        let trace: RowMajorMatrix<KoalaBear> =
            chip.generate_trace(&shard, &mut ExecutionRecord::default());
        println!("{:?}", trace.values)
    }

    #[test]
    fn prove_koala_bear() {
        let config = KoalaBearPoseidon2::new();
        let mut challenger = config.challenger();

        let mut shard = ExecutionRecord::default();
        for i in 0..255 {
            let mv = rng().random_range(0..u8::MAX);
            let mv_next = mv.wrapping_add(1);
            shard.add_events.push(AluEvent::new(
                i << 2,
                Opcode::Add,
                mv_next,
                mv,
            ));
        }
        for i in 0..255 {
            let mv = rng().random_range(0..u8::MAX);
            let mv_next = mv.wrapping_sub(1);
            shard.add_events.push(AluEvent::new(
                i << 2,
                Opcode::Sub,
                mv_next,
                mv,
            ));
        }

        let chip = AddSubChip::default();
        let trace: RowMajorMatrix<KoalaBear> =
            chip.generate_trace(&shard, &mut ExecutionRecord::default());
        let proof = prove::<KoalaBearPoseidon2, _>(&config, &chip, &mut challenger, trace);

        let mut challenger = config.challenger();
        verify(&config, &chip, &mut challenger, &proof).unwrap();
    }
}
