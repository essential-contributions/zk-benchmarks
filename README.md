# ZK Benchmarks

A series of benchmarks to gage pain points during the fraud proof construction process. These benchmarks are using SP1 due to the fact that it is currently the fastest RISCV zkvm for consumer grade hardware. The goal is for consumer grade mid-tier hardware to be able to generate a fraud proof in an hour or less.

#### TODO
- Increase the sample size once SP1 has stable continuations with aggregated proofs

## Running the Benchmarks

First, install the [SP1 toolchain](https://succinctlabs.github.io/sp1/getting-started/install.html) then clone the repo and run the following...

```
cd script
RUSTFLAGS="-Ctarget-cpu=native" cargo run --release
```

The flag `RUSTFLAGS="-Ctarget-cpu=native"` ensures the prover takes advantage of vector based instructions.

### Results

```
Intel Core i7-1260P - May 3rd, 2024

5.84s to prove a hash of 8k bytes
47.39s to prove an ed25519 signature
49.87s to prove a sparse merkle tree proof
```
```
Intel Core i7-8700 3.2GHz - May 3rd, 2024

4.23s to prove a hash of 8k bytes
37.65s to prove an ed25519 signature
27.05s to prove a sparse merkle tree proof
```
```
Intel Core i7-13700K - May 3rd, 2024

1.74s to prove a hash of 8k bytes
13.45s to prove an ed25519 signature
9.92s to prove a sparse merkle tree proof
```
