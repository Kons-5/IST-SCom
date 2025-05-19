use fleetcore::{BaseInputs, BaseJournal};
use risc0_zkvm::guest::env;
use risc0_zkvm::sha::Digest;
use sha2::{Sha256, Digest as ShaDigest};

fn main() {
    let input: BaseInputs = env::read();

    // Hash the (nonce || board)
    let mut hasher = Sha256::new();
    hasher.update(input.random.as_bytes());  // nonce
    hasher.update(&input.board);
    let hash_result = hasher.finalize();

    // Convert to Digest
    let commitment = Digest::try_from(hash_result.as_slice()).expect("Hash size mismatch");

    // Output the result to the journal
    let output = BaseJournal {
        gameid: input.gameid,
        fleet: input.fleet,
        board: commitment,
    };
    env::commit(&output);
}
