use core::borrow::Borrow;
use p3_air::{Air, BaseAir};
use p3_field::FieldAlgebra;
use p3_matrix::Matrix;

use bf_core_executor::{Opcode, DEFAULT_PC_INC};
use bf_stark::air::BfAirBuilder;

use crate::operations::AddOperation;

use super::{AddSubChip, AddSubCols, NUM_ADD_SUB_COLS};

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
            local.pc + AB::Expr::from_canonical_u32(DEFAULT_PC_INC),
            Opcode::Add.as_field::<AB::F>(),
            local.add_operation.value,
            local.mv,
            local.is_add,
        );

        builder.receive_alu(
            local.pc,
            local.pc + AB::Expr::from_canonical_u32(DEFAULT_PC_INC),
            Opcode::Add.as_field::<AB::F>(),
            local.mv,
            local.add_operation.value,
            local.is_sub,
        );
    }
}
