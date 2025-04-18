use core::borrow::Borrow;
use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::FieldAlgebra;
use p3_matrix::Matrix;

use bf_core_executor::{Opcode, DEFAULT_PC_INC};
use bf_stark::air::{BaseAirBuilder, BfAirBuilder};

use super::{JumpChip, JumpCols, NUM_JUMP_COLS};
use crate::operations::{IsZeroOperation, KoalaBearWordRangeChecker};

impl<F> BaseAir<F> for JumpChip {
    fn width(&self) -> usize {
        NUM_JUMP_COLS
    }
}

impl<AB> Air<AB> for JumpChip
where
    AB: BfAirBuilder,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.row_slice(0);
        let local: &JumpCols<AB::Var> = (*local).borrow();

        let is_real = local.is_loop_start + local.is_loop_end;
        builder.assert_bool(local.is_loop_start);
        builder.assert_bool(local.is_loop_end);
        builder.assert_bool(is_real.clone());

        IsZeroOperation::<AB::F>::eval(builder, local.mv.into(), local.is_mv_zero, is_real.clone());

        // [: jump if mv = 0
        builder
            .when(local.is_loop_start)
            .when(local.is_mv_zero.result)
            .assert_eq(local.next_pc.reduce::<AB>(), local.dst.reduce::<AB>());

        // [: skip if mv != 0
        builder.when(local.is_loop_start).when_not(local.is_mv_zero.result).assert_eq(
            local.next_pc.reduce::<AB>(),
            local.pc.reduce::<AB>() + AB::F::from_canonical_u32(DEFAULT_PC_INC),
        );

        // ]: jump if mv != 0
        builder
            .when(local.is_loop_end)
            .when_not(local.is_mv_zero.result)
            .assert_eq(local.next_pc.reduce::<AB>(), local.dst.reduce::<AB>());

        // ]: skip if mv = 0
        builder.when(local.is_loop_end).when(local.is_mv_zero.result).assert_eq(
            local.next_pc.reduce::<AB>(),
            local.pc.reduce::<AB>() + AB::F::from_canonical_u32(DEFAULT_PC_INC),
        );

        KoalaBearWordRangeChecker::<AB::F>::range_check(
            builder,
            local.pc,
            local.pc_range_checker,
            is_real.clone(),
        );

        KoalaBearWordRangeChecker::<AB::F>::range_check(
            builder,
            local.next_pc,
            local.next_pc_range_checker,
            is_real.clone(),
        );

        let opcode = local.is_loop_start * Opcode::LoopStart.as_field::<AB::F>()
            + local.is_loop_end * Opcode::LoopEnd.as_field::<AB::F>();

        builder.receive_jump(
            local.pc.reduce::<AB>(),
            local.next_pc.reduce::<AB>(),
            opcode,
            local.dst.reduce::<AB>(),
            local.mv,
            is_real,
        );
    }
}
