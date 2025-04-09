mod air;
mod cols;
mod trace;

pub use cols::*;

#[derive(Default)]
pub struct AddSubChip;

#[cfg(test)]
mod tests {
    use p3_koala_bear::KoalaBear;
    use p3_matrix::dense::RowMajorMatrix;
    use rand::{rng, Rng};

    use bf_core_executor::{events::AluEvent, ExecutionRecord, Opcode};
    use bf_stark::{
        air::MachineAir, koala_bear_poseidon2::KoalaBearPoseidon2, StarkGenericConfig,
    };

    use crate::utils::{uni_stark_prove as prove, uni_stark_verify as verify};
    use super::AddSubChip;

    #[test]
    fn generate_trace() {
        let mut shard = ExecutionRecord::default();
        shard.add_events = vec![AluEvent::new(0, Opcode::Add, 11, 10)];
        let chip = AddSubChip::default();
        let trace: RowMajorMatrix<KoalaBear> =
            chip.generate_trace(&shard, &mut ExecutionRecord::default());
        println!("{:?}", trace.values)
    }

    #[test]
    fn prove_koala_bear() {
        let config = KoalaBearPoseidon2::new();
        let mut challenger = config.challenger();

        let mut shard = ExecutionRecord::default();
        for i in 0..255 {
            let mv = rng().random_range(0..u8::MAX);
            let mv_next = mv.wrapping_add(1);
            shard.add_events.push(AluEvent::new(
                i << 2,
                Opcode::Add,
                mv_next,
                mv,
            ));
        }
        for i in 0..255 {
            let mv = rng().random_range(0..u8::MAX);
            let mv_next = mv.wrapping_sub(1);
            shard.add_events.push(AluEvent::new(
                i << 2,
                Opcode::Sub,
                mv_next,
                mv,
            ));
        }

        let chip = AddSubChip::default();
        let trace: RowMajorMatrix<KoalaBear> =
            chip.generate_trace(&shard, &mut ExecutionRecord::default());
        let proof = prove::<KoalaBearPoseidon2, _>(&config, &chip, &mut challenger, trace);

        let mut challenger = config.challenger();
        verify(&config, &chip, &mut challenger, &proof).unwrap();
    }
}
