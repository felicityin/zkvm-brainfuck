use bf_core_executor::events::ByteRecord;
use bf_stark::air::BfAirBuilder;

use p3_air::AirBuilder;
use p3_field::{Field, FieldAlgebra};

use crate::air::U8AirBuilder;

/// A set of columns needed to compute the add of two words.
#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
pub struct AddOperation<T> {
    /// The result of `a + b`.
    pub value: T,

    /// Trace.
    pub carry: T,
}

impl<F: Field> AddOperation<F> {
    pub fn populate(&mut self, record: &mut impl ByteRecord, a_u8: u8, b_u8: u8) -> u8 {
        let expected = a_u8.wrapping_add(b_u8);
        self.value = F::from_canonical_u8(expected);

        if (a_u8 as u32) + (b_u8 as u32) > 255 {
            self.carry = F::ONE;
        }

        let base = 256u32;
        let overflow = a_u8.wrapping_add(b_u8).wrapping_sub(expected) as u32;
        debug_assert_eq!(overflow.wrapping_mul(overflow.wrapping_sub(base)), 0);

        // Range check
        {
            record.add_u8_range_check(a_u8);
            record.add_u8_range_check(b_u8);
            record.add_u8_range_check(expected);
        }
        expected
    }

    pub fn eval<AB: BfAirBuilder>(
        builder: &mut AB,
        a: AB::Var,
        b: AB::Var,
        cols: AddOperation<AB::Var>,
        is_real: AB::Expr,
    ) {
        let one = AB::Expr::ONE;
        let base = AB::F::from_canonical_u32(256);

        let mut builder_is_real = builder.when(is_real.clone());

        // For limb, assert that difference between the carried result and the non-carried
        // result is either zero or the base.
        let overflow = a + b - cols.value;
        builder_is_real.assert_zero(overflow.clone() * (overflow.clone() - base));

        // If the carry is one, then the overflow must be the base.
        builder_is_real.assert_zero(cols.carry * (overflow.clone() - base));

        // If the carry is not one, then the overflow must be zero.
        builder_is_real.assert_zero((cols.carry - one.clone()) * overflow.clone());

        // Assert that the carry is either zero or one.
        builder_is_real.assert_bool(cols.carry);
        builder_is_real.assert_bool(is_real.clone());

        // Range check each byte.
        {
            builder.range_check_u8(a, is_real.clone());
            builder.range_check_u8(b, is_real.clone());
            builder.range_check_u8(cols.value, is_real);
        }
    }
}
