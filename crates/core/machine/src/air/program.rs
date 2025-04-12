use std::iter::once;

use p3_air::AirBuilder;

use bf_stark::{
    air::{AirLookup, BaseAirBuilder},
    LookupKind,
};

use crate::cpu::cols::InstructionCols;

/// A trait which contains methods related to program lookups in an AIR.
pub trait ProgramAirBuilder: BaseAirBuilder {
    /// Sends an instruction.
    fn send_program(
        &mut self,
        pc: impl Into<Self::Expr>,
        instruction: InstructionCols<impl Into<Self::Expr> + Copy>,
        multiplicity: impl Into<Self::Expr>,
    ) {
        let values = once(pc.into())
            .chain(once(instruction.opcode.into()))
            .chain(instruction.into_iter().map(|x| x.into()))
            .collect();

        self.send(AirLookup::new(values, multiplicity.into(), LookupKind::Program));
    }

    /// Receives an instruction.
    fn receive_program(
        &mut self,
        pc: impl Into<Self::Expr>,
        instruction: InstructionCols<impl Into<Self::Expr> + Copy>,
        multiplicity: impl Into<Self::Expr>,
    ) {
        let values: Vec<<Self as AirBuilder>::Expr> = once(pc.into())
            .chain(once(instruction.opcode.into()))
            .chain(instruction.into_iter().map(|x| x.into()))
            .collect();

        self.receive(AirLookup::new(values, multiplicity.into(), LookupKind::Program));
    }
}
