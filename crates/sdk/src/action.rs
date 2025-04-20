use anyhow::{Ok, Result};

use bf_prover::components::DefaultProverComponents;
use bf_prover::types::BfProvingKey;

use crate::{BfProofWithPublicValues, Prover};

/// Builder to prepare and configure execution of a program on an input.
/// May be run with [Self::run].
pub struct Execute<'a> {
    prover: &'a dyn Prover<DefaultProverComponents>,
    elf: &'a str,
    stdin: Vec<u8>,
}

impl<'a> Execute<'a> {
    /// Prepare to execute the given program on the given input (without generating a proof).
    ///
    /// Prefer using [ProverClient::execute](super::ProverClient::execute).
    /// See there for more documentation.
    pub fn new(
        prover: &'a dyn Prover<DefaultProverComponents>,
        elf: &'a str,
        stdin: Vec<u8>,
    ) -> Self {
        Self { prover, elf, stdin }
    }

    /// Execute the program on the input, consuming the built action `self`.
    pub fn run(self) -> Result<Vec<u8>> {
        let Self { prover, elf, stdin } = self;
        Ok(prover.prover().execute(elf, stdin)?)
    }
}

/// Builder to prepare and configure proving execution of a program on an input.
/// May be run with [Self::run].
pub struct Prove<'a> {
    prover: &'a dyn Prover<DefaultProverComponents>,
    pk: &'a BfProvingKey,
    stdin: Vec<u8>,
}

impl<'a> Prove<'a> {
    /// Prepare to prove the execution of the given program with the given input.
    ///
    /// Prefer using [ProverClient::prove](super::ProverClient::prove).
    /// See there for more documentation.
    pub fn new(
        prover: &'a dyn Prover<DefaultProverComponents>,
        pk: &'a BfProvingKey,
        stdin: Vec<u8>,
    ) -> Self {
        Self { prover, pk, stdin }
    }

    /// Prove the execution of the program on the input, consuming the built action `self`.
    pub fn run(self) -> Result<BfProofWithPublicValues> {
        let Self { prover, pk, stdin } = self;
        prover.prove(pk, stdin)
    }
}
