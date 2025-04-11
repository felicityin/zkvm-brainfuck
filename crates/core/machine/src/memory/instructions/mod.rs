pub mod air;
pub mod cols;
pub mod trace;

pub use cols::*;

#[derive(Default)]
pub struct MemoryInstructionsChip;

#[cfg(test)]
mod tests {
    use p3_koala_bear::KoalaBear;
    use p3_matrix::dense::RowMajorMatrix;

    use bf_core_executor::{events::MemInstrEvent, ExecutionRecord, Opcode};
    use bf_stark::{
        air::MachineAir, koala_bear_poseidon2::KoalaBearPoseidon2, StarkGenericConfig,
    };

    use crate::utils::{uni_stark_prove as prove, uni_stark_verify as verify};
    use super::MemoryInstructionsChip;

    #[test]
    fn prove_mem_instrs() {
        let config = KoalaBearPoseidon2::new();
        let mut challenger = config.challenger();

        let mut shard = ExecutionRecord::default();
        shard.memory_instr_events.push(MemInstrEvent::new(
            1,
            1,
            Opcode::MemStepForward,
            1,
            2,
        ));
        shard.memory_instr_events.push(MemInstrEvent::new(
            1,
            1,
            Opcode::MemStepBackward,
            2,
            1,
        ));

        let chip = MemoryInstructionsChip::default();
        let trace: RowMajorMatrix<KoalaBear> =
            chip.generate_trace(&shard, &mut ExecutionRecord::default());
        let proof = prove::<KoalaBearPoseidon2, _>(&config, &chip, &mut challenger, trace);

        let mut challenger = config.challenger();
        verify(&config, &chip, &mut challenger, &proof).unwrap();
    }
}
