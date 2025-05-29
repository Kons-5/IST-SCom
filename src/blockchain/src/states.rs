use risc0_zkvm::Digest;
use std::{collections::HashMap, sync::{Arc, Mutex}};
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct SharedData {
    pub tx: broadcast::Sender<String>,
    pub gmap: Arc<Mutex<HashMap<String, Game>>>,
    pub rng: Arc<Mutex<rand::rngs::StdRng>>,
}

pub struct Player {
    pub name: String,                       // Player ID
    pub current_state: Digest,              // Commitment hash
}

pub struct Game {
    pub pmap: HashMap<String, Player>,      // All players in the game
    pub shot_position: u8,                  // Last shot position
    pub player_order: Vec<String>,          // Player order
    pub next_player: Option<String>,        // player allowed to fire
    pub next_report: Option<String>,        // player expected to report

}
