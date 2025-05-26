//! zkVM guest program for verifying a Report note in Battleship game
//!
//! This proof verifies whether a player truthfully responded to an opponent’s shot,
//! and commits both the previous and updated board hashes in the output journal.

use fleetcore::{FireInputs, ReportJournal};

use risc0_zkvm::guest::env;
use risc0_zkvm::sha::Digest;
use sha2::{Sha256, Digest as ShaDigest};

/// Entry point for the zkVM guest program.
///
/// Reads the input, determines whether the player reported a "Hit" or "Miss",
/// and commits a `ReportJournal` with the verified board transition.
fn main() {
    let input: FireInputs = env::read();

    // Validate and compute based on report type
    let journal = match input.target.as_str() {
        "Hit" => handle_hit(&input),
        "Miss" => handle_miss(&input),
        _ => panic!("Invalid report value: {}", input.target),
    };

    env::commit(&journal);
}

/// Computes a commitment hash for a board and nonce.
///
/// # Parameters
/// - `board`: The fleet
/// - `nonce`: The secret nonce used in board commitments
///
/// # Returns
/// - `Digest`: The SHA-256 hash of `nonce || board`
fn hash_board(board: &[u8], nonce: &str) -> Digest {
    // Commitment hash: Hash(nonce || board)
    let mut hasher = Sha256::new();
    hasher.update(nonce.as_bytes());
    hasher.update(board);
    Digest::try_from(hasher.finalize().as_slice()).expect("Hash size mismatch")
}


/// Handles a reported "Miss" outcome.
///
/// Verifies that the shot position does not exist in the board (i.e., no hit occurred).
/// Since the board is unchanged, the same hash is committed for both original and updated states.
///
/// # Panics
/// - If the position is found in the board (which contradicts a "Miss" report)
fn handle_miss(input: &FireInputs) -> ReportJournal {
    // Position must not be present in the board
    assert!(
        !input.board.contains(&input.pos),
        "Claimed Miss, but target position was a hit"
    );

    // Hash board once
    // Same for both original board and updated board
    let hash = hash_board(&input.board, &input.random);

    ReportJournal {
        gameid: input.gameid.clone(),
        fleet: input.fleet.clone(),
        report: input.target.clone(),
        pos: input.pos,
        board: hash.clone(),
        next_board: hash,
    }
}

/// Handles a reported "Hit" outcome.
///
/// Verifies that the position was removed from the board, then reconstructs the
/// original board state and computes both the old and new commitment hashes.
///
/// # Panics
/// - If the position is still present in the updated board (shot not applied)
fn handle_hit(input: &FireInputs) -> ReportJournal {
    // Position must no longe be present in the board
    assert!(
        !input.board.contains(&input.pos),
        "Claimed Hit, but position still in updated board"
    );

    // Reconstruct original board
    let mut original_board = input.board.clone();
    original_board.push(input.pos);
    original_board.sort();

    // Hash original board
    // This is the committed board before the shot
    let board_hash = hash_board(&original_board, &input.random);

    // Hash next board
    // This is the updated board after the shot
    let next_board_hash = hash_board(&input.board, &input.random);

    ReportJournal {
        gameid: input.gameid.clone(),
        fleet: input.fleet.clone(),
        report: input.target.clone(),
        pos: input.pos,
        board: board_hash,
        next_board: next_board_hash,
    }
}
