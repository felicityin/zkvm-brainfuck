use std::iter::once;

use p3_air::{AirBuilder, AirBuilderWithPublicValues, FilteredAirBuilder, PermutationAirBuilder};
use p3_field::{Field, FieldAlgebra};
use p3_uni_stark::{
    ProverConstraintFolder, StarkGenericConfig, SymbolicAirBuilder, VerifierConstraintFolder,
};

use crate::lookup::LookupKind;
use super::lookup::AirLookup;

/// The default increment for the program counter. Is used for all instructions except
/// for jumps.
pub const DEFAULT_PC_INC: u32 = 1;

/// This is used in the `InstrEvent` to indicate that the instruction is not from the CPU.
pub const UNUSED_PC: u32 = 0;

/// A builder that can send and receive messages (or lookups) with other AIRs.
pub trait MessageBuilder<M> {
    /// Sends a message.
    fn send(&mut self, message: M);

    /// Receives a message.
    fn receive(&mut self, message: M);
}

/// A trait which contains basic methods for building an AIR.
pub trait BaseAirBuilder: AirBuilder + MessageBuilder<AirLookup<Self::Expr>> {
    /// Returns a sub-builder whose constraints are enforced only when `condition` is not one.
    fn when_not<I: Into<Self::Expr>>(&mut self, condition: I) -> FilteredAirBuilder<Self> {
        self.when_ne(condition, Self::F::ONE)
    }

    /// Will return `a` if `condition` is 1, else `b`.  This assumes that `condition` is already
    /// checked to be a boolean.
    #[inline]
    fn if_else(
        &mut self,
        condition: impl Into<Self::Expr> + Clone,
        a: impl Into<Self::Expr> + Clone,
        b: impl Into<Self::Expr> + Clone,
    ) -> Self::Expr {
        condition.clone().into() * a.into() + (Self::Expr::ONE - condition.into()) * b.into()
    }
}

/// A trait which contains methods for byte lookups in an AIR.
pub trait ByteAirBuilder: BaseAirBuilder {
    /// Sends a byte operation to be processed.
    fn send_byte(
        &mut self,
        opcode: impl Into<Self::Expr>,
        value: impl Into<Self::Expr>,
        multiplicity: impl Into<Self::Expr>,
    ) {
        self.send(
            AirLookup::new(
                vec![opcode.into(), value.into()],
                multiplicity.into(),
                LookupKind::Byte,
            ),
        );
    }

    /// Receives a byte operation to be processed.
    fn receive_byte(
        &mut self,
        opcode: impl Into<Self::Expr>,
        value: impl Into<Self::Expr>,
        multiplicity: impl Into<Self::Expr>,
    ) {
        self.receive(
            AirLookup::new(
                vec![opcode.into(), value.into()],
                multiplicity.into(),
                LookupKind::Byte,
            ),
        );
    }
}

/// A trait which contains methods related to ALU lookups in an AIR.
pub trait InstructionAirBuilder: BaseAirBuilder {
    /// Sends an instruction to be processed.
    #[allow(clippy::too_many_arguments)]
    fn send_instruction(
        &mut self,
        clk: impl Into<Self::Expr> + Clone,
        pc: impl Into<Self::Expr>,
        next_pc: impl Into<Self::Expr>,
        opcode: impl Into<Self::Expr>,
        a: impl Into<Self::Expr>,
        b: impl Into<Self::Expr>,
        is_memory: impl Into<Self::Expr>,
        multiplicity: impl Into<Self::Expr>,
    ) {
        let values = once(clk.into())
            .chain(once(pc.into()))
            .chain(once(next_pc.into()))
            .chain(once(opcode.into()))
            .chain(once(a.into()))
            .chain(once(b.into()))
            .chain(once(is_memory.into()))
            .collect();

        self.send(
            AirLookup::new(values, multiplicity.into(), LookupKind::Instruction),
        );
    }

