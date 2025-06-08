// methods/guest/src/bin/contest.rs

use fleetcore::{BaseInputs, BaseJournal};
use proofs::hash_board;
use risc0_zkvm::guest::env;
use risc0_zkvm::sha::Digest;

fn main() {
    let input: BaseInputs = env::read();

    // Check that the fleet still has ships (i.e., contest is valid)
    assert!(
        !input.board.is_empty(),
        "You cannot contest with an empty fleet"
    );

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
