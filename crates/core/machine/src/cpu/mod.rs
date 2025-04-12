mod air;
pub mod cols;
mod trace;

pub use cols::*;

/// A chip that implements the CPU.
#[derive(Default)]
pub struct CpuChip;

#[cfg(test)]
mod tests {
    use p3_field::{Field, FieldAlgebra};
    use p3_koala_bear::KoalaBear;
    use p3_matrix::dense::RowMajorMatrix;

    use bf_core_executor::{events::CpuEvent, ExecutionRecord, Opcode};
    use bf_stark::{
        air::MachineAir, koala_bear_poseidon2::KoalaBearPoseidon2, StarkGenericConfig,
    };

    use crate::utils::{uni_stark_prove as prove, uni_stark_verify as verify};
    use super::CpuChip;

    // #[test]
    // fn prove_cpu() {
    //     let config = KoalaBearPoseidon2::new();
    //     let mut challenger = config.challenger();

    //     let mut shard = ExecutionRecord::default();
    //     shard.cpu_events.push(CpuEvent::new(
    //         1,
    //         5,
    //         Opcode::LoopStart,
    //         5,
    //         0,
    //     ));

    //     let chip = CpuChip::default();
    //     let trace: RowMajorMatrix<KoalaBear> =
    //         chip.generate_trace(&shard, &mut ExecutionRecord::default());
    //     let proof = prove::<KoalaBearPoseidon2, _>(&config, &chip, &mut challenger, trace);

    //     let mut challenger = config.challenger();
    //     verify(&config, &chip, &mut challenger, &proof).unwrap();
    // }
}
