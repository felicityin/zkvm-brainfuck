use bf_core_machine::brainfuck::BfAir;
use bf_stark::koala_bear_poseidon2::KoalaBearPoseidon2;
use bf_stark::{CpuProver, MachineProver, StarkGenericConfig};

/// The configuration for the core prover.
pub type CoreSC = KoalaBearPoseidon2;

/// The configuration for the inner prover.
pub type InnerSC = KoalaBearPoseidon2;

pub trait BfProverComponents: Send + Sync {
    /// The prover for making core proofs.
    type CoreProver: MachineProver<CoreSC, BfAir<<CoreSC as StarkGenericConfig>::Val>> + Send + Sync;
}

pub struct DefaultProverComponents;

impl BfProverComponents for DefaultProverComponents {
    type CoreProver = CpuProver<CoreSC, BfAir<<CoreSC as StarkGenericConfig>::Val>>;
}
