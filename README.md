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
Rockchip RK3588 (OrangePi5Plus 16GB) - June 3rd, 2024

8.77s to prove a hash of 8k bytes
80.70s to prove an ed25519 signature
53.15s to prove a sparse merkle tree proof
```
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

#### AWS (mid range)

```
c6i.2xlarge	($0.34)	8vCPU	16GiB - May 31st, 2024

3.47s to prove a hash of 8k bytes
30.86s to prove an ed25519 signature
21.03s to prove a sparse merkle tree proof
```
```
c6a.2xlarge	($0.31)	8vCPU	16GiB - May 31st, 2024

3.57s to prove a hash of 8k bytes
33.47s to prove an ed25519 signature
21.93s to prove a sparse merkle tree proof
```
```
c6g.2xlarge	($0.27)	8vCPU	16GiB - May 31st, 2024

5.42s to prove a hash of 8k bytes
49.58s to prove an ed25519 signature
32.95s to prove a sparse merkle tree proof
```

#### AWS (high end)

```
c6a.8xlarge	($1.22)	32vCPU	64GiB - May 31st, 2024

1.65s to prove a hash of 8k bytes
10.31s to prove an ed25519 signature
10.61s to prove a sparse merkle tree proof
```
```
c6i.8xlarge	($1.36)	32vCPU	64GiB - May 31st, 2024

1.56s to prove a hash of 8k bytes
9.17s to prove an ed25519 signature
9.78s to prove a sparse merkle tree proof
```
