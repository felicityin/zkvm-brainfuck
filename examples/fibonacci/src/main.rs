use bf_sdk::{utils, ProverClient};

/// The ELF we want to execute inside the zkVM.
const ELF: &str = ",>+>+<<[->>[->+>+<<]<[->>+<<]>>[-<+>]>[-<<<+>>>]<<<<]>>.";

fn main() {
    // Setup logging.
    utils::setup_logger();

    // Create an input stream and write '17' to it.
    let stdin = vec![17];

    // Create a `ProverClient` method.
    let client = ProverClient::new();

    // Execute the guest using the `ProverClient.execute` method, without generating a proof.
    let output = client.execute(ELF, stdin.clone()).run().unwrap();
    println!("result: {output:?}");

    // Generate the proof for the given guest and input.
    let (pk, vk) = client.setup(ELF);
    let proof = client.prove(&pk, stdin).run().unwrap();
    println!("generated proof");

    // Verify proof and public values
    client.verify(&proof, &vk).expect("verification failed");
}
