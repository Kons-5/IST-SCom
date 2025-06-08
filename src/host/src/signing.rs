//! Post-quantum digital signature logic using Dilithium.

use base64::{engine::general_purpose, Engine as _};
use fleetcore::SignedMessage;
use pqcrypto_dilithium::dilithium2::{detached_sign, keypair, PublicKey, SecretKey};
use pqcrypto_traits::sign::{DetachedSignature as _, PublicKey as _, SecretKey as _};

/// Signs a serializable payload and returns a `SignedMessage`.
pub fn sign_payload<T: serde::Serialize>(
    payload: T,
    pubkey_b64: &str,
    privkey_b64: &str,
) -> Option<SignedMessage<T>> {
    let pk = import_key_base64(pubkey_b64);
    let sk = import_key_base64(privkey_b64);

    let payload_bytes = serde_json::to_vec(&payload).ok()?;
    let signature = sign_message(&payload_bytes, &sk);

    Some(SignedMessage {
        payload,
        signature,
        public_key: pk,
    })
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
    let sk = SecretKey::from_bytes(private_key_bytes).expect("Invalid private key");
    detached_sign(message, &sk).as_bytes().to_vec()
}

/// Generates a Dilithium keypair.
///
/// # Returns
/// A tuple containing the private and public keys as byte vectors.
pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    let (pk, sk) = keypair();
    (sk.as_bytes().to_vec(), pk.as_bytes().to_vec())
}

/// Exports a key as base64.
pub fn export_key_base64(key: &[u8]) -> String {
    general_purpose::STANDARD.encode(key)
}

/// Import a key from base64.
pub fn import_key_base64(key: &str) -> Vec<u8> {
    general_purpose::STANDARD
        .decode(key)
        .expect("Invalid Base64 key")
}