    /// Receives an ALU operation to be processed.
    #[allow(clippy::too_many_arguments)]
    fn receive_instruction(
        &mut self,
        clk: impl Into<Self::Expr> + Clone,
        pc: impl Into<Self::Expr>,
        next_pc: impl Into<Self::Expr>,
        opcode: impl Into<Self::Expr>,
        a: impl Into<Self::Expr>,
        b: impl Into<Self::Expr>,
        is_memory: impl Into<Self::Expr>,
        multiplicity: impl Into<Self::Expr>,
    ) {
        let values = once(clk.into())
            .chain(once(pc.into()))
            .chain(once(next_pc.into()))
            .chain(once(opcode.into()))
            .chain(once(a.into()))
            .chain(once(b.into()))
            .chain(once(is_memory.into()))
            .collect();

        self.receive(
            AirLookup::new(values, multiplicity.into(), LookupKind::Instruction),
        );
    }

    /// Sends an ALU operation to be processed.
    fn send_alu(
        &mut self,
        opcode: impl Into<Self::Expr>,
        a: impl Into<Self::Expr>,
        b: impl Into<Self::Expr>,
        multiplicity: impl Into<Self::Expr>,
    ) {
        self.send_instruction(
            Self::Expr::ZERO,
            Self::Expr::from_canonical_u32(UNUSED_PC),
            Self::Expr::from_canonical_u32(UNUSED_PC + DEFAULT_PC_INC),
            opcode,
            a,
            b,
            Self::Expr::ZERO,
            multiplicity,
        )
    }

    /// Receives an ALU operation to be processed.
    fn receive_alu(
        &mut self,
        opcode: impl Into<Self::Expr>,
        a: impl Into<Self::Expr>,
        b: impl Into<Self::Expr>,
        multiplicity: impl Into<Self::Expr>,
    ) {
        self.receive_instruction(
            Self::Expr::ZERO,
            Self::Expr::from_canonical_u32(UNUSED_PC),
            Self::Expr::from_canonical_u32(UNUSED_PC + DEFAULT_PC_INC),
            opcode,
            a,
            b,
            Self::Expr::ZERO,
            multiplicity,
        )
    }
}

/// A message builder for which sending and receiving messages is a no-op.
pub trait EmptyMessageBuilder: AirBuilder {}

impl<AB: EmptyMessageBuilder, M> MessageBuilder<M> for AB {
    fn send(&mut self, _message: M) {}

    fn receive(&mut self, _message: M) {}
}

/// A trait that contains the common helper methods for building machine AIRs.
pub trait MachineAirBuilder: BaseAirBuilder {}

/// A trait which contains all helper methods for building machine AIRs.
pub trait BfAirBuilder: MachineAirBuilder + ByteAirBuilder + InstructionAirBuilder {}

impl<AB: AirBuilder + MessageBuilder<M>, M> MessageBuilder<M> for FilteredAirBuilder<'_, AB> {
    fn send(&mut self, message: M) {
        self.inner.send(message);
    }

    fn receive(&mut self, message: M) {
        self.inner.receive(message);
    }
}

impl<AB: AirBuilder + MessageBuilder<AirLookup<AB::Expr>>> BaseAirBuilder for AB {}
impl<AB: BaseAirBuilder> ByteAirBuilder for AB {}
impl<AB: BaseAirBuilder> InstructionAirBuilder for AB {}

impl<AB: BaseAirBuilder> MachineAirBuilder for AB {}
impl<AB: BaseAirBuilder> BfAirBuilder for AB {}

impl<SC: StarkGenericConfig> EmptyMessageBuilder for ProverConstraintFolder<'_, SC> {}
impl<SC: StarkGenericConfig> EmptyMessageBuilder for VerifierConstraintFolder<'_, SC> {}
impl<F: Field> EmptyMessageBuilder for SymbolicAirBuilder<F> {}

#[cfg(debug_assertions)]
#[cfg(not(doctest))]
impl<F: Field> EmptyMessageBuilder for p3_uni_stark::DebugConstraintBuilder<'_, F> {}
