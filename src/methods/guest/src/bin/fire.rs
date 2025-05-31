use fleetcore::{FireInputs, FireJournal};

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

    // Commitment hash: Hash(nonce || board)
    let mut hasher = Sha256::new();
    hasher.update(input.random.as_bytes());
    hasher.update(&input.board);
    let hash_result = hasher.finalize();

    let commitment = Digest::try_from(hash_result.as_slice()).expect("Hash size mismatch");

    // 4. Build fire journal
    let output = FireJournal {
        gameid: input.gameid,
        fleet: input.fleet,
        target: input.target,
        pos: input.pos,
        board: commitment,
    };

    // write public output to the journal
    env::commit(&output);
}
