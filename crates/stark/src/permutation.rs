use std::borrow::Borrow;

use hashbrown::HashMap;
use itertools::Itertools;
use p3_air::{ExtensionBuilder, PairBuilder};
use p3_field::{ExtensionField, Field, FieldAlgebra, FieldExtensionAlgebra, PrimeField};
use p3_matrix::{dense::RowMajorMatrix, Matrix};
use p3_maybe_rayon::prelude::*;
use rayon_scan::ScanParallelIterator;
use strum::IntoEnumIterator;

use crate::{
    air::MultiTableAirBuilder,
    lookup::Lookup,
};

/// Computes the width of the permutation trace.
#[inline]
#[must_use]
pub const fn permutation_trace_width(num_interactions: usize, batch_size: usize) -> usize {
    if num_interactions == 0 {
        0
    } else {
        num_interactions.div_ceil(batch_size) + 1
    }
}

/// Populates a permutation row.
#[inline]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::needless_pass_by_value)]
pub fn populate_permutation_row<F: PrimeField, EF: ExtensionField<F>>(
    row: &mut [EF],
    preprocessed_row: &[F],
    main_row: &[F],
    sends: &[Lookup<F>],
    receives: &[Lookup<F>],
    random_elements: &[EF],
    batch_size: usize,
) {
    let alpha = random_elements[0];

    // Generate the RLC elements to uniquely identify each item in the looked up tuple.
    let betas = random_elements[1].powers();

    let interaction_chunks = &sends
        .iter()
        .map(|int| (int, true))
        .chain(receives.iter().map(|int| (int, false)))
        .chunks(batch_size);

    // Compute the denominators \prod_{i\in B} row_fingerprint(alpha, beta).
    for (value, chunk) in row.iter_mut().zip(interaction_chunks) {
        *value = chunk
            .into_iter()
            .map(|(interaction, is_send)| {
                let mut denominator = alpha;
                let mut betas = betas.clone();
                denominator +=
                    betas.next().unwrap() * EF::from_canonical_usize(interaction.argument_index());
                for (columns, beta) in interaction.values.iter().zip(betas) {
                    denominator += beta * columns.apply::<F, F>(preprocessed_row, main_row);
                }
                let mut mult = interaction.multiplicity.apply::<F, F>(preprocessed_row, main_row);

                if !is_send {
                    mult = -mult;
                }

                EF::from_base(mult) / denominator
            })
            .sum();
    }
}

/// Generates the permutation trace for the given chip and main trace based on a variant of `LogUp`.
///
/// The permutation trace has `(N+1)*EF::NUM_COLS` columns, where N is the number of interactions in
/// the chip.
pub fn generate_permutation_trace<F: PrimeField, EF: ExtensionField<F>>(
    sends: &[Lookup<F>],
    receives: &[Lookup<F>],
    preprocessed: Option<&RowMajorMatrix<F>>,
    main: &RowMajorMatrix<F>,
    random_elements: &[EF],
    batch_size: usize,
) -> (RowMajorMatrix<EF>, EF) {
    let permutation_trace_width =
        permutation_trace_width(sends.len() + receives.len(), batch_size);

    let height = main.height();
    // let permutation_trace_width = grouped_widths.values().sum::<usize>();
    let mut permutation_trace = RowMajorMatrix::new(
        vec![EF::ZERO; permutation_trace_width * height],
        permutation_trace_width,
    );

    let mut cumulative_sum = EF::ZERO;

    let row_range = 0..permutation_trace_width;

    // Compute the permutation trace values in parallel.
    match preprocessed {
        Some(prep) => {
            permutation_trace
                .par_rows_mut()
                .zip_eq(prep.par_row_slices())
                .zip_eq(main.par_row_slices())
                .for_each(|((row, prep_row), main_row)| {
                    populate_permutation_row(
                        &mut row[row_range.start..row_range.end],
                        prep_row,
                        main_row,
                        sends,
                        receives,
                        random_elements,
                        batch_size,
                    );
                });
        }
        None => {
            permutation_trace.par_rows_mut().zip_eq(main.par_row_slices()).for_each(
                |(row, main_row)| {
                    populate_permutation_row(
                        &mut row[row_range.start..row_range.end],
                        &[],
                        main_row,
                        sends,
                        receives,
                        random_elements,
                        batch_size,
                    );
                },
            );
        }
    }

    let zero = EF::ZERO;
    let cumulative_sums = permutation_trace
        .par_rows_mut()
        .map(|row| row[row_range.start..row_range.end - 1].iter().copied().sum::<EF>())
        .collect::<Vec<_>>();

    let cumulative_sums =
        cumulative_sums.into_par_iter().scan(|a, b| *a + *b, zero).collect::<Vec<_>>();

    cumulative_sum = *cumulative_sums.last().unwrap();

    permutation_trace.par_rows_mut().zip_eq(cumulative_sums.clone().into_par_iter()).for_each(
        |(row, cumulative_sum)| {
            row[row_range.end - 1] = cumulative_sum;
        },
    );

    (permutation_trace, cumulative_sum)
}

