use serde::{de::DeserializeOwned, Deserialize, Serialize};

use bf_stark::{ShardProof, StarkProvingKey, StarkVerifyingKey};

use crate::CoreSC;

/// The information necessary to generate a proof for a given program.
#[derive(Clone, Serialize, Deserialize)]
pub struct BfProvingKey {
    pub pk: StarkProvingKey<CoreSC>,
    pub elf: String,
    /// Verifying key is also included as we need it for recursion
    pub vk: BfVerifyingKey,
}

/// The information necessary to verify a proof for a given program.
#[derive(Clone, Serialize, Deserialize)]
pub struct BfVerifyingKey {
    pub vk: StarkVerifyingKey<CoreSC>,
}

/// A proof of a ELF execution with given inputs and outputs.
#[derive(Serialize, Deserialize, Clone)]
#[serde(bound(serialize = "P: Serialize"))]
#[serde(bound(deserialize = "P: DeserializeOwned"))]
pub struct BfProofWithMetadata<P: Clone> {
    pub proof: P,
    pub stdin: Vec<u8>,
    pub public_values: Vec<u8>,
    pub cycles: u64,
}

/// A proof of a program without any wrapping.
pub type BfCoreProof = BfProofWithMetadata<BfCoreProofData>;

#[derive(Serialize, Deserialize, Clone)]
pub struct BfCoreProofData(pub ShardProof<CoreSC>);
