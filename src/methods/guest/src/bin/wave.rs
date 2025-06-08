use fleetcore::{BaseInputs, BaseJournal};
use proofs::hash_board;
use risc0_zkvm::guest::env;
use risc0_zkvm::sha::Digest;
use sha2::{Digest as ShaDigest, Sha256};

fn main() {
    let input: BaseInputs = env::read();

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

    // Compute commitment: H(nonce || board)
    let board_hash = hash_board(&input.board, &input.random);

    // Commit public output
    let output = BaseJournal {
        gameid: input.gameid,
        fleet: input.fleet,
        board: board_hash,
        token_commitment: token_hash.expect("Token hash missing"),
    };

    env::commit(&output);
}
