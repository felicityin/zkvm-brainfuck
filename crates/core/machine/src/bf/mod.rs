use core::fmt;
use hashbrown::{HashMap, HashSet};
use itertools::Itertools;
use p3_field::PrimeField32;
use strum_macros::{EnumDiscriminants, EnumIter};

pub use bf_chips::*;
use bf_core_executor::{ExecutionRecord, Program};
// use zkm_core_executor::events::PrecompileEvent;
// use zkm_curves::weierstrass::{bls12_381::Bls12381BaseField, bn254::Bn254BaseField};
use bf_stark::{
    air::MachineAir,
    Chip, LookupKind, StarkGenericConfig, StarkMachine,
};

/// A module for importing all the different MIPS chips.
pub(crate) mod bf_chips {
    pub use crate::{
        alu::AddSubChip,
        bytes::ByteChip,
        jump::JumpChip,
        cpu::CpuChip,
        io::IoChip,
        memory::{MemoryChip, MemoryInstructionsChip},
        program::ProgramChip,
    };
    // pub use bf_curves::{
    //     edwards::{ed25519::Ed25519Parameters, EdwardsCurve},
    //     weierstrass::{
    //         bls12_381::Bls12381Parameters, bn254::Bn254Parameters, secp256k1::Secp256k1Parameters,
    //         secp256r1::Secp256r1Parameters, SwCurve,
    //     },
    // };
}
/// An AIR for encoding MIPS execution.
///
/// This enum contains all the different AIRs that are used in the zkMIPS IOP. Each variant is
/// a different AIR that is used to encode a different part of the zkMIPS execution, and the
/// different AIR variants have a joint lookup argument.
#[derive(bf_derive::MachineAir, EnumDiscriminants)]
#[strum_discriminants(derive(Hash, EnumIter))]
pub enum BfAir<F: PrimeField32> {
    /// An AIR that contains a preprocessed program table and a lookup for the instructions.
    Program(ProgramChip),
    /// An AIR for the CPU. Each row represents a cpu cycle.
    Cpu(CpuChip),
    /// A lookup table for byte operations.
    ByteLookup(ByteChip<F>),
    /// An AIR for the Add and SUB instruction.
    AddSub(AddSubChip),
    /// An AIR for the Memory instructions.
    Memory(MemoryChip),
    /// An AIR for the Jump instructions.
    Jump(JumpChip),
    /// An AIR for memory instructions.
    MemoryInstrs(MemoryInstructionsChip),
    /// An AIR for I/O instructions.
    IO(IoChip),
}

impl<F: PrimeField32> BfAir<F> {
    pub fn machine<SC: StarkGenericConfig<Val = F>>(config: SC) -> StarkMachine<SC, Self> {
        let chips = Self::chips();
        StarkMachine::new(config, chips)
    }

    /// Get all the different AIRs.
    pub fn chips() -> Vec<Chip<F, Self>> {
        let mut chips = vec![];

        let cpu = Chip::new(BfAir::Cpu(CpuChip::default()));
        chips.push(cpu);

        let program = Chip::new(BfAir::Program(ProgramChip::default()));
        chips.push(program);

        let add_sub = Chip::new(BfAir::AddSub(AddSubChip::default()));
        chips.push(add_sub);

        let jump = Chip::new(BfAir::Jump(JumpChip::default()));
        chips.push(jump);

        let memory = Chip::new(BfAir::Memory(MemoryChip::new()));
        chips.push(memory);

        let byte = Chip::new(BfAir::ByteLookup(ByteChip::default()));
        chips.push(byte);

        let memory_instructions =
            Chip::new(BfAir::MemoryInstrs(MemoryInstructionsChip::default()));
        chips.push(memory_instructions);

        chips
    }
}

impl<F: PrimeField32> fmt::Debug for BfAir<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<F: PrimeField32> PartialEq for BfAir<F> {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl<F: PrimeField32> Eq for BfAir<F> {}

impl<F: PrimeField32> core::hash::Hash for BfAir<F> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.name().hash(state);
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
pub mod tests {
    use hashbrown::HashMap;
    use itertools::Itertools;
    use p3_koala_bear::KoalaBear;
    use strum::IntoEnumIterator;

    use bf_core_executor::{
        // programs::tests::{
        //     fibonacci_program, hello_world_program, sha3_chain_program, simple_memory_program,
        //     simple_program, ssz_withdrawals_program,
        // },
        Instruction, Opcode, Program,
    };
    use bf_stark::air::MachineAir;
    use bf_stark::{
        koala_bear_poseidon2::KoalaBearPoseidon2, CpuProver, StarkProvingKey, StarkVerifyingKey,
   };

   use crate::{
        bf::BfsAir,
        utils,
        utils::{prove, run_test, setup_logger},
    };

    #[test]
    fn test_add_prove() {
        setup_logger();
        let instructions = vec![
            Instruction::new(Opcode::Add),
        ];
        let program = Program::new(instructions);
        run_test::<CpuProver<_, _>>(program).unwrap();
    }
}
