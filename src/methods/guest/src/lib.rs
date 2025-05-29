use risc0_zkvm::sha::Digest;
use sha2::{Digest as ShaDigest, Sha256};

pub mod validate;

/// Computes a commitment hash for a board and nonce.
///
/// # Arguments
/// - `board`: The fleet position as a byte array
/// - `nonce`: The secret nonce
///
/// # Returns
/// - A SHA-256 hash of `nonce || board`
pub fn hash_board(board: &[u8], nonce: &str) -> Digest {
    let mut hasher = Sha256::new();
    hasher.update(nonce.as_bytes());
    hasher.update(board);
    Digest::try_from(hasher.finalize().as_slice()).expect("Hash size mismatch")
}
