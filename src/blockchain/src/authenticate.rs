//! Signature verification using Dilithium2 public-key cryptography.

use crate::states::SharedData;
use fleetcore::{BaseJournal, CommunicationData, SignedMessage};
use pqcrypto_dilithium::dilithium2::{verify_detached_signature, DetachedSignature, PublicKey};
use pqcrypto_traits::sign::{DetachedSignature as _, PublicKey as _};
use serde::{Deserialize, Serialize};

/// Verifies authenticity and consistency of a signed message.
///
/// # Arguments
/// - `shared`: Shared blockchain state
/// - `signed`: The signed message, including payload, signature, and public key
///
/// # Returns
/// - `Ok(())` if valid
/// - `Err(msg)` if invalid
pub fn authenticate(
    shared: &SharedData,
    signed: &SignedMessage<CommunicationData>,
) -> Result<(), String> {
    let payload = &signed.payload;
    let signature = &signed.signature;
    let public_key = &signed.public_key;

    let message_bytes =
        serde_json::to_vec(payload).map_err(|_| "Failed to serialize payload".to_string())?;

    if !verify_signature(&message_bytes, signature, public_key) {
        return Err("Invalid signature".to_string());
    }

    let journal: BaseJournal = payload
        .receipt
        .journal
        .decode()
        .expect("Invalid journal data");

    let gmap = shared.gmap.lock().unwrap();
    if let Some(game) = gmap.get(&journal.gameid) {
        if let Some(player) = game.pmap.get(&journal.fleet) {
            if player.public_key != *public_key {
                return Err(format!("Public key mismatch for player {}", journal.fleet));
            }
        }
    }

    Ok(())
}

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
