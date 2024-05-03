#![no_main]

sp1_zkvm::entrypoint!(main);

use ed25519_dalek::{Signature, VerifyingKey};
use sha2::{Digest, Sha256};
use op_benchmarks_core::Outputs;

const OP_HASH_8K: u8 = 0;
const OP_SIGNATURE: u8 = 1;
const OP_SMT_PROOF: u8 = 2;

fn main() {
    // get input
    let input = sp1_zkvm::io::read_vec();
    let op = input[0];

    // execute op
    if op == OP_HASH_8K {
        sp1_zkvm::io::commit(&execute_hash_8k(input.as_slice()));
    } else if op == OP_SIGNATURE {
        sp1_zkvm::io::commit(&execute_signature(input.as_slice()));
    } else if op == OP_SMT_PROOF {
        sp1_zkvm::io::commit(&execute_smt_proof(input.as_slice()));
    }
}

fn execute_hash_8k(input: &[u8]) -> Outputs {
    // decode inputs
    let repeat = u32::from_le_bytes([input[1], input[2], input[3], input[4]]);
    let hash_size = u32::from_le_bytes([input[5], input[6], input[7], input[8]]) as usize;

    // do hashing
    let mut result: [u8; 32] = [0u8; 32];
    for i in 0..(repeat as usize) {
        let data = &input[(i * hash_size + 9)..((i+ 1) * hash_size + 9)];
        result = sha256_hash(data);
    }
    
    Outputs {
        hash_output: result,
        num_iterations: repeat,
    }
}

fn execute_signature(input: &[u8]) -> Outputs {
    // decode inputs
    let repeat = u32::from_le_bytes([input[1], input[2], input[3], input[4]]);
    let mut message_hash = [0u8; 32];
    message_hash.copy_from_slice(&input[5..37]);

    // validate signatures
    for i in 0..(repeat as usize) {
        let mut public_key_bytes = [0u8; 32];
        public_key_bytes.copy_from_slice(&input[(i * 96 + 37)..(i * 96 + 69)]);
        let mut signature_bytes = [0u8; 64];
        signature_bytes.copy_from_slice(&input[(i * 96 + 69)..(i * 96 + 133)]);

        let verifying_key = VerifyingKey::from_bytes(&public_key_bytes).unwrap();
        let signature = Signature::from_bytes(&signature_bytes);
        assert!(verifying_key.verify_strict(&message_hash, &signature).is_ok());
    }
    
    Outputs {
        hash_output: message_hash,
        num_iterations: repeat,
    }
}

fn execute_smt_proof(input: &[u8]) -> Outputs {
    // decode inputs
    let repeat = u32::from_le_bytes([input[1], input[2], input[3], input[4]]);
    let mut merkle_root = [0u8; 32];
    merkle_root.copy_from_slice(&input[5..37]);

    // validate proofs
    for i in 0..(repeat as usize) {
        let mut index = [0u8; 32];
        index.copy_from_slice(&input[(i * 8224 + 37)..(i * 8224 + 69)]);
        let mut leaf = [0u8; 32];
        leaf.copy_from_slice(&input[(i * 8224 + 69)..(i * 8224 + 101)]);

        let mut proof = [[0u8; 32]; 255];
        for j in 0..255 {
            proof[j].copy_from_slice(&input[((i * 8224) + (j * 32) + 101)..((i * 8224) + (j * 32) + 133)]);
        }

        let root = root_from_merkle_proof(&leaf, &index, &proof);
        assert!(merkle_root == root);
    }
    
    Outputs {
        hash_output: merkle_root,
        num_iterations: repeat,
    }
}

fn root_from_merkle_proof(leaf: &[u8; 32], index: &[u8; 32], proof: &[[u8; 32]; 255]) -> [u8; 32] {
    let mut idx = index.clone();
    let mut root = leaf.clone();
    for i in 0..255 {
        let mut combined_digest: [u8; 64] = [0; 64];
        if (idx[31] & 1) == 0 {
            combined_digest[0..32].copy_from_slice(&root);
            combined_digest[32..64].copy_from_slice(&proof[i]);
        } else {
            combined_digest[0..32].copy_from_slice(&proof[i]);
            combined_digest[32..64].copy_from_slice(&root);
        }
        root = sha256_hash(&combined_digest);
        idx = shr_index(&idx);
    }
    root
}

fn shr_index(index: &[u8; 32]) -> [u8; 32] {
    let mut shifted = [0u8; 32];
    for i in (1..31).rev() {
        shifted[i] = (index[i] >> 1) + (index[i-1] << 7);
    }
    shifted[0] = index[0] >> 1;
    shifted
}

fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}
