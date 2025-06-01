//! Post-quantum digital signature logic using Dilithium.
//!

use base64::{engine::general_purpose, Engine as _};

// -----------------------------------------------------------------------------
// DILITHIUM SIGNATURE
// -----------------------------------------------------------------------------
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
    let sk = SecretKey::from_bytes(private_key_bytes).expect("Invalid private key");
    detached_sign(message, &sk).as_bytes().to_vec()
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

// -----------------------------------------------------------------------------
// RSA
// -----------------------------------------------------------------------------
use rsa::{pkcs8::{EncodePrivateKey, EncodePublicKey, DecodePublicKey}, RsaPrivateKey, RsaPublicKey};
use rsa::pkcs1v15::Pkcs1v15Encrypt;

use rand::{rngs::OsRng, RngCore};
use sha2::{Digest as ShaDigest, Sha256};

/// Generates an RSA keypair (2048-bit)
pub fn generate_rsa_keypair() -> (Vec<u8>, Vec<u8>) {
    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate RSA key");
    let public_key = RsaPublicKey::from(&private_key);

    let priv_pem = private_key.to_pkcs8_pem(Default::default()).unwrap();
    let pub_pem = public_key.to_public_key_pem(Default::default()).unwrap();

    (priv_pem.as_bytes().to_vec(), pub_pem.as_bytes().to_vec())
}

pub fn prepare_turn_token(rsa_pubkey_base64: &str) -> Option<(String, [u8; 32])> {
    // Decode and parse RSA key
    let rsa_bytes = import_key_base64(rsa_pubkey_base64);
    let rsa_pem = String::from_utf8(rsa_bytes).ok()?;
    let rsa_pub = RsaPublicKey::from_public_key_pem(&rsa_pem).ok()?;

    // Generate random r
    let mut r = [0u8; 32];
    OsRng.fill_bytes(&mut r);

    // Compute hash of r (the commitment)
    let r_hash = Sha256::digest(&r);
    let r_hash_array = <[u8; 32]>::try_from(r_hash).ok()?;

    // Encrypt r to recipient
    let enc = rsa_pub.encrypt(&mut OsRng, Pkcs1v15Encrypt, &r).ok()?;

    let enc_token = export_key_base64(&enc);
    Some((enc_token, r_hash_array))
}
