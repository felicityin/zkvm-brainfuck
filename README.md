# Description

> This repo is inspired by [SP1](https://github.com/succinctlabs/sp1) and [zkMIPS](https://github.com/zkMIPS/zkMIPS).

zkvm-brainfuck is a ZK-VM for the Brainfuck language.

- Designed as a register machine instead of a stack machine with RAM.
- Compared to SP1, this system employs a single shard and produces only core proofs.
- Lookup serves two key purposes:
   - Cross-Chip Communication - The chip needs to send the logic which itself cannot verify to other chips for verification.
   - Consistency of memory access (the data read by the memory is the data written before) - Proving that the read and write data are “permuted”.

# Usage

```rust
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
    println!("result: {:?}", output);

    // Generate the proof for the given guest and input.
    let (pk, vk) = client.setup(ELF);
    let proof = client.prove(&pk, stdin).run().unwrap();
    println!("generated proof");

    // Verify proof and public values
    client.verify(&proof, &vk).expect("verification failed");
}
```

# Test

Test all.
```shell
cargo test -r
```

Test e2e.
```
cargo test -r test_e2e_core
```

Debug.
```
RUST_LOG=debug cargo test -r test_e2e_core --features debug -- --nocapture
```

# Reference

[stark-brainfuck](https://aszepieniec.github.io/stark-brainfuck/index)
