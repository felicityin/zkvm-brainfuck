use core::borrow::Borrow;
use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::FieldAlgebra;
use p3_matrix::Matrix;

use bf_core_executor::ByteOpcode;
use bf_stark::{
    air::{BaseAirBuilder, BfAirBuilder},
    Word,
};

use crate::{
    air::{U8AirBuilder, MemoryAirBuilder, BfCoreAirBuilder},
    cpu::{
        cols::{CpuCols, NUM_CPU_COLS},
        CpuChip,
    },
};

impl<F> BaseAir<F> for CpuChip {
    fn width(&self) -> usize {
        NUM_CPU_COLS
    }
}

impl<AB> Air<AB> for CpuChip
where
    AB: BfCoreAirBuilder,
    AB::Var: Sized,
{
    #[inline(never)]
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let (local, next) = (main.row_slice(0), main.row_slice(1));
        let local: &CpuCols<AB::Var> = (*local).borrow();
        let next: &CpuCols<AB::Var> = (*next).borrow();

        let clk = AB::Expr::from_canonical_u32(1u32 << 16) * local.clk_8bit_limb + local.clk_16bit_limb;

        // Program constraints.
        builder.send_program(local.pc, local.instruction, local.is_real);

        // Register constraints.
        self.eval_registers::<AB>(builder, local, clk.clone());

        // builder.send_instruction(
        //     local.pc,
        //     local.next_pc,
        //     local.instruction.opcode,
        //     local.mv,
        //     local.next_mv,
        //     local.mp,
        //     local.next_mp,
        //     local.is_mv_immutable,
        //     local.is_real,
        // );

        // Check that the clk is updated correctly.
        self.eval_clk(builder, local, next, clk.clone());

        // Check that the pc is updated correctly.
        builder.when_transition().when(next.is_real).assert_eq(local.next_pc, next.pc);

        // Check that the is_real flag is correct.
        self.eval_is_real(builder, local, next);

        builder.assert_bool(local.is_alu);
        builder.assert_bool(local.is_jump);
        builder.assert_bool(local.is_memory_instr);
        builder.assert_bool(local.is_io);
        builder.assert_bool(local.is_mv_immutable);
    }
}

impl CpuChip {
    pub(crate) fn eval_instruction<AB: BfAirBuilder>(
        &self,
        builder: &mut AB,
        local: &CpuCols<AB::Var>,
        next: &CpuCols<AB::Var>,
        clk: AB::Expr,
    ) {
        // builder.send_alu();
        // builder.send_jump();
        // builder.send_memory();
        // builder.send_io();
    }

    /// Constraints related to the clk.
    ///
    /// This method ensures that the clk starts at 0 and is transitioned appropriately.
    /// It will also check that clk values are within 24 bits.
    /// The range check are needed for the memory access timestamp check, which assumes those values are within 2^24.
    /// See [`MemoryAirBuilder::verify_mem_access_ts`].
    pub(crate) fn eval_clk<AB: BfAirBuilder>(
        &self,
        builder: &mut AB,
        local: &CpuCols<AB::Var>,
        next: &CpuCols<AB::Var>,
        clk: AB::Expr,
    ) {
        // Verify that the first row has a clk value of 0.
        builder.when_first_row().assert_zero(clk.clone());

        let expected_next_clk =
            clk.clone() + AB::Expr::from_canonical_u32(2);

        let next_clk =
            AB::Expr::from_canonical_u32(1u32 << 16) * next.clk_8bit_limb + next.clk_16bit_limb;
        builder.when_transition().when(next.is_real).assert_eq(expected_next_clk, next_clk);

        // Range check that the clk is within 24 bits using it's limb values.
        builder.eval_range_check_24bits(
            clk,
            local.clk_16bit_limb,
            local.clk_8bit_limb,
            local.is_real,
        );
    }

    /// Constraints related to the is_real column.
    ///
    /// This method checks that the is_real column is a boolean. It also checks that the first row
    /// is 1 and once its 0, it never changes value.
    pub(crate) fn eval_is_real<AB: BfAirBuilder>(
        &self,
        builder: &mut AB,
        local: &CpuCols<AB::Var>,
        next: &CpuCols<AB::Var>,
    ) {
        // Check the is_real flag.  It should be 1 for the first row. Once its 0, it should never
        // change value.
        builder.assert_bool(local.is_real);
        builder.when_first_row().assert_one(local.is_real);
        builder.when_transition().when_not(local.is_real).assert_zero(next.is_real);
    }

    /// Computes whether the opcode is a branch instruction.
    pub(crate) fn eval_registers<AB: BfAirBuilder>(
        &self,
        builder: &mut AB,
        local: &CpuCols<AB::Var>,
        clk: AB::Expr,
    ) {
        builder.eval_memory_access(
            clk.clone(),
            local.mv,
            &local.mv_access,
            AB::Expr::ONE - local.is_memory_instr,
        );

        builder.eval_memory_access(
            clk.clone() + AB::F::from_canonical_u32(1),
            local.next_mv,
            &local.next_mv_access,
            local.is_alu,
        );

        // Always range check the value in `mv`, as input instruction `,` may witness
        // an invalid value and write it to memory.
        builder.range_check_u8(local.mv.into(), local.is_real);

        // If we are performing an ALU​​, ​​JMP​​, or ​​OUTPUT instruction, then the value of `mv` is the previous value.
        builder
            .when(local.is_mv_immutable)
            .assert_eq(local.mv_val(), local.mv_access.prev_value);
    }    
}