/// Evaluates the permutation constraints for the given chip.
///
/// In particular, the constraints checked here are:
///     - The running sum column starts at zero.
///     - That the RLC per interaction is computed correctly.
///     - The running sum column ends at the (currently) given cumalitive sum.
#[allow(clippy::too_many_lines)]
pub fn eval_permutation_constraints<'a, F, AB>(
    sends: &[Lookup<F>],
    receives: &[Lookup<F>],
    batch_size: usize,
    builder: &mut AB,
) where
    F: Field,
    AB::EF: ExtensionField<F>,
    AB: MultiTableAirBuilder<'a, F = F> + PairBuilder,
    AB: 'a,
{
    let permutation_width =
        permutation_trace_width(sends.len() + receives.len(), batch_size);

    // Get the permutation challenges.
    let permutation_challenges = builder.permutation_randomness();
    let random_elements: Vec<AB::ExprEF> =
        permutation_challenges.iter().map(|x| (*x).into()).collect();
    let cumulative_sum: AB::ExprEF = (*builder.cumulative_sum()).into();

    let preprocessed = builder.preprocessed();
    let main = builder.main();
    let perm = builder.permutation().to_row_major_matrix();

    let preprocessed_local = preprocessed.row_slice(0);
    let main_local = main.to_row_major_matrix();
    let main_local = main_local.row_slice(0);
    let main_local: &[AB::Var] = (*main_local).borrow();
    let perm_width = perm.width();
    let perm_local = perm.row_slice(0);
    let perm_local: &[AB::VarEF] = (*perm_local).borrow();
    let perm_next = perm.row_slice(1);
    let perm_next: &[AB::VarEF] = (*perm_next).borrow();

    // Assert that the permutation trace width is correct.
    if perm_width != permutation_width {
        panic!(
            "permutation trace width is incorrect: expected {permutation_width}, got {perm_width}",
        );
    }

    let (alpha, beta) = (&random_elements[0], &random_elements[1]);

    // Ensure that each batch sum m_i/f_i is computed correctly.
    let interaction_chunks = &sends
        .iter()
        .map(|send| (send, true))
        .chain(receives.iter().map(|receive| (receive, false)))
        .chunks(batch_size);

    // Assert that the i-eth entry is equal to the sum_i m_i/rlc_i by constraints:
    // entry * \prod_i rlc_i = \sum_i m_i * \prod_{j!=i} rlc_j over all columns of the permutation
    // trace except the last column.
    for (entry, chunk) in perm_local[0..perm_local.len() - 1].iter().zip(interaction_chunks) {
        // First, we calculate the random linear combinations and multiplicities with the correct
        // sign depending on wetther the interaction is a send or a receive.
        let mut rlcs: Vec<AB::ExprEF> = Vec::with_capacity(batch_size);
        let mut multiplicities: Vec<AB::Expr> = Vec::with_capacity(batch_size);
        for (interaction, is_send) in chunk {
            let mut rlc = alpha.clone();
            let mut betas = beta.powers();

            rlc = rlc.clone()
                + betas.next().unwrap()
                    * AB::ExprEF::from_canonical_usize(interaction.argument_index());
            for (field, beta) in interaction.values.iter().zip(betas.clone()) {
                let elem = field.apply::<AB::Expr, AB::Var>(&preprocessed_local, main_local);
                rlc = rlc.clone() + beta * elem;
            }
            rlcs.push(rlc);

            let send_factor = if is_send { AB::F::ONE } else { -AB::F::ONE };
            multiplicities.push(
                interaction
                    .multiplicity
                    .apply::<AB::Expr, AB::Var>(&preprocessed_local, main_local)
                    * send_factor,
            );
        }

        // Now we can calculate the numerator and denominator of the combined batch.
        let mut product = AB::ExprEF::ONE;
        let mut numerator = AB::ExprEF::ZERO;
        for (i, (m, rlc)) in multiplicities.into_iter().zip(rlcs.iter()).enumerate() {
            // Calculate the running product of all rlcs.
            product = product.clone() * rlc.clone();

            // Calculate the product of all but the current rlc.
            let mut all_but_current = AB::ExprEF::ONE;
            for other_rlc in
                rlcs.iter().enumerate().filter(|(j, _)| i != *j).map(|(_, rlc)| rlc)
            {
                all_but_current = all_but_current.clone() * other_rlc.clone();
            }
            numerator = numerator.clone() + AB::ExprEF::from_base(m) * all_but_current;
        }

        // Finally, assert that the entry is equal to the numerator divided by the product.
        let entry: AB::ExprEF = (*entry).into();
        builder.assert_eq_ext(product.clone() * entry.clone(), numerator);
    }

    // Compute the running local and next permutation sums.
    let sum_local = perm_local[..permutation_width - 1]
        .iter()
        .map(|x| (*x).into())
        .sum::<AB::ExprEF>();
    let sum_next = perm_next[..permutation_width - 1]
        .iter()
        .map(|x| (*x).into())
        .sum::<AB::ExprEF>();
    let phi_local: AB::ExprEF = (*perm_local.last().unwrap()).into();
    let phi_next: AB::ExprEF = (*perm_next.last().unwrap()).into();

    // Assert that cumulative sum is initialized to `phi_local` on the first row.
    builder.when_first_row().assert_eq_ext(phi_local.clone(), sum_local);

    // Assert that the cumulative sum is constrained to `phi_next - phi_local` on the transition
    // rows.
    builder.when_transition().assert_eq_ext(phi_next - phi_local.clone(), sum_next);

    builder.when_last_row().assert_eq_ext(*perm_local.last().unwrap(), cumulative_sum);
}
