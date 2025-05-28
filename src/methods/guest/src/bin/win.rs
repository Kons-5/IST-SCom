use fleetcore::{BaseInputs, BaseJournal};
use proofs::hash_board;

use risc0_zkvm::guest::env;

fn main() {
    // Read the input
    let input: BaseInputs = env::read();

    // Validate that the fleet is NOT fully sunk
    assert!(
        !input.board.is_empty(),
        "Your fleet is fully sunk — cannot fire"
    );

    let digest = hash_board(&input.board, &input.random);

    let output = BaseJournal {
        gameid: input.gameid,
        fleet: input.fleet,
        board: digest,
    };

    env::commit(&output);
}
