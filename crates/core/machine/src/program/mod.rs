use std::collections::HashMap;

use core::{
    borrow::{Borrow, BorrowMut},
    mem::size_of,
};
use p3_air::{Air, BaseAir, PairBuilder};
use p3_field::PrimeField32;
use p3_matrix::{dense::RowMajorMatrix, Matrix};
use p3_maybe_rayon::prelude::{ParallelBridge, ParallelIterator};

use bf_core_executor::{ExecutionRecord, Program};
use bf_derive::AlignedBorrow;
use bf_stark::air::{MachineAir, BfAirBuilder};

use crate::{
    air::ProgramAirBuilder,
    utils::{next_power_of_two, pad_rows_fixed, zeroed_f_vec},
};
use crate::cpu::cols::InstructionCols;

/// The number of preprocessed program columns.
pub const NUM_PROGRAM_PREPROCESSED_COLS: usize = size_of::<ProgramPreprocessedCols<u8>>();

/// The number of columns for the program multiplicities.
pub const NUM_PROGRAM_MULT_COLS: usize = size_of::<ProgramMultiplicityCols<u8>>();

/// The column layout for the chip.
#[derive(AlignedBorrow, Clone, Copy, Default)]
#[repr(C)]
pub struct ProgramPreprocessedCols<T> {
    pub pc: T,
    pub instruction: InstructionCols<T>,
}

/// The column layout for the chip.
#[derive(AlignedBorrow, Clone, Copy, Default)]
#[repr(C)]
pub struct ProgramMultiplicityCols<T> {
    pub multiplicity: T,
}

/// A chip that implements addition for the opcodes ADD and ADDI.
#[derive(Default)]
pub struct ProgramChip;

impl ProgramChip {
    pub const fn new() -> Self {
        Self {}
    }
}

impl<F: PrimeField32> MachineAir<F> for ProgramChip {
    type Record = ExecutionRecord;

    type Program = Program;

    fn name(&self) -> String {
        "Program".to_string()
    }

    fn preprocessed_width(&self) -> usize {
        NUM_PROGRAM_PREPROCESSED_COLS
    }

    fn generate_preprocessed_trace(&self, program: &Self::Program) -> Option<RowMajorMatrix<F>> {
        debug_assert!(!program.instructions.is_empty(), "empty program");
        // Generate the trace rows for each event.
        let nb_rows = program.instructions.len();
        let padded_nb_rows = next_power_of_two(nb_rows);
        let mut values = zeroed_f_vec(padded_nb_rows * NUM_PROGRAM_PREPROCESSED_COLS);
        let chunk_size = std::cmp::max((nb_rows + 1) / num_cpus::get(), 1);

        values
            .chunks_mut(chunk_size * NUM_PROGRAM_PREPROCESSED_COLS)
            .enumerate()
            .par_bridge()
            .for_each(|(i, rows)| {
                rows.chunks_mut(NUM_PROGRAM_PREPROCESSED_COLS).enumerate().for_each(|(j, row)| {
                    let idx = i * chunk_size + j;

                    if idx < nb_rows {
                        let cols: &mut ProgramPreprocessedCols<F> = row.borrow_mut();
                        let instruction = &program.instructions[idx];
                        let pc = idx as u32;
                        cols.pc = F::from_canonical_u32(pc);
                        cols.instruction.populate(instruction);
                    }
                });
            });

        // Convert the trace to a row major matrix.
        Some(RowMajorMatrix::new(values, NUM_PROGRAM_PREPROCESSED_COLS))
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

        // Collect the number of times each instruction is called from the cpu events.
        // Store it as a map of PC -> count.
        let mut instruction_counts = HashMap::new();
        input.cpu_events.iter().for_each(|event| {
            let pc = event.pc;
            instruction_counts.entry(pc).and_modify(|count| *count += 1).or_insert(1);
        });

        let mut rows = input
            .program
            .instructions
            .clone()
            .into_iter()
            .enumerate()
            .map(|(i, _)| {
                let pc = i as u32 * 4;
                let mut row = [F::ZERO; NUM_PROGRAM_MULT_COLS];
                let cols: &mut ProgramMultiplicityCols<F> = row.as_mut_slice().borrow_mut();
                cols.multiplicity =
                    F::from_canonical_usize(*instruction_counts.get(&pc).unwrap_or(&0));
                row
            })
            .collect::<Vec<_>>();

        // Pad the trace to a power of two depending on the proof shape in `input`.
        pad_rows_fixed(&mut rows, || [F::ZERO; NUM_PROGRAM_MULT_COLS]);

        RowMajorMatrix::new(rows.into_iter().flatten().collect::<Vec<_>>(), NUM_PROGRAM_MULT_COLS)
    }

    fn included(&self, _: &Self::Record) -> bool {
        true
    }
}

impl<F> BaseAir<F> for ProgramChip {
    fn width(&self) -> usize {
        NUM_PROGRAM_MULT_COLS
    }
}

impl<AB> Air<AB> for ProgramChip
where
    AB: BfAirBuilder + PairBuilder,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let preprocessed = builder.preprocessed();

        let prep_local = preprocessed.row_slice(0);
        let prep_local: &ProgramPreprocessedCols<AB::Var> = (*prep_local).borrow();
        let mult_local = main.row_slice(0);
        let mult_local: &ProgramMultiplicityCols<AB::Var> = (*mult_local).borrow();

        // Constrain the lookup with CPU table
        builder.receive_program(prep_local.pc, prep_local.instruction, mult_local.multiplicity);
    }
}
