//! A library for interacting with the zkVM.

pub mod action;

pub mod proof;
pub mod provers;

use bf_prover::components::DefaultProverComponents;
pub use proof::*;
pub use provers::BfVerificationError;

pub use provers::{CpuProver, Prover};

// Re-export the utilities.
pub use bf_core_machine::utils;
pub use bf_core_machine::utils::setup_logger;
pub use bf_prover::{BfProver, BfProvingKey, BfVerifyingKey, CoreSC, InnerSC};

/// A client for interacting with zkMIPS.
pub struct ProverClient {
    /// The underlying prover implementation.
    pub prover: Box<dyn Prover<DefaultProverComponents>>,
}

impl ProverClient {
    /// Creates a new [ProverClient].
    ///
    /// ### Examples
    ///
    /// ```no_run
    /// use bf_sdk::ProverClient;
    ///
    /// let client = ProverClient::new();
    /// ```
    pub fn new() -> Self {
        Self { prover: Box::new(CpuProver::new()) }
    }

    /// Returns a [ProverClientBuilder] to easily create a [ProverClient].
    pub fn builder() -> ProverClientBuilder {
        ProverClientBuilder::default()
    }

    /// Prepare to execute the given program on the given input (without generating a proof).
    /// The returned [action::Execute] may be configured via its methods before running.
    ///
    /// To execute, call [action::Execute::run], which returns the output.
    ///
    /// ### Examples
    /// ```no_run
    /// use bf_sdk::ProverClient;
    ///
    /// // Load the program.
    /// let elf = test_artifacts::FIBO_BF;
    ///
    /// // Initialize the prover client.
    /// let client = ProverClient::new();
    ///
    /// // Setup the inputs.
    /// let stdin = vec![17];
    ///
    /// // Execute the program on the inputs.
    /// let output = client.execute(elf, stdin).run().unwrap();
    /// ```
    pub fn execute<'a>(&'a self, elf: &'a str, stdin: Vec<u8>) -> action::Execute<'a> {
        action::Execute::new(self.prover.as_ref(), elf, stdin)
    }

    /// Prepare to prove the execution of the given program with the given input.
    ///
    /// To prove, call [action::Prove::run], which returns a proof of the program's execution.
    ///
    /// ### Examples
    /// ```no_run
    /// use bf_sdk::ProverClient;
    ///
    /// // Load the program.
    /// let elf = test_artifacts::FIBO_BF;
    ///
    /// // Initialize the prover client.
    /// let client = ProverClient::new();
    ///
    /// // Setup the program.
    /// let (pk, vk) = client.setup(elf);
    ///
    /// // Setup the inputs.
    /// let stdin = vec![17];
    ///
    /// // Generate the proof.
    /// let proof = client.prove(&pk, stdin).run().unwrap();
    /// ```
    pub fn prove<'a>(&'a self, pk: &'a BfProvingKey, stdin: Vec<u8>) -> action::Prove<'a> {
        action::Prove::new(self.prover.as_ref(), pk, stdin)
    }

    /// Verifies that the given proof is valid and matches the given verification key produced by
    /// [Self::setup].
    ///
    /// ### Examples
    /// ```no_run
    /// use bf_sdk::ProverClient;
    ///
    /// let elf = test_artifacts::FIBO_BF;
    /// let client = ProverClient::new();
    /// let (pk, vk) = client.setup(elf);
    /// let stdin = vec![17];
    /// let proof = client.prove(&pk, stdin).run().unwrap();
    /// client.verify(&proof, &vk).unwrap();
    /// ```
    pub fn verify(
        &self,
        proof: &BfProofWithPublicValues,
        vk: &BfVerifyingKey,
    ) -> Result<(), BfVerificationError> {
        self.prover.verify(proof, vk)
    }

    /// Setup a program to be proven and verified by the zkVM by computing the proving
    /// and verifying keys.
    ///
    /// The proving key and verifying key essentially embed the program, as well as other auxiliary
    /// data (such as lookup tables) that are used to prove the program's correctness.
    ///
    /// ### Examples
    /// ```no_run
    /// use bf_sdk::ProverClient;
    ///
    /// let elf = test_artifacts::FIBO_BF;
    /// let client = ProverClient::new();
    /// let stdin = vec![17];
    /// let (pk, vk) = client.setup(elf);
    /// ```
    pub fn setup(&self, elf: &str) -> (BfProvingKey, BfVerifyingKey) {
        self.prover.setup(elf)
    }
}

impl Default for ProverClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder type for [`ProverClient`].
#[derive(Debug, Default)]
pub struct ProverClientBuilder {
    private_key: Option<String>,
    skip_simulation: bool,
}

impl ProverClientBuilder {
    ///  Sets the private key.
    pub fn private_key(mut self, private_key: String) -> Self {
        self.private_key = Some(private_key);
        self
    }

    /// Skips simulation.
    pub fn skip_simulation(mut self) -> Self {
        self.skip_simulation = true;
        self
    }

    /// Builds a [ProverClient], using the provided private key.
    pub fn build(self) -> ProverClient {
        ProverClient::new()
    }
}

#[cfg(test)]
mod tests {
    use super::setup_logger;
    use crate::ProverClient;

    #[test]
    fn test_execute() {
        setup_logger();
        let client = ProverClient::new();
        let elf = test_artifacts::FIBO_BF;
        let stdin = vec![17];
        let output = client.execute(elf, stdin).run().unwrap();
        assert_eq!(85, output[0]);
    }

    #[test]
    fn test_e2e_core() {
        setup_logger();
        let client = ProverClient::new();
        let elf = test_artifacts::FIBO_BF;
        let (pk, vk) = client.setup(elf);
        let stdin = vec![17];

        // Generate proof & verify.
        let proof = client.prove(&pk, stdin).run().unwrap();
        client.verify(&proof, &vk).unwrap();
    }
}
