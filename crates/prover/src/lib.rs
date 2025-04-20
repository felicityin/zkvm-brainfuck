//! An end-to-end-prover implementation for the zkVM.

#![allow(clippy::too_many_arguments)]
#![allow(clippy::new_without_default)]
#![allow(clippy::collapsible_else_if)]

pub mod components;
pub mod types;
pub mod verify;

use tracing::instrument;

use bf_core_executor::{ExecutionError, Executor, Program};
use bf_core_machine::{brainfuck::BfAir, utils::BfCoreProverError};
use bf_stark::{koala_bear_poseidon2::KoalaBearPoseidon2, MachineProver};

pub use types::*;

use components::{BfProverComponents, DefaultProverComponents};

/// The configuration for the core prover.
pub type CoreSC = KoalaBearPoseidon2;

/// The configuration for the inner prover.
pub type InnerSC = KoalaBearPoseidon2;

/// A end-to-end prover implementation for the zkVM.
pub struct BfProver<C: BfProverComponents = DefaultProverComponents> {
    /// The machine used for proving the core step.
    pub core_prover: C::CoreProver,
}

impl<C: BfProverComponents> BfProver<C> {
    /// Initializes a new [BfProver].
    #[instrument(name = "initialize prover", level = "debug", skip_all)]
    pub fn new() -> Self {
        // Initialize the provers.
        let core_machine = BfAir::machine(CoreSC::default());
        let core_prover = C::CoreProver::new(core_machine);

        Self { core_prover }
    }

    /// Creates a proving key and a verifying key for a given MIPS ELF.
    #[instrument(name = "setup", level = "debug", skip_all)]
    pub fn setup(&self, elf: &str) -> (BfProvingKey, BfVerifyingKey) {
        let program = Program::from(elf).unwrap();
        let (pk, vk) = self.core_prover.setup(&program);
        let vk = BfVerifyingKey { vk };
        let pk = BfProvingKey {
            pk: self.core_prover.pk_to_host(&pk),
            elf: elf.to_owned(),
            vk: vk.clone(),
        };
        (pk, vk)
    }

    /// Generate a proof of a program with the specified inputs.
    #[instrument(name = "execute", level = "info", skip_all)]
    pub fn execute<'a>(&'a self, elf: &str, input: Vec<u8>) -> Result<Vec<u8>, ExecutionError> {
        let program = Program::from(elf).unwrap();
        let mut runtime = Executor::new(program, input);
        runtime.run()?;
        Ok(runtime.state.output_stream)
    }

    /// Generate shard proofs which split up and prove the valid execution of a MIPS program with
    /// the core prover. Uses the provided context.
    #[instrument(name = "prove", level = "info", skip_all)]
    pub fn prove<'a>(
        &'a self,
        pk: &BfProvingKey,
        stdin: &[u8],
    ) -> Result<BfCoreProof, BfCoreProverError> {
        let program = Program::from(&pk.elf).unwrap();
        let pk = self.core_prover.pk_to_device(&pk.pk);
        let (proof, public_values_stream, cycles) =
            bf_core_machine::utils::prove::<_, C::CoreProver>(
                &self.core_prover,
                &pk,
                program,
                stdin.to_owned(),
            )?;
        Ok(BfCoreProof {
            proof: BfCoreProofData(proof.shard_proof),
            stdin: stdin.to_owned(),
            public_values: public_values_stream,
            cycles,
        })
    }
}

#[cfg(any(test, feature = "export-tests"))]
pub mod tests {
    use super::*;

    use anyhow::Result;

    #[cfg(test)]
    use bf_core_machine::utils::setup_logger;
    #[cfg(test)]
    use serial_test::serial;

    /// Tests an end-to-end workflow of proving a program across the entire proof generation
    /// pipeline.
    #[test]
    #[serial]
    fn test_e2e() -> Result<()> {
        let elf = test_artifacts::FIBO_BF;
        setup_logger();

        let prover = BfProver::<DefaultProverComponents>::new();
        test_e2e_prover::<DefaultProverComponents>(&prover, elf, vec![17], true)
    }

    pub fn test_e2e_prover<C: BfProverComponents>(
        prover: &BfProver<C>,
        elf: &str,
        stdin: Vec<u8>,
        verify: bool,
    ) -> Result<()> {
        tracing::info!("initializing prover");

        tracing::info!("setup elf");
        let (pk, vk) = prover.setup(elf);

        tracing::info!("prove");
        let core_proof = prover.prove(&pk, &stdin)?;

        if verify {
            tracing::info!("verify core");
            prover.verify(&core_proof.proof, &vk)?;
        }

        Ok(())
    }
}
