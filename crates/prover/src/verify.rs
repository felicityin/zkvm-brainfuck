use anyhow::Result;

use bf_core_machine::cpu::MAX_CPU_LOG_DEGREE;
use bf_stark::{MachineProof, MachineProver, MachineVerificationError, StarkGenericConfig};

use crate::{components::BfProverComponents, BfCoreProofData, BfProver, BfVerifyingKey, CoreSC};

impl<C: BfProverComponents> BfProver<C> {
    /// Verify a core proof by verifying the shard, verifying lookup bus.
    pub fn verify(
        &self,
        proof: &BfCoreProofData,
        vk: &BfVerifyingKey,
    ) -> Result<(), MachineVerificationError<CoreSC>> {
        let shard = &proof.0;
        if !shard.contains_cpu() {
            return Err(MachineVerificationError::MissingCpuInFirstShard);
        }

        // CPU log degree bound constraints.
        //
        // Assert that the CPU log degree does not exceed `MAX_CPU_LOG_DEGREE`. This is to ensure
        // that the lookup argument's multiplicities do not overflow.
        let shard_proof = &proof.0;
        let log_degree_cpu = shard_proof.log_degree_cpu();
        if log_degree_cpu > MAX_CPU_LOG_DEGREE {
            return Err(MachineVerificationError::CpuLogDegreeTooLarge(log_degree_cpu));
        }

        // Verify the shard proof.
        let mut challenger = self.core_prover.config().challenger();
        let machine_proof = MachineProof { shard_proof: proof.0.clone() };
        self.core_prover.machine().verify(&vk.vk, &machine_proof, &mut challenger)?;

        Ok(())
    }
}
