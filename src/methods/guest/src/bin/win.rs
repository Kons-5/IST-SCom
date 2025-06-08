use fleetcore::{BaseInputs, BaseJournal};
use proofs::hash_board;
use risc0_zkvm::guest::env;
use risc0_zkvm::sha::Digest;

fn main() {
    // Read the input
    let input: BaseInputs = env::read();

    // Validate that the fleet is NOT fully sunk
    assert!(!input.board.is_empty(), "Your fleet is fully sunk...");

    // Compute commitment: H(nonce || board)
    let digest = hash_board(&input.board, &input.random);

    // Commit public output
    let output = BaseJournal {
        gameid: input.gameid,
        fleet: input.fleet,
        board: digest,
        token_commitment: Digest::default(), // null
    };

    env::commit(&output);
}
