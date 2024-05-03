use op_benchmarks_core::Outputs;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use sha2::{Digest, Sha256};
use sp1_sdk::{ProverClient, SP1ProvingKey, SP1Stdin, SP1VerifyingKey};
use std::time::Instant;

const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");
const OP_HASH_8K: u8 = 0;
const OP_SIGNATURE: u8 = 1;
const OP_SMT_PROOF: u8 = 2;

fn main() {
    //sp1_sdk::utils::setup_logger();

    // Setup
    println!("Setting up client...");
    let start = Instant::now();
    let client = ProverClient::new();
    let (pk, vk) = client.setup(ELF);
    println!("(finished in {:?})", start.elapsed());
    println!();

    // Run benchmarks
    benchmark_hash_8k(&client, &pk, &vk, 4);
    benchmark_signature(&client, &pk, &vk, 2);
    benchmark_smt_proof(&client, &pk, &vk, 2);
}

fn benchmark_hash_8k(client: &ProverClient, pk: &SP1ProvingKey, vk: &SP1VerifyingKey, repeat: u32) {
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

    // Generate proof
    println!("Generating proof...");
    let start = Instant::now();
    let mut proof = client.prove(pk, stdin).expect("proving failed");
    let elapsed = start.elapsed();

    // Print the outputs and verify zk proof
    let out = proof.public_values.read::<Outputs>();
    client.verify(&proof, vk).expect("verification failed");
    println!("Proved {} hashes with result {}", out.num_iterations, hex::encode(out.hash_output));
    println!("{:?} to prove a hash of 8k bytes", elapsed / repeat);
    println!();
}

fn benchmark_signature(client: &ProverClient, pk: &SP1ProvingKey, vk: &SP1VerifyingKey, repeat: u32) {
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

    // Generate proof
    println!("Generating proof...");
    let start = Instant::now();
    let mut proof = client.prove(pk, stdin).expect("proving failed");
    let elapsed = start.elapsed();

    // Print the outputs and verify zk proof
    let out = proof.public_values.read::<Outputs>();
    client.verify(&proof, vk).expect("verification failed");
    println!("Proved {} signatures for message {}", out.num_iterations, hex::encode(out.hash_output));
    println!("{:?} to prove an ed25519 signature", elapsed / repeat);
    println!();
}

fn benchmark_smt_proof(client: &ProverClient, pk: &SP1ProvingKey, vk: &SP1VerifyingKey, repeat: u32) {
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

    // Generate proof
    println!("Generating proof...");
    let start = Instant::now();
    let mut proof = client.prove(pk, stdin).expect("proving failed");
    let elapsed = start.elapsed();

    // Print the outputs and verify zk proof
    let out = proof.public_values.read::<Outputs>();
    client.verify(&proof, vk).expect("verification failed");
    println!("Proved {} SMT proofs with root {}", out.num_iterations, hex::encode(out.hash_output));
    println!("{:?} to prove a sparse merkle tree proof", elapsed / repeat);
    println!();
}

fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}
