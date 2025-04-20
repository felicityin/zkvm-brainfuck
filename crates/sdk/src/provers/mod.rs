mod cpu;

pub use cpu::CpuProver;

use anyhow::Result;
use thiserror::Error;

use bf_prover::{
    components::BfProverComponents, BfCoreProofData, BfProver, BfProvingKey, BfVerifyingKey, CoreSC,
};
use bf_stark::MachineVerificationError;

use crate::BfProofWithPublicValues;

#[derive(Error, Debug)]
pub enum BfVerificationError {
    #[error("Invalid public values")]
    InvalidPublicValues,
    #[error("Core machine verification error: {0}")]
    Core(MachineVerificationError<CoreSC>),
}

/// An implementation of [crate::ProverClient].
pub trait Prover<C: BfProverComponents>: Send + Sync {
    fn prover(&self) -> &BfProver<C>;

    fn setup(&self, elf: &str) -> (BfProvingKey, BfVerifyingKey);

    /// Prove the execution of a ELF with the given inputs.
    fn prove(&self, pk: &BfProvingKey, stdin: Vec<u8>) -> Result<BfProofWithPublicValues>;

    /// Verify that a proof is valid given its vkey and metadata.
    fn verify(
        &self,
        bundle: &BfProofWithPublicValues,
        vkey: &BfVerifyingKey,
    ) -> Result<(), BfVerificationError> {
        self.prover()
            .verify(&BfCoreProofData(bundle.proof.clone()), vkey)
            .map_err(BfVerificationError::Core)
    }
}
