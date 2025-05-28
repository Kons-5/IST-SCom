//! Signature verification using Dilithium2 public-key cryptography.

use pqcrypto_dilithium::dilithium2::{verify_detached_signature, DetachedSignature, PublicKey};
use pqcrypto_traits::sign::{DetachedSignature as _, PublicKey as _};
use serde::{Deserialize, Serialize};

/// Verifies a Dilithium signature.
///
/// # Arguments
/// * `message` - The original message
/// * `signature` - Signature bytes
/// * `public_key_bytes` - Public key bytes
///
/// # Returns
/// `true` if valid, `false` otherwise
pub fn verify_signature(message: &[u8], signature: &[u8], public_key_bytes: &[u8]) -> bool {
    let pk = match PublicKey::from_bytes(public_key_bytes) {
        Ok(pk) => pk,
        Err(_) => return false,
    };

    let sig = match DetachedSignature::from_bytes(signature) {
        Ok(sig) => sig,
        Err(_) => return false,
    };

    verify_detached_signature(&sig, message, &pk).is_ok()
}
