use p3_field::PrimeField32;
use p3_koala_bear::KoalaBear;
use size::Size;
use thiserror::Error;
use web_time::Instant;

use bf_core_executor::{
    ExecutionError, Executor, Program,
};
use bf_stark::{
    koala_bear_poseidon2::KoalaBearPoseidon2,
    MachineVerificationError,
};
use bf_stark::{
    Com, StarkGenericConfig, UniConfig, MachineProof, MachineProver,
    OpeningProof, PcsProverData,
};

use crate::brainfuck::BfAir;

#[derive(Error, Debug)]
pub enum BfCoreProverError {
    #[error("failed to execute program: {0}")]
    ExecutionError(ExecutionError),
    #[error("serialization error: {0}")]
    SerializationError(bincode::Error),
}

pub fn prove<SC: StarkGenericConfig, P: MachineProver<SC, BfAir<SC::Val>>>(
    prover: &P,
    pk: &P::DeviceProvingKey,
    program: Program,
    input: Vec<u8>,
) -> Result<(MachineProof<SC>, Vec<u8>, u64), BfCoreProverError>
where
    SC::Val: PrimeField32,
    SC::Challenger: 'static + Clone + Send,
    OpeningProof<SC>: Send,
    Com<SC>: Send + Sync,
    PcsProverData<SC>: Send + Sync,
{
    // Setup the runtime.
    let mut runtime = Executor::new(program, input);

    // Prove the program.
    let mut challenger = prover.config().challenger();
    let proving_start = Instant::now();
    let proof =
        prover.prove(&pk, &mut runtime.record, &mut challenger).unwrap();
    let proving_duration = proving_start.elapsed().as_millis();
    let nb_bytes = bincode::serialize(&proof).unwrap().len();

    // Print the summary.
    tracing::info!(
        "summary: cycles={}, e2e={}, khz={:.2}, proofSize={}",
        runtime.state.global_clk,
        proving_duration,
        (runtime.state.global_clk as f64 / proving_duration as f64),
        Size::from_bytes(nb_bytes),
    );

    #[cfg(feature = "debug")]
    {
        let mut challenger = prover.machine().config().challenger();
        let pk_host = prover.pk_to_host(pk);
        prover.machine().debug_constraints(&pk_host, runtime.record, &mut challenger);
    }

    Ok((proof, runtime.state.output_stream, runtime.state.global_clk))
}

pub fn run_test<P: MachineProver<KoalaBearPoseidon2, BfAir<KoalaBear>>>(
    program: Program,
    input: Vec<u8>,
) -> Result<MachineProof<KoalaBearPoseidon2>, MachineVerificationError<KoalaBearPoseidon2>> {
    let runtime = tracing::debug_span!("runtime.run(...)").in_scope(|| {
        let mut runtime = Executor::new(program, input);
        runtime.run().unwrap();
        runtime
    });
    run_test_core::<P>(runtime)
}

#[allow(unused_variables)]
pub fn run_test_core<P: MachineProver<KoalaBearPoseidon2, BfAir<KoalaBear>>>(
    runtime: Executor,
) -> Result<MachineProof<KoalaBearPoseidon2>, MachineVerificationError<KoalaBearPoseidon2>> {
    let config = KoalaBearPoseidon2::new();
    let machine = BfAir::machine(config);
    let prover = P::new(machine);

    let (pk, _) = prover.setup(runtime.program.as_ref());
    let (proof, output, _) = prove(
        &prover,
        &pk,
        Program::clone(&runtime.program),
        runtime.state.input_stream,
    )
    .unwrap();

    let config = KoalaBearPoseidon2::new();
    let machine = BfAir::machine(config);
    let (pk, vk) = machine.setup(runtime.program.as_ref());
    let mut challenger = machine.config().challenger();
    machine.verify(&vk, &proof, &mut challenger).unwrap();

    Ok(proof)
}

#[cfg(debug_assertions)]
#[cfg(not(doctest))]
pub fn uni_stark_prove<SC, A>(
    config: &SC,
    air: &A,
    challenger: &mut SC::Challenger,
    trace: RowMajorMatrix<SC::Val>,
) -> Proof<UniConfig<SC>>
where
    SC: StarkGenericConfig,
    A: Air<p3_uni_stark::SymbolicAirBuilder<SC::Val>>
        + for<'a> Air<p3_uni_stark::ProverConstraintFolder<'a, UniConfig<SC>>>
        + for<'a> Air<p3_uni_stark::DebugConstraintBuilder<'a, SC::Val>>,
{
    p3_uni_stark::prove(&UniConfig(config.clone()), air, challenger, trace, &vec![])
}

#[cfg(not(debug_assertions))]
pub fn uni_stark_prove<SC, A>(
    config: &SC,
    air: &A,
    challenger: &mut SC::Challenger,
    trace: RowMajorMatrix<SC::Val>,
) -> Proof<UniConfig<SC>>
where
    SC: StarkGenericConfig,
    A: Air<p3_uni_stark::SymbolicAirBuilder<SC::Val>>
        + for<'a> Air<p3_uni_stark::ProverConstraintFolder<'a, UniConfig<SC>>>,
{
    p3_uni_stark::prove(&UniConfig(config.clone()), air, challenger, trace, &vec![])
}

#[cfg(debug_assertions)]
#[cfg(not(doctest))]
pub fn uni_stark_verify<SC, A>(
    config: &SC,
    air: &A,
    challenger: &mut SC::Challenger,
    proof: &Proof<UniConfig<SC>>,
) -> Result<(), p3_uni_stark::VerificationError<p3_uni_stark::PcsError<UniConfig<SC>>>>
where
    SC: StarkGenericConfig,
    A: Air<p3_uni_stark::SymbolicAirBuilder<SC::Val>>
        + for<'a> Air<p3_uni_stark::VerifierConstraintFolder<'a, UniConfig<SC>>>
        + for<'a> Air<p3_uni_stark::DebugConstraintBuilder<'a, SC::Val>>,
{
    p3_uni_stark::verify(&UniConfig(config.clone()), air, challenger, proof, &vec![])
}

#[cfg(not(debug_assertions))]
pub fn uni_stark_verify<SC, A>(
    config: &SC,
    air: &A,
    challenger: &mut SC::Challenger,
    proof: &Proof<UniConfig<SC>>,
) -> Result<(), p3_uni_stark::VerificationError<p3_uni_stark::PcsError<UniConfig<SC>>>>
where
    SC: StarkGenericConfig,
    A: Air<p3_uni_stark::SymbolicAirBuilder<SC::Val>>
        + for<'a> Air<p3_uni_stark::VerifierConstraintFolder<'a, UniConfig<SC>>>,
{
    p3_uni_stark::verify(&UniConfig(config.clone()), air, challenger, proof, &vec![])
}

use p3_air::Air;
use p3_matrix::dense::RowMajorMatrix;
use p3_uni_stark::Proof;
