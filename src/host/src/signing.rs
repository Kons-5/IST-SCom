//! Post-quantum digital signature logic using Dilithium.

use base64::{engine::general_purpose, Engine as _};
use pqcrypto_dilithium::dilithium2::{detached_sign, keypair, PublicKey, SecretKey};
use pqcrypto_traits::sign::{DetachedSignature as _, PublicKey as _, SecretKey as _};

/// Generates a Dilithium keypair.
///
/// # Returns
/// A tuple containing the private and public keys as byte vectors.
pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    let (pk, sk) = keypair();
    (sk.as_bytes().to_vec(), pk.as_bytes().to_vec())
}

/// Signs a message with a Dilithium private key.
///
/// # Arguments
/// * `message` - The message to sign
/// * `private_key_bytes` - Byte slice of the private key
///
/// # Returns
/// Signature as a byte vector
pub fn sign_message(message: &[u8], private_key_bytes: &[u8]) -> Vec<u8> {
    let sk = SecretKey::from_bytes(private_key_bytes).expect("invalid private key");
    detached_sign(message, &sk).as_bytes().to_vec()
}

/// Exports the public key as base64.
pub fn export_public_key_base64(public_key: &[u8]) -> String {
    general_purpose::STANDARD.encode(public_key)
}
