use anyhow::Result;

use bf_prover::{components::DefaultProverComponents, BfProver};

use crate::{BfProofWithPublicValues, BfProvingKey, BfVerifyingKey, Prover};

/// An implementation of [crate::ProverClient] that can generate end-to-end proofs locally.
pub struct CpuProver {
    prover: BfProver<DefaultProverComponents>,
}

impl CpuProver {
    /// Creates a new [LocalProver].
    pub fn new() -> Self {
        let prover = BfProver::new();
        Self { prover }
    }

    /// Creates a new [LocalProver] from an existing [BfProver].
    pub fn from_prover(prover: BfProver<DefaultProverComponents>) -> Self {
        Self { prover }
    }
}

impl Prover<DefaultProverComponents> for CpuProver {
    fn setup(&self, elf: &str) -> (BfProvingKey, BfVerifyingKey) {
        self.prover.setup(elf)
    }

    fn prover(&self) -> &BfProver<DefaultProverComponents> {
        &self.prover
    }

    fn prove(&self, pk: &BfProvingKey, stdin: Vec<u8>) -> Result<BfProofWithPublicValues> {
        let proof: bf_prover::BfProofWithMetadata<bf_prover::BfCoreProofData> =
            self.prover.prove(pk, &stdin)?;
        Ok(BfProofWithPublicValues { proof: proof.proof.0, stdin: proof.stdin })
    }
}

impl Default for CpuProver {
    fn default() -> Self {
        Self::new()
    }
}
