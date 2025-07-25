use risc0_zkvm::Digest;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct SharedData {
    pub tx: broadcast::Sender<String>,
    pub gmap: Arc<Mutex<HashMap<String, Game>>>,
    pub rng: Arc<Mutex<rand::rngs::StdRng>>,
}

pub struct Player {
    pub name: String,          // Player ID
    pub current_state: Digest, // Commitment hash
    pub public_key: Vec<u8>,   // Dilithium public key
    pub rsa_pubkey: Vec<u8>,   // Token RSA public key
}

pub struct PendingWin {
    pub claimant: String, // Fleet ID that claimed win
    pub board: Digest,    // Committed board hash
    pub time: Instant,    // Time when claim was made
}

pub struct Game {
    pub pmap: HashMap<String, Player>,   // All players in the game
    pub shot_position: Option<u8>,       // Last shot position
    pub pending_win: Option<PendingWin>, // If someone has claimed victory

    // Token authentication
    pub turn_commitment: Option<Digest>,
    pub encrypted_token: Option<String>,
}
