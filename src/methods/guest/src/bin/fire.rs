use fleetcore::{FireInputs, FireJournal};
use proofs::hash_board;
use risc0_zkvm::guest::env;
use risc0_zkvm::sha::Digest;
use sha2::{Digest as ShaDigest, Sha256};

fn main() {
    // Read the input
    let input: FireInputs = env::read();

    // Validate that the fleet is NOT fully sunk
    assert!(
        !input.board.is_empty(),
        "Your fleet is fully sunk. Cannot fire!"
    );

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

    // Commitment hash: Hash(nonce || board)
    let board_hash = hash_board(&input.board, &input.random);

    // Build fire journal
    let output = FireJournal {
        gameid: input.gameid,
        fleet: input.fleet,
        target: input.target,
        pos: input.pos,
        board: board_hash,
        token_commitment: token_hash.expect("Token hash missing"),
    };

    // Write public output to the journal
    env::commit(&output);
}
