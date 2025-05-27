use fleetcore::{BaseInputs, BaseJournal};
use proofs::hash_board;

use risc0_zkvm::guest::env;

fn main() {
    let input: BaseInputs = env::read();

    let digest = hash_board(&input.board, &input.random);

    let output = BaseJournal {
        gameid: input.gameid,
        fleet: input.fleet,
        board: digest,
    };

    env::commit(&output);
}
