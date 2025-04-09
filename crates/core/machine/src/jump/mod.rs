mod air;
mod cols;
mod trace;

pub use cols::*;

#[derive(Default)]
pub struct JumpChip;

#[cfg(test)]
mod tests {
    use p3_field::{Field, FieldAlgebra};
    use p3_koala_bear::KoalaBear;
    use p3_matrix::dense::RowMajorMatrix;

    use bf_core_executor::{events::JumpEvent, ExecutionRecord, Opcode};
    use bf_stark::{
        air::MachineAir, koala_bear_poseidon2::KoalaBearPoseidon2, StarkGenericConfig,
    };

    use crate::utils::{uni_stark_prove as prove, uni_stark_verify as verify};
    use super::JumpChip;

    #[test]
    fn test_zero() {
        type F = KoalaBear;

        let mv = F::from_canonical_u8(0);
        assert_eq!(mv, F::ZERO);

        let mv = F::from_canonical_u8(2);
        let is_mv_zero = F::from_canonical_u8(1) - mv * mv.inverse();
        assert_eq!(is_mv_zero, F::ZERO);
    }

    #[test]
    fn prove_jump() {
        let config = KoalaBearPoseidon2::new();
        let mut challenger = config.challenger();

        let mut shard = ExecutionRecord::default();
        shard.jump_events.push(JumpEvent::new(
            1,
            5,
            Opcode::LoopStart,
            5,
            0,
        ));
        shard.jump_events.push(JumpEvent::new(
            1,
            2,
            Opcode::LoopStart,
            5,
            1,
        ));
        shard.jump_events.push(JumpEvent::new(
            1,
            5,
            Opcode::LoopEnd,
            5,
            5,
        ));
        shard.jump_events.push(JumpEvent::new(
            1,
            2,
            Opcode::LoopEnd,
            5,
            0,
        ));

        let chip = JumpChip::default();
        let trace: RowMajorMatrix<KoalaBear> =
            chip.generate_trace(&shard, &mut ExecutionRecord::default());
        let proof = prove::<KoalaBearPoseidon2, _>(&config, &chip, &mut challenger, trace);

        let mut challenger = config.challenger();
        verify(&config, &chip, &mut challenger, &proof).unwrap();
    }
}
