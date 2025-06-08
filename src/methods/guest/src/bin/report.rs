//! zkVM guest program for verifying a Report note in Battleship game
//!
//! This proof verifies whether a player truthfully responded to an opponent’s shot,
//! and commits both the previous and updated board hashes in the output journal.

use fleetcore::{FireInputs, ReportJournal};
use proofs::hash_board;
use risc0_zkvm::guest::env;
use risc0_zkvm::sha::Digest;
use sha2::{Digest as ShaDigest, Sha256};

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

/// Handles a reported "Miss" outcome.
///
/// Verifies that the shot position does not exist in the board (i.e., no hit occurred).
/// Since the board is unchanged, the same hash is committed for both original and updated states.
///
/// # Panics
/// - If the position is found in the board (which contradicts a "Miss" report)
fn handle_miss(input: &FireInputs) -> ReportJournal {
    // Validate token ownership
    let token_hash: Option<Digest> = input.token_auth.as_ref().map(|auth| {
        let hash = Sha256::digest(&auth.token);
        let digest = Digest::try_from(hash.as_slice()).expect("Invalid hash size");
        assert_eq!(
            &digest, &auth.expected_hash,
            "Token mismatch: you do not own the turn"
        );
        digest
    });

    // Position must not be present in the board
    assert!(
        !input.board.contains(&input.pos),
        "Claimed miss, but target position was a hit"
    );

    // Hash board once
    // Same for both original board and updated board
    let board_hash = hash_board(&input.board, &input.random);

    ReportJournal {
        gameid: input.gameid.clone(),
        fleet: input.fleet.clone(),
        report: input.target.clone(),
        pos: input.pos,
        board: board_hash.clone(),
        next_board: board_hash,
        token_commitment: token_hash.expect("Token hash missing"),
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
    // Validate token ownership
    let token_hash: Option<Digest> = input.token_auth.as_ref().map(|auth| {
        let hash = Sha256::digest(&auth.token);
        let digest = Digest::try_from(hash.as_slice()).expect("Invalid hash size");
        assert_eq!(
            &digest, &auth.expected_hash,
            "Token mismatch: you do not own the turn"
        );
        digest
    });

    // Position must no longer be present in the board
    assert!(
        input.board.contains(&input.pos),
        "Claimed hit, but position still in updated board"
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
        token_commitment: token_hash.expect("Token hash missing"),
    }
}
