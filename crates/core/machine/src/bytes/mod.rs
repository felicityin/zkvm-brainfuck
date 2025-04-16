pub mod air;
pub mod cols;
pub mod trace;

use core::borrow::BorrowMut;
use std::marker::PhantomData;

use itertools::Itertools;
use p3_field::Field;
use p3_matrix::dense::RowMajorMatrix;

use bf_core_executor::{events::ByteLookupEvent, ByteOpcode};

use self::cols::{BytePreprocessedCols, NUM_BYTE_PREPROCESSED_COLS};
use crate::{bytes::trace::NUM_ROWS, utils::zeroed_f_vec};

/// The number of different byte operations.
pub const NUM_BYTE_OPS: usize = 10;

/// A chip for computing byte operations.
///
/// The chip contains a preprocessed table of all possible byte operations. Other chips can then
/// use lookups into this table to compute their own operations.
#[derive(Debug, Clone, Copy, Default)]
pub struct ByteChip<F>(PhantomData<F>);

impl<F: Field> ByteChip<F> {
    /// Creates the preprocessed byte trace.
    ///
    /// This function returns a `trace` which is a matrix containing all possible byte operations.
    pub fn trace() -> RowMajorMatrix<F> {
        // The trace containing all values, with all multiplicities set to zero.
        let mut initial_trace = RowMajorMatrix::new(
            zeroed_f_vec(NUM_ROWS * NUM_BYTE_PREPROCESSED_COLS),
            NUM_BYTE_PREPROCESSED_COLS,
        );

        // Record all the necessary operations for each byte lookup.
        let opcodes = ByteOpcode::all();

        // Iterate over all options for pairs of bytes `a` and `b`.
        for (row_index, (b, c)) in (0..=u8::MAX).cartesian_product(0..=u8::MAX).enumerate() {
            let b = b as u8;
            let c = c as u8;
            let col: &mut BytePreprocessedCols<F> = initial_trace.row_mut(row_index).borrow_mut();

            // Set the values of `b` and `c`.
            col.b = F::from_canonical_u8(b);
            col.c = F::from_canonical_u8(c);

            // Iterate over all operations for results and updating the table map.
            for opcode in opcodes.iter() {
                match opcode {
                    ByteOpcode::U8Range => ByteLookupEvent::new(*opcode, 0, 0, b, c),
                    ByteOpcode::U16Range => {
                        let v = ((b as u32) << 8) + c as u32;
                        col.value_u16 = F::from_canonical_u32(v);
                        ByteLookupEvent::new(*opcode, v as u16, 0, 0, 0)
                    }
                };
            }
        }

        initial_trace
    }
}

#[cfg(test)]
mod tests {
    use p3_koala_bear::KoalaBear;
    use std::time::Instant;

    use super::*;

    #[test]
    pub fn test_trace_and_map() {
        let start = Instant::now();
        ByteChip::<KoalaBear>::trace();
        println!("trace and map: {:?}", start.elapsed());
    }
}
