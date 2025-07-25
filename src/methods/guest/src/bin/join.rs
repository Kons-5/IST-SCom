use fleetcore::{BaseInputs, BaseJournal};
use proofs::hash_board;
use proofs::validate::validate_battleship_board;
use risc0_zkvm::guest::env;
use risc0_zkvm::sha::Digest;

fn main() {
    // Read input from host
    let input: BaseInputs = env::read();

    // Validate fleet configuration
    if !validate_battleship_board(&input.board) {
        panic!("Invalid fleet configuration");
    }

    // Compute commitment: H(nonce || board)
    let board_hash = hash_board(&input.board, &input.random);

    // Commit public output
    let output = BaseJournal {
        gameid: input.gameid,
        fleet: input.fleet,
        board: board_hash,
        token_commitment: Digest::default(), // null
    };

    env::commit(&output);
}
