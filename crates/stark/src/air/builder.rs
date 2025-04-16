use std::iter::once;

use p3_air::{AirBuilder, FilteredAirBuilder, PermutationAirBuilder};
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
        a: impl Into<Self::Expr>,
        b: impl Into<Self::Expr>,
        c: impl Into<Self::Expr>,
        multiplicity: impl Into<Self::Expr>,
    ) {
        self.send(
            AirLookup::new(
                vec![opcode.into(), a.into(), b.into(), c.into()],
                multiplicity.into(),
                LookupKind::Byte,
            ),
        );
    }

    /// Receives a byte operation to be processed.
    fn receive_byte(
        &mut self,
        opcode: impl Into<Self::Expr>,
        a: impl Into<Self::Expr>,
        b: impl Into<Self::Expr>,
        c: impl Into<Self::Expr>,
        multiplicity: impl Into<Self::Expr>,
    ) {
        self.receive(
            AirLookup::new(
                vec![opcode.into(), a.into(), b.into(), c.into()],
                multiplicity.into(),
                LookupKind::Byte,
            ),
        );
    }
}

/// A trait which contains methods related to ALU lookups in an AIR.
pub trait InstructionAirBuilder: BaseAirBuilder {
    /// Sends an ALU operation to be processed.
    fn send_alu(
        &mut self,
        pc: impl Into<Self::Expr>,
        opcode: impl Into<Self::Expr>,
        next_mv: impl Into<Self::Expr>,
        mv: impl Into<Self::Expr>,
        multiplicity: impl Into<Self::Expr>,
    ) {
        let values = once(pc.into())
            .chain(once(opcode.into()))
            .chain(once(next_mv.into()))
            .chain(once(mv.into()))
            .collect();

        self.send(
            AirLookup::new(values, multiplicity.into(), LookupKind::Alu),
        );
    }

    /// Receives an ALU operation to be processed.
    fn receive_alu(
        &mut self,
        pc: impl Into<Self::Expr>,
        opcode: impl Into<Self::Expr>,
        next_mv: impl Into<Self::Expr>,
        mv: impl Into<Self::Expr>,
        multiplicity: impl Into<Self::Expr>,
    ) {
        let values = once(pc.into())
            .chain(once(opcode.into()))
            .chain(once(next_mv.into()))
            .chain(once(mv.into()))
            .collect();

        self.receive(
            AirLookup::new(values, multiplicity.into(), LookupKind::Alu),
        );
    }

    /// Sends a Jump operation to be processed.
    fn send_jump(
        &mut self,
        pc: impl Into<Self::Expr>,
        next_pc: impl Into<Self::Expr>,
        opcode: impl Into<Self::Expr>,
        target_pc: impl Into<Self::Expr>,
        mv: impl Into<Self::Expr>,
        multiplicity: impl Into<Self::Expr>,
    ) {
        let values = once(pc.into())
            .chain(once(next_pc.into()))
            .chain(once(opcode.into()))
            .chain(once(target_pc.into()))
            .chain(once(mv.into()))
            .collect();

        self.send(
            AirLookup::new(values, multiplicity.into(), LookupKind::Jump),
        );
    }

    /// Receives a Jump operation to be processed.
    fn receive_jump(
        &mut self,
        pc: impl Into<Self::Expr>,
        next_pc: impl Into<Self::Expr>,
        opcode: impl Into<Self::Expr>,
        target_pc: impl Into<Self::Expr>,
        mv: impl Into<Self::Expr>,
        multiplicity: impl Into<Self::Expr>,
    ) {
        let values = once(pc.into())
            .chain(once(next_pc.into()))
            .chain(once(opcode.into()))
            .chain(once(target_pc.into()))
            .chain(once(mv.into()))
            .collect();

        self.receive(
            AirLookup::new(values, multiplicity.into(), LookupKind::Jump),
        );
    }

    /// Sends a memory pointer operation to be processed.
    fn send_memory_instr(
        &mut self,
        pc: impl Into<Self::Expr>,
        next_pc: impl Into<Self::Expr>,
        opcode: impl Into<Self::Expr>,
        mp: impl Into<Self::Expr>,
        next_mp: impl Into<Self::Expr>,
        multiplicity: impl Into<Self::Expr>,
    ) {
        let values = once(pc.into())
            .chain(once(next_pc.into()))
            .chain(once(opcode.into()))
            .chain(once(mp.into()))
            .chain(once(next_mp.into()))
            .collect();

        self.send(
            AirLookup::new(values, multiplicity.into(), LookupKind::MemInstr),
        );
    }

    /// Receives an ALU operation to be processed.
    fn receive_memory_instr(
        &mut self,
        pc: impl Into<Self::Expr>,
        next_pc: impl Into<Self::Expr>,
        opcode: impl Into<Self::Expr>,
        mp: impl Into<Self::Expr>,
        next_mp: impl Into<Self::Expr>,
        multiplicity: impl Into<Self::Expr>,
    ) {
        let values = once(pc.into())
            .chain(once(next_pc.into()))
            .chain(once(opcode.into()))
            .chain(once(mp.into()))
            .chain(once(next_mp.into()))
            .collect();

        self.receive(
            AirLookup::new(values, multiplicity.into(), LookupKind::MemInstr),
        );
    }

    /// Sends an ALU operation to be processed.
    fn send_io(
        &mut self,
        pc: impl Into<Self::Expr>,
        opcode: impl Into<Self::Expr>,
        mp: impl Into<Self::Expr>,
        mv: impl Into<Self::Expr>,
        multiplicity: impl Into<Self::Expr>,
    ) {
        let values = once(pc.into())
            .chain(once(opcode.into()))
            .chain(once(mp.into()))
            .chain(once(mv.into()))
            .collect();

        self.send(
            AirLookup::new(values, multiplicity.into(), LookupKind::IO),
        );
    }

    /// Receives an ALU operation to be processed.
    fn receive_io(
        &mut self,
        pc: impl Into<Self::Expr>,
        opcode: impl Into<Self::Expr>,
        mp: impl Into<Self::Expr>,
        mv: impl Into<Self::Expr>,
        multiplicity: impl Into<Self::Expr>,
    ) {
        let values = once(pc.into())
            .chain(once(opcode.into()))
            .chain(once(mp.into()))
            .chain(once(mv.into()))
            .collect();

        self.receive(
            AirLookup::new(values, multiplicity.into(), LookupKind::IO),
        );
    }
}

/// A message builder for which sending and receiving messages is a no-op.
pub trait EmptyMessageBuilder: AirBuilder {}

impl<AB: EmptyMessageBuilder, M> MessageBuilder<M> for AB {
    fn send(&mut self, _message: M) {}

    fn receive(&mut self, _message: M) {}
}

/// A builder that implements a permutation argument.
pub trait MultiTableAirBuilder<'a>: PermutationAirBuilder {
    /// The type of the cumulative sum.
    type Sum: Into<Self::ExprEF> + Copy;

    /// Returns the cumulative sum of the permutation.
    fn cumulative_sum(&self) -> &'a Self::Sum;
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
