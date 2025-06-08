//! Logic for the token generation using RSA.

use rsa::pkcs1v15::Pkcs1v15Encrypt;
use rsa::{
    pkcs8::{DecodePublicKey, EncodePrivateKey, EncodePublicKey},
    RsaPrivateKey, RsaPublicKey,
};

use base64::{engine::general_purpose, Engine as _};
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest as ShaDigest, Sha256};

/// Encrypts a given 32-byte token using a base64-encoded RSA public key.
/// Returns the encrypted token (base64) and the SHA-256 hash of the original token.
pub fn prepare_turn_token(rsa_pubkey_base64: &str, token_b64: &str) -> Option<(String, [u8; 32])> {
    // Decode and parse RSA key
    let rsa_bytes = import_key_base64(rsa_pubkey_base64);
    let rsa_pem = String::from_utf8(rsa_bytes).ok()?;
    let rsa_pub = RsaPublicKey::from_public_key_pem(&rsa_pem).ok()?;

    // Compute hash of token (commitment)
    let token = import_key_base64(token_b64);
    let token_hash = Sha256::digest(&token);
    let token_hash_array = <[u8; 32]>::try_from(token_hash).ok()?;

    // Encrypt token to recipient
    let enc = rsa_pub.encrypt(&mut OsRng, Pkcs1v15Encrypt, &token).ok()?;

    let enc_token = export_key_base64(&enc);
    Some((enc_token, token_hash_array))
}

/// Generates a random 32-byte token and returns it as a base64 string.
pub fn generate_raw_token_base64() -> String {
    let mut token = [0u8; 32];
    OsRng.fill_bytes(&mut token);
    let token_b64 = general_purpose::STANDARD.encode(token);
    token_b64
}

/// Generates an RSA keypair (2048-bit)
pub fn generate_rsa_keypair() -> (Vec<u8>, Vec<u8>) {
    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate RSA key");
    let public_key = RsaPublicKey::from(&private_key);

    let priv_pem = private_key.to_pkcs8_pem(Default::default()).unwrap();
    let pub_pem = public_key.to_public_key_pem(Default::default()).unwrap();

    (priv_pem.as_bytes().to_vec(), pub_pem.as_bytes().to_vec())
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
