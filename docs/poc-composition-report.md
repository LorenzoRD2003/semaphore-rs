# Short Report: How the Two PoCs Relate

This repository now contains two separate proof-of-concept binaries:

- [identity_signature_low_order_forgery.rs](/home/lorenzo/Desktop/semaphore-rs/bin/identity_signature_low_order_forgery.rs)
- [semaphore_zero_secret_membership.rs](/home/lorenzo/Desktop/semaphore-rs/bin/semaphore_zero_secret_membership.rs)

They are related, but they do **not** show the same bug.

## 1. Invalid-public-key signature forgery

The first PoC targets the signature API in [src/identity.rs](/home/lorenzo/Desktop/semaphore-rs/src/identity.rs).

It shows that `Signature::verify()` accepts forged signatures when the verifier is given the Edwards identity point as a public key. In that case, the verification equation collapses and the signature no longer proves knowledge of any meaningful secret.

This is an **API-level bug**:

- it affects consumers that accept externally supplied Baby Jubjub public keys,
- it does not directly affect the main Semaphore proof flow in this repository.

## 2. Degenerate Semaphore identity with `secret = 0`

The second PoC targets the Semaphore proof system itself.

It shows that the circuit accepts `secret = 0`, which implies:

- public key = Edwards identity point `(0, 1)`,
- identity commitment = `Poseidon(0, 1)`,
- and therefore a publicly known witness for that identity.

If that commitment is ever admitted into the group, then anyone can produce a valid Semaphore proof for that member.

This is a **protocol-level issue**:

- it is not about signature verification,
- it is about the set of identities that the proof system is willing to accept.

## 3. Can the two PoCs be combined?

Yes, but only under a particular enrollment model.

They compose if an application does something like this:

1. receives a Baby Jubjub public key from a user,
2. uses the signature API to “prove possession” of that key,
3. computes the Semaphore identity commitment from that public key,
4. inserts that commitment into the Semaphore group.

Under that model:

- the first PoC can be used to fake possession of the identity point public key,
- the application may then admit the degenerate commitment `Poseidon(0, 1)`,
- and the second PoC becomes a full protocol break for that enrolled member, because anyone knows the witness `secret = 0`.

## 4. When they do not compose

They do **not** automatically compose in the current Rust crate's honest path.

The reason is:

- [Identity::new()](/home/lorenzo/Desktop/semaphore-rs/src/identity.rs:31) does not generate `secret = 0`,
- and the main proof flow in [src/proof.rs](/home/lorenzo/Desktop/semaphore-rs/src/proof.rs:121) takes an `Identity`, not an arbitrary public key.

So the repository does not currently expose a one-call path from the signature API PoC to the proof-system PoC.

## 5. Practical takeaway

The most serious combined risk is not “cofactor clearing” alone. It is:

- weak validation for externally supplied public keys,
- plus a proof circuit that accepts a degenerate identity with a publicly known witness.

If a surrounding system uses the signature API as part of Semaphore member enrollment, these two issues can become one end-to-end exploit chain.
