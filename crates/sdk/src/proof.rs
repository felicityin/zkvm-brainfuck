use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use bf_prover::CoreSC;
use bf_stark::{MachineVerificationError, ShardProof};

/// A proof generated with Bf, bundled together with stdin, public values, and the zkMIPS version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BfProofWithPublicValues {
    pub proof: ShardProof<CoreSC>,
    pub stdin: Vec<u8>,
}

pub type BfCoreProofVerificationError = MachineVerificationError<CoreSC>;
