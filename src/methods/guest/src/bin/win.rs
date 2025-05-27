use fleetcore::{BaseInputs, BaseJournal};
use risc0_zkvm::guest::env;
//use risc0_zkvm::Digest;
//use sha2::{Digest as _, Sha256};

fn main() {

    // read the input
    let _input: BaseInputs = env::read();

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
