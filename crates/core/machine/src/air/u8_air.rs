use p3_field::FieldAlgebra;

use bf_core_executor::ByteOpcode;
use bf_stark::air::ByteAirBuilder;

pub trait U8AirBuilder: ByteAirBuilder {
    /// Check that limb of the given slice is a u8.
    fn range_check_u8(
        &mut self,
        value: impl Into<Self::Expr> + Clone,
        multiplicity: impl Into<Self::Expr> + Clone,
    ) {
        let opcode = Self::Expr::from_canonical_u8(ByteOpcode::U8Range as u8);
        self.send_byte(opcode, Self::Expr::ZERO, value, multiplicity);
    }
}
