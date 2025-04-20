# Description

> This repo is inspired by [SP1](https://github.com/succinctlabs/sp1) and [zkMIPS](https://github.com/zkMIPS/zkMIPS).
> This repo is still in progress.

zkvm-brainfuck is a ZK-VM for the Brainfuck language.

- Designed as a register machine instead of a stack machine that utilizes RAM.
- Compared to SP1, this system employs a single shard and produces only core proofs.

# Usage

```rust
use bf_sdk::ProverClient;

setup_logger();
let client = ProverClient::new();
let elf = test_artifacts::FIBO_BF;
let stdin = vec![17];

// Execute
let output = client.execute(elf, stdin).run().unwrap();
assert_eq!(85, output[0]);

let (pk, vk) = client.setup(elf);

// Generate proof & verify.
let proof = client.prove(&pk, stdin).run().unwrap();
client.verify(&proof, &vk).unwrap();
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
