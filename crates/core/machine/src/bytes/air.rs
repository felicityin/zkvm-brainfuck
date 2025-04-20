use core::borrow::Borrow;

use p3_air::{Air, BaseAir, PairBuilder};
use p3_field::{Field, FieldAlgebra};
use p3_matrix::Matrix;

use bf_core_executor::ByteOpcode;
use bf_stark::air::BfAirBuilder;

use super::{
    cols::{ByteMultCols, BytePreprocessedCols, NUM_BYTE_MULT_COLS},
    ByteChip,
};

impl<F: Field> BaseAir<F> for ByteChip<F> {
    fn width(&self) -> usize {
        NUM_BYTE_MULT_COLS
    }
}

impl<AB: BfAirBuilder + PairBuilder> Air<AB> for ByteChip<AB::F> {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local_mult = main.row_slice(0);
        let local_mult: &ByteMultCols<AB::Var> = (*local_mult).borrow();

        let prep = builder.preprocessed();
        let prep = prep.row_slice(0);
        let local: &BytePreprocessedCols<AB::Var> = (*prep).borrow();

        // Send all the lookups for each operation.
        for (i, opcode) in ByteOpcode::all().iter().enumerate() {
            let field_op = opcode.as_field::<AB::F>();
            let mult = local_mult.multiplicities[i];
            match opcode {
                ByteOpcode::U8Range => {
                    builder.receive_byte(field_op, AB::F::ZERO, local.value_u8, mult)
                }
                ByteOpcode::U16Range => {
                    builder.receive_byte(field_op, local.value_u16, AB::F::ZERO, mult)
                }
            }
        }
    }
}
