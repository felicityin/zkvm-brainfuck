use std::hash::Hash;

use p3_air::{Air, BaseAir, PairBuilder};
use p3_field::{ExtensionField, Field, PrimeField, PrimeField32};
use p3_matrix::dense::RowMajorMatrix;
use p3_uni_stark::{get_max_constraint_degree, SymbolicAirBuilder};
use p3_util::log2_ceil_usize;

use super::{
    eval_permutation_constraints, generate_permutation_trace, permutation_trace_width,
    PROOF_MAX_NUM_PVS,
};
use crate::{
    air::{BfAirBuilder, MachineAir, MultiTableAirBuilder},
    lookup::{Lookup, LookupBuilder, LookupKind},
};

/// An Air that encodes lookups based on lookups.
pub struct Chip<F: Field, A> {
    /// The underlying AIR of the chip for constraint evaluation.
    air: A,
    /// The lookups that the chip sends.
    sends: Vec<Lookup<F>>,
    /// The lookups that the chip receives.
    receives: Vec<Lookup<F>>,
    /// The relative log degree of the quotient polynomial, i.e. `log2(max_constraint_degree - 1)`.
    log_quotient_degree: usize,
}

impl<F: Field, A> Chip<F, A> {
    /// The send lookups of the chip.
    pub fn sends(&self) -> &[Lookup<F>] {
        &self.sends
    }

    /// The receive lookups of the chip.
    pub fn receives(&self) -> &[Lookup<F>] {
        &self.receives
    }

    /// The relative log degree of the quotient polynomial, i.e. `log2(max_constraint_degree - 1)`.
    pub const fn log_quotient_degree(&self) -> usize {
        self.log_quotient_degree
    }

    /// Consumes the chip and returns the underlying air.
    pub fn into_inner(self) -> A {
        self.air
    }
}

impl<F: PrimeField32, A: MachineAir<F>> Chip<F, A> {
    /// Returns whether the given chip is included in the execution record of the shard.
    pub fn included(&self, record: &A::Record) -> bool {
        self.air.included(record)
    }
}

impl<F, A> Chip<F, A>
where
    F: Field,
    A: BaseAir<F>,
{
    /// Records the lookups and constraint degree from the air and crates a new chip.
    pub fn new(air: A) -> Self
    where
        A: MachineAir<F> + Air<LookupBuilder<F>> + Air<SymbolicAirBuilder<F>>,
    {
        let mut builder = LookupBuilder::new(air.preprocessed_width(), air.width());
        air.eval(&mut builder);
        let (sends, receives) = builder.lookups();

        let nb_byte_sends = sends.iter().filter(|s| s.kind == LookupKind::Byte).count();
        let nb_byte_receives = receives.iter().filter(|r| r.kind == LookupKind::Byte).count();
        tracing::debug!(
            "chip {} has {} byte lookups",
            air.name(),
            nb_byte_sends + nb_byte_receives
        );

        let mut max_constraint_degree =
            get_max_constraint_degree(&air, air.preprocessed_width(), PROOF_MAX_NUM_PVS);

        if !sends.is_empty() || !receives.is_empty() {
            max_constraint_degree = max_constraint_degree.max(3);
        }
        let log_quotient_degree = log2_ceil_usize(max_constraint_degree - 1);

        Self { air, sends, receives, log_quotient_degree }
    }

    /// Returns the number of lookups in the chip.
    #[inline]
    pub fn num_lookups(&self) -> usize {
        self.sends.len() + self.receives.len()
    }

    /// Returns the number of sent byte lookups in the chip.
    #[inline]
    pub fn num_sent_byte_lookups(&self) -> usize {
        self.sends.iter().filter(|i| i.kind == LookupKind::Byte).count()
    }

    /// Returns the number of sends of the given kind.
    #[inline]
    pub fn num_sends_by_kind(&self, kind: LookupKind) -> usize {
        self.sends.iter().filter(|i| i.kind == kind).count()
    }

    /// Returns the number of receives of the given kind.
    #[inline]
    pub fn num_receives_by_kind(&self, kind: LookupKind) -> usize {
        self.receives.iter().filter(|i| i.kind == kind).count()
    }

    /// Generates a permutation trace for the given matrix.
    pub fn generate_permutation_trace<EF: ExtensionField<F>>(
        &self,
        preprocessed: Option<&RowMajorMatrix<F>>,
        main: &RowMajorMatrix<F>,
        random_elements: &[EF],
    ) -> (RowMajorMatrix<EF>, EF)
    where
        F: PrimeField,
        A: MachineAir<F>,
    {
        let batch_size = self.logup_batch_size();
        generate_permutation_trace(
            &self.sends,
            &self.receives,
            preprocessed,
            main,
            random_elements,
            batch_size,
        )
    }

    /// Returns the width of the permutation trace.
    #[inline]
    pub fn permutation_width(&self) -> usize {
        permutation_trace_width(self.sends().len() + self.receives().len(), self.logup_batch_size())
    }

    /// Returns the cost of a row in the chip.
    #[inline]
    pub fn cost(&self) -> u64 {
        (self.width() + 4 * self.permutation_width()) as u64
    }

    /// Returns the width of the quotient polynomial.
    #[inline]
    pub const fn quotient_width(&self) -> usize {
        1 << self.log_quotient_degree
    }

    /// Returns the log2 of the batch size.
    #[inline]
    pub const fn logup_batch_size(&self) -> usize {
        1 << self.log_quotient_degree
    }
}

