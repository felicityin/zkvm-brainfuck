use std::iter::once;

use p3_air::AirBuilder;
use p3_field::FieldAlgebra;

use bf_core_executor::ByteOpcode;
use bf_stark::air::{BaseAirBuilder, ByteAirBuilder};
use bf_stark::{AirLookup, LookupKind};

use crate::memory::{MemoryAccessCols, MemoryCols};

pub trait MemoryAirBuilder: BaseAirBuilder {
    /// Constrain a memory read or write.
    ///
    /// This method verifies that a memory access timestamp clk is greater than the
    /// previous access's timestamp.  It will also add to the memory argument.
    fn eval_memory_access<E: Into<Self::Expr> + Clone>(
        &mut self,
        clk: impl Into<Self::Expr>,
        addr: impl Into<Self::Expr>,
        memory_access: &impl MemoryCols<E>,
        do_check: impl Into<Self::Expr>,
    ) {
        let do_check: Self::Expr = do_check.into();
        let clk: Self::Expr = clk.into();
        let mem_access = memory_access.access();

        self.assert_bool(do_check.clone());

        // Verify that the current memory access time is greater than the previous's.
        self.eval_memory_access_timestamp(mem_access, do_check.clone(), clk.clone());

        // Add to the memory argument.
        let addr = addr.into();
        let prev_clk = mem_access.prev_clk.clone().into();
        let prev_values = once(prev_clk)
            .chain(once(addr.clone()))
            .chain(once(memory_access.prev_value().clone().into()))
            .collect();
        let current_values: Vec<<Self as AirBuilder>::Expr> = once(clk)
            .chain(once(addr.clone()))
            .chain(once(memory_access.value().clone().into()))
            .collect();

        // The previous values get sent with multiplicity = 1, for "read".
        self.send(AirLookup::new(prev_values, do_check.clone(), LookupKind::Memory));

        // The current values get "received", i.e. multiplicity = -1
        self.receive(AirLookup::new(current_values, do_check, LookupKind::Memory));
    }

    /// Verifies the memory access timestamp.
    ///
    /// This method verifies that the current memory access happened after the previous one's.
    /// Specifically it will ensure that if the current and previous access are in the same shard,
    /// then the current's clk val is greater than the previous's.  If they are not in the same
    /// shard, then it will ensure that the current's shard val is greater than the previous's.
    fn eval_memory_access_timestamp(
        &mut self,
        mem_access: &MemoryAccessCols<impl Into<Self::Expr> + Clone>,
        do_check: impl Into<Self::Expr>,
        clk: impl Into<Self::Expr>,
    ) {
        let do_check: Self::Expr = do_check.into();

        // Get the comparison timestamp values for the current and previous memory access.
        let prev_comp_val = mem_access.prev_clk.clone().into();
        let current_comp_val = clk.into();

        // Assert `current_comp_val > prev_comp_val`. We check this by asserting that
        // `0 <= current_comp_val-prev_comp_val-1 < 2^24`.
        //
        // The equivalence of these statements comes from the fact that if
        // `current_comp_val <= prev_comp_val`, then `current_comp_val-prev_comp_val-1 < 0` and will
        // underflow in the prime field, resulting in a value that is `>= 2^24` as long as both
        // `current_comp_val, prev_comp_val` are range-checked to be `<2^24` and as long as we're
        // working in a field larger than `2 * 2^24` (which is true of the KoalaBear and Mersenne31
        // prime).
        let diff_minus_one = current_comp_val - prev_comp_val - Self::Expr::ONE;

        // Verify that mem_access.ts_diff = mem_access.ts_diff_16bit_limb
        // + mem_access.ts_diff_8bit_limb * 2^16.
        self.eval_range_check_24bits(
            diff_minus_one,
            mem_access.diff_16bit_limb.clone(),
            mem_access.diff_8bit_limb.clone(),
            do_check,
        );
    }

    /// Verifies the inputted value is within 24 bits.
    ///
    /// This method verifies that the inputted is less than 2^24 by doing a 16 bit and 8 bit range
    /// check on it's limbs.  It will also verify that the limbs are correct.  This method is needed
    /// since the memory access timestamp check (see [Self::verify_mem_access_ts]) needs to assume
    /// the clk is within 24 bits.
    fn eval_range_check_24bits(
        &mut self,
        value: impl Into<Self::Expr>,
        limb_16: impl Into<Self::Expr> + Clone,
        limb_8: impl Into<Self::Expr> + Clone,
        do_check: impl Into<Self::Expr> + Clone,
    ) {
        // Verify that value = limb_16 + limb_8 * 2^16.
        self.when(do_check.clone()).assert_eq(
            value,
            limb_16.clone().into()
                + limb_8.clone().into() * Self::Expr::from_canonical_u32(1 << 16),
        );

        // Send the range checks for the limbs.
        self.send_byte(
            Self::Expr::from_canonical_u8(ByteOpcode::U16Range as u8),
            Self::Expr::ZERO,
            limb_16,
            do_check.clone(),
        );

        self.send_byte(
            Self::Expr::from_canonical_u8(ByteOpcode::U8Range as u8),
            limb_8,
            Self::Expr::ZERO,
            do_check,
        )
    }
}
