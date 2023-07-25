# Plonk Implementation of ArionHash

This repository contains the pure [Rust](https://www.rust-lang.org/) implementation of ArionHash in the [Dusk Network Plonk](https://github.com/dusk-network/plonk) library.
It is based on the Dusk Network [Hades252](https://github.com/dusk-network/Hades252) and [Poseidon252](https://github.com/dusk-network/Poseidon252).

The code for generating Merkle trees and benchmarking is forked from [dusk-merkle](https://github.com/dusk-network/merkle) (commit `1988ee18221354cedf2d5878f8a867f3d670a6a0`).

## Usage

To benchmark ArionHash make sure that [Rust](https://www.rust-lang.org/) is installed on your system.
Then one can compile and run the benchmark using the following commands.

```bash
cd Merkle Tree Benchmark
cargo bench --features zk
```
