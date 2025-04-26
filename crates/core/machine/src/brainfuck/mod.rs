use core::fmt;
use p3_field::PrimeField32;
use strum_macros::{EnumDiscriminants, EnumIter};

pub use bf_chips::*;
use bf_stark::{air::MachineAir, Chip, StarkGenericConfig, StarkMachine};

/// A module for importing all the different MIPS chips.
pub(crate) mod bf_chips {
    pub use crate::{
        alu::AddSubChip,
        bytes::ByteChip,
        cpu::CpuChip,
        io::IoChip,
        jump::JumpChip,
        memory::{MemoryChip, MemoryInstructionsChip},
        program::ProgramChip,
    };
}
/// An AIR for encoding execution.
///
/// This enum contains all the different AIRs that are used in the zkMIPS IOP. Each variant is
/// a different AIR that is used to encode a different part of the zkMIPS execution, and the
/// different AIR variants have a joint lookup argument.
#[derive(bf_derive::MachineAir, EnumDiscriminants)]
#[strum_discriminants(derive(Hash, EnumIter))]
pub enum BfAir<F: PrimeField32> {
    /// An AIR that contains a preprocessed program table and a lookup for the instructions.
    Program(ProgramChip),
    /// An AIR for the Memory.
    Memory(MemoryChip),
    /// An AIR for the CPU. Each row represents a cpu cycle.
    Cpu(CpuChip),
    /// A lookup table for byte operations.
    ByteLookup(ByteChip<F>),
    /// An AIR for the Add and Sub instruction.
    AddSub(AddSubChip),
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

        let cpu = Chip::new(BfAir::Cpu(CpuChip));
        chips.push(cpu);

        let program = Chip::new(BfAir::Program(ProgramChip));
        chips.push(program);

        let add_sub = Chip::new(BfAir::AddSub(AddSubChip));
        chips.push(add_sub);

        let jump = Chip::new(BfAir::Jump(JumpChip));
        chips.push(jump);

        let memory = Chip::new(BfAir::Memory(MemoryChip::new()));
        chips.push(memory);

        let byte = Chip::new(BfAir::ByteLookup(ByteChip::default()));
        chips.push(byte);

        let memory_instructions = Chip::new(BfAir::MemoryInstrs(MemoryInstructionsChip));
        chips.push(memory_instructions);

        let io = Chip::new(BfAir::IO(IoChip));
        chips.push(io);

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
    use bf_core_executor::{Instruction, Opcode, Program};
    use bf_stark::CpuProver;
    use test_artifacts::{FIBO_BF, HELLO_BF, LOOP_BF, MOVE_BF, PRINTA_BF};

    use crate::utils::{run_test, setup_logger};

    #[test]
    fn test_instructions_prove() {
        setup_logger();
        let instructions = vec![
            Instruction::new(Opcode::Add),
            Instruction::new(Opcode::Sub),
            Instruction::new(Opcode::MemStepForward),
            Instruction::new(Opcode::MemStepBackward),
            Instruction::new(Opcode::Input),
            Instruction::new(Opcode::Output),
        ];
        let program = Program::new(instructions);
        run_test::<CpuProver<_, _>>(program, vec![1]).unwrap();
    }

    #[test]
    fn test_add_sub_prove() {
        setup_logger();
        let program = Program::from("++-").unwrap();
        run_test::<CpuProver<_, _>>(program, vec![]).unwrap();
    }

    #[test]
    fn test_mem_prove() {
        setup_logger();
        let program = Program::from(">><").unwrap();
        run_test::<CpuProver<_, _>>(program, vec![]).unwrap();
    }

    #[test]
    fn test_jmp_prove() {
        setup_logger();
        let program = Program::from("[----]").unwrap();
        run_test::<CpuProver<_, _>>(program, vec![]).unwrap();
    }

    #[test]
    fn test_io_prove() {
        setup_logger();
        let program = Program::from(",.").unwrap();
        run_test::<CpuProver<_, _>>(program, vec![1]).unwrap();
    }

    #[test]
    fn test_printa_prove() {
        setup_logger();
        let program = Program::from(PRINTA_BF).unwrap();
        run_test::<CpuProver<_, _>>(program, vec![]).unwrap();
    }

    #[test]
    fn test_move_prove() {
        setup_logger();
        let program = Program::from(MOVE_BF).unwrap();
        run_test::<CpuProver<_, _>>(program, vec![]).unwrap();
    }

    #[test]
    fn test_loop_prove() {
        setup_logger();
        let program = Program::from(LOOP_BF).unwrap();
        run_test::<CpuProver<_, _>>(program, vec![]).unwrap();
    }

    #[test]
    fn test_hello_prove() {
        setup_logger();
        let program = Program::from(HELLO_BF).unwrap();
        run_test::<CpuProver<_, _>>(program, vec![]).unwrap();
    }

    #[test]
    fn test_fibo_prove() {
        setup_logger();
        let program = Program::from(FIBO_BF).unwrap();
        run_test::<CpuProver<_, _>>(program, vec![17]).unwrap();
    }
}
