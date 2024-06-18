//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can be verified
//! on-chain.
//!
//! You can run this script using the following command:
//! ```shell
//! RUSTFLAGS="-Ctarget-cpu=native" RUST_LOG=info cargo run --bin prove --release
//! ```

use std::{path::PathBuf, time::Instant};

use clap::Parser;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sp1_sdk::{HashableKey, ProverClient, SP1PlonkBn254Proof, SP1ProvingKey, SP1Stdin, SP1VerifyingKey};
use zk_benchmarks_core::Outputs;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
///
/// This file is generated by running `cargo prove build` inside the `program` directory.
pub const ZK_BENCHMARKS_ELF: &[u8] = include_bytes!("../../../program/elf/riscv32im-succinct-zkvm-elf");
const OP_HASH_8K: u8 = 0;
const OP_SIGNATURE: u8 = 1;
const OP_SMT_PROOF: u8 = 2;

/// The arguments for the prove command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct ProveArgs {
    #[clap(long, default_value = "false")]
    evm: bool,
}

fn main() {
    // Setup the logger and parse the command line arguments.
    sp1_sdk::utils::setup_logger();
    let args = ProveArgs::parse();

    // Setup the prover client and program.
    println!("Setting up client...");
    let start = Instant::now();
    let client = ProverClient::new();
    let (pk, vk) = client.setup(ZK_BENCHMARKS_ELF);
    println!("(finished in {:?})", start.elapsed());
    println!();

    // Run each benchmark.
    benchmark_hash_8k(&client, &pk, &vk, 4, args.evm);
    benchmark_signature(&client, &pk, &vk, 2, args.evm);
    benchmark_smt_proof(&client, &pk, &vk, 2, args.evm);
}

/// Run the 8k chunk hash benchmark.
fn benchmark_hash_8k(client: &ProverClient, pk: &SP1ProvingKey, vk: &SP1VerifyingKey, repeat: u32, evm: bool) {
    println!("[Hash 8k]");
    const HASH_SIZE: u32 = 8192;

    // Write inputs
    let mut encoded = vec![OP_HASH_8K];
    encoded.extend(repeat.to_le_bytes());
    encoded.extend(HASH_SIZE.to_le_bytes());
    for _ in 0..repeat {
        encoded.extend([0u8; HASH_SIZE as usize]);
    }
    let mut stdin = SP1Stdin::new();
    stdin.write_slice(encoded.as_slice());

    if evm {
        // Generate the proof.
        println!("Generating proof wrapped for EVM...");
        let start = Instant::now();
        let mut proof = client.prove_plonk(&pk, stdin).expect("failed to generate proof");
        let elapsed = start.elapsed();

        // Print the outputs.
        create_plonk_fixture(&proof, &vk);
        let out = proof.public_values.read::<Outputs>();
        println!("Proved {} hashes with result {}", out.num_iterations, hex::encode(out.hash_output));
        println!("{:?} to prove a hash of 8k bytes", elapsed / repeat);
        println!();

    } else {
        // Generate the proof.
        println!("Generating proof...");
        let start = Instant::now();
        let mut proof = client.prove(&pk, stdin).expect("failed to generate proof");
        let elapsed = start.elapsed();

        // Print the outputs and verify zk proof.
        let out = proof.public_values.read::<Outputs>();
        client.verify(&proof, vk).expect("verification failed");
        println!("Proved {} hashes with result {}", out.num_iterations, hex::encode(out.hash_output));
        println!("{:?} to prove a hash of 8k bytes", elapsed / repeat);
        println!();
    }
}

/// Run the ed25519 signature benchmark.
fn benchmark_signature(client: &ProverClient, pk: &SP1ProvingKey, vk: &SP1VerifyingKey, repeat: u32, evm: bool) {
    println!("[ED25519 Signature]");
    let message = "validating block: 19634367";
    let message_hash = sha256_hash(message.as_bytes());

    // Make signatures
    let mut public_keys: Vec<[u8; 32]> = Vec::new();
    let mut signatures: Vec<[u8; 64]> = Vec::new();
    let mut randomness = sha256_hash(&[0xcd; 32]);
    for _i in 0..repeat {
        randomness = sha256_hash(&randomness);
        let signing_key: SigningKey = SigningKey::from_bytes(&randomness);
        let verifying_key: VerifyingKey = signing_key.verifying_key();
        let signature: Signature = signing_key.sign(&message_hash);

        public_keys.push(verifying_key.to_bytes());
        signatures.push(signature.to_bytes());

        assert!(verifying_key.verify_strict(&message_hash, &signature).is_ok());
    }

    // Write inputs
    let mut encoded = vec![OP_SIGNATURE];
    encoded.extend(repeat.to_le_bytes());
    encoded.extend_from_slice(&message_hash);
    for (public_key, signature) in public_keys.into_iter().zip(signatures) {
        encoded.extend_from_slice(&public_key);
        encoded.extend_from_slice(&signature);
    }
    let mut stdin = SP1Stdin::new();
    stdin.write_slice(encoded.as_slice());

    if evm {
        // Generate the proof.
        println!("Generating proof wrapped for EVM...");
        let start = Instant::now();
        let mut proof = client.prove_plonk(&pk, stdin).expect("failed to generate proof");
        let elapsed = start.elapsed();

        // Print the outputs.
        create_plonk_fixture(&proof, &vk);
        let out = proof.public_values.read::<Outputs>();
        println!("Proved {} signatures for message {}", out.num_iterations, hex::encode(out.hash_output));
        println!("{:?} to prove an ed25519 signature", elapsed / repeat);
        println!();

    } else {
        // Generate the proof.
        println!("Generating proof...");
        let start = Instant::now();
        let mut proof = client.prove(&pk, stdin).expect("failed to generate proof");
        let elapsed = start.elapsed();

        // Print the outputs and verify zk proof.
        let out = proof.public_values.read::<Outputs>();
        client.verify(&proof, vk).expect("verification failed");
        println!("Proved {} signatures for message {}", out.num_iterations, hex::encode(out.hash_output));
        println!("{:?} to prove an ed25519 signature", elapsed / repeat);
        println!();
    }
}