impl<F, A> BaseAir<F> for Chip<F, A>
where
    F: Field,
    A: BaseAir<F>,
{
    fn width(&self) -> usize {
        self.air.width()
    }

    fn preprocessed_trace(&self) -> Option<RowMajorMatrix<F>> {
        panic!("Chip should not use the `BaseAir` method, but the `MachineAir` method.")
    }
}

impl<F, A> MachineAir<F> for Chip<F, A>
where
    F: Field,
    A: MachineAir<F>,
{
    type Record = A::Record;

    type Program = A::Program;

    fn name(&self) -> String {
        self.air.name()
    }

    fn preprocessed_width(&self) -> usize {
        <A as MachineAir<F>>::preprocessed_width(&self.air)
    }

    fn generate_preprocessed_trace(&self, program: &A::Program) -> Option<RowMajorMatrix<F>> {
        <A as MachineAir<F>>::generate_preprocessed_trace(&self.air, program)
    }

    fn generate_trace(&self, input: &A::Record, output: &mut A::Record) -> RowMajorMatrix<F> {
        self.air.generate_trace(input, output)
    }

    fn generate_dependencies(&self, input: &A::Record, output: &mut A::Record) {
        self.air.generate_dependencies(input, output);
    }

    fn included(&self, shard: &Self::Record) -> bool {
        self.air.included(shard)
    }

    fn local_only(&self) -> bool {
        self.air.local_only()
    }
}

// Implement AIR directly on Chip, evaluating both execution and permutation constraints.
impl<'a, F, A, AB> Air<AB> for Chip<F, A>
where
    F: Field,
    A: Air<AB>,
    AB: BfAirBuilder<F = F> + MultiTableAirBuilder<'a> + PairBuilder + 'a,
{
    fn eval(&self, builder: &mut AB) {
        // Evaluate the execution trace constraints.
        self.air.eval(builder);
        // Evaluate permutation constraints.
        let batch_size = self.logup_batch_size();
        eval_permutation_constraints(&self.sends, &self.receives, batch_size, builder);
    }
}

impl<F, A> PartialEq for Chip<F, A>
where
    F: Field,
    A: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.air == other.air
    }
}

impl<F: Field, A: Eq> Eq for Chip<F, A> where F: Field + Eq {}

impl<F, A> Hash for Chip<F, A>
where
    F: Field,
    A: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.air.hash(state);
    }
}
