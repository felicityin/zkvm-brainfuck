use std::borrow::Borrow;

use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::FieldAlgebra;
use p3_matrix::Matrix;

use bf_core_executor::Opcode;
use bf_stark::air::BfAirBuilder;

use super::{MemoryInstructionsChip, MemoryInstructionsCols, NUM_MEMORY_INSTRUCTIONS_COLS};
use crate::operations::KoalaBearWordRangeChecker;

impl<F> BaseAir<F> for MemoryInstructionsChip {
    fn width(&self) -> usize {
        NUM_MEMORY_INSTRUCTIONS_COLS
    }
}

impl<AB> Air<AB> for MemoryInstructionsChip
where
    AB: BfAirBuilder,
    AB::Var: Sized,
{
    #[inline(never)]
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let (local, next) = (main.row_slice(0), main.row_slice(1));
        let local: &MemoryInstructionsCols<AB::Var> = (*local).borrow();
        let next: &MemoryInstructionsCols<AB::Var> = (*next).borrow();

        let is_real = local.is_step_forward + local.is_step_backward;
        builder.assert_bool(local.is_step_forward);
        builder.assert_bool(local.is_step_backward);
        builder.assert_bool(is_real.clone());

        builder.when(local.is_step_forward).assert_eq(
            local.next_mp.reduce::<AB>(),
            local.mp.reduce::<AB>() + AB::F::from_canonical_u32(1),
        );

        builder.when(local.is_step_backward).assert_eq(
            local.next_mp.reduce::<AB>(),
            local.mp.reduce::<AB>() - AB::F::from_canonical_u32(1),
        );

        builder
            .when_transition()
            .when(next.is_real)
            .assert_eq(local.next_mp.reduce::<AB>(), next.mp.reduce::<AB>());

        KoalaBearWordRangeChecker::<AB::F>::range_check(
            builder,
            local.mp,
            local.mp_range_checker,
            local.is_real.into(),
        );

        KoalaBearWordRangeChecker::<AB::F>::range_check(
            builder,
            local.next_mp,
            local.next_mp_range_checker,
            local.is_real.into(),
        );

        let opcode = local.is_step_forward * Opcode::MemStepForward.as_field::<AB::F>()
            + local.is_step_backward * Opcode::MemStepBackward.as_field::<AB::F>();

        builder.receive_memory_instr(
            local.clk,
            local.pc,
            opcode,
            local.mp.reduce::<AB>(),
            local.next_mp.reduce::<AB>(),
            is_real,
        );
    }
}