/// Run the sparse merkle tree benchmark.
fn benchmark_smt_proof(client: &ProverClient, pk: &SP1ProvingKey, vk: &SP1VerifyingKey, repeat: u32, evm: bool) {
    println!("[Sparse Merkle Tree Proof]");

    // Make merkle proofs
    let mut proof = [[0u8; 32]; 256];
    let leaf = sha256_hash(&[12u8; 64]);
    let mut hash = leaf;
    for i in 0..256 {
        proof[i] = hash;

        let mut combined = [0u8; 64];
        combined[0..32].copy_from_slice(&hash);
        combined[32..64].copy_from_slice(&hash);
        hash = sha256_hash(&combined);
    }

    // Write inputs
    let mut encoded = vec![OP_SMT_PROOF];
    encoded.extend(repeat.to_le_bytes());
    encoded.extend(proof[255]);
    for i in 0..repeat {
        let mut index = [0u8; 32];
        index[28..32].copy_from_slice(&(1000u32 + i).to_le_bytes());
        encoded.extend(index);
        encoded.extend(leaf);
        for p in proof.iter().take(255) {
            encoded.extend(p);
        }
    }
    let mut stdin = SP1Stdin::new();
    stdin.write_slice(encoded.as_slice());

    if evm {
        // Generate the proof.
        println!("Generating proof wrapped for EVM...");
        let start = Instant::now();
        let mut proof = client.prove_plonk(&pk, stdin).expect("failed to generate proof");
        let elapsed = start.elapsed();

        // Print the outputs.
        create_plonk_fixture(&proof, &vk);
        let out = proof.public_values.read::<Outputs>();
        println!("Proved {} SMT proofs with root {}", out.num_iterations, hex::encode(out.hash_output));
        println!("{:?} to prove a sparse merkle tree proof", elapsed / repeat);
        println!();

    } else {
        // Generate the proof.
        println!("Generating proof...");
        let start = Instant::now();
        let mut proof = client.prove(&pk, stdin).expect("failed to generate proof");
        let elapsed = start.elapsed();

        // Print the outputs and verify zk proof.
        let out = proof.public_values.read::<Outputs>();
        client.verify(&proof, vk).expect("verification failed");
        println!("Proved {} SMT proofs with root {}", out.num_iterations, hex::encode(out.hash_output));
        println!("{:?} to prove a sparse merkle tree proof", elapsed / repeat);
        println!();
    }
}

fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// A fixture that can be used to test the verification of SP1 zkVM proofs inside Solidity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SP1ZKBenchmarksProofFixture {
    vkey: String,
    public_values: String,
    proof: String,
}

/// Create a fixture for the given proof.
fn create_plonk_fixture(proof: &SP1PlonkBn254Proof, vk: &SP1VerifyingKey) {
    // Create the testing fixture so we can test things end-ot-end.
    let fixture = SP1ZKBenchmarksProofFixture {
        vkey: vk.bytes32().to_string(),
        public_values: proof.public_values.bytes().to_string(),
        proof: proof.bytes().to_string(),
    };

    // The verification key is used to verify that the proof corresponds to the execution of the
    // program on the given input.
    //
    // Note that the verification key stays the same regardless of the input.
    println!("Verification Key: {}", fixture.vkey);

    // The public values are the values whicha are publically commited to by the zkVM.
    //
    // If you need to expose the inputs or outputs of your program, you should commit them in
    // the public values.
    println!("Public Values: {}", fixture.public_values);

    // The proof proves to the verifier that the program was executed with some inputs that led to
    // the give public values.
    println!("Proof Bytes: {}", fixture.proof);

    // Save the fixture to a file.
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../contracts/src/fixtures");
    std::fs::create_dir_all(&fixture_path).expect("failed to create fixture path");
    std::fs::write(
        fixture_path.join("fixture.json"),
        serde_json::to_string_pretty(&fixture).unwrap(),
    )
    .expect("failed to write fixture");
}