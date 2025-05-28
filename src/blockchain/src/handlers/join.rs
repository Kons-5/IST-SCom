use crate::{xy_pos, Game, Player, SharedData};
use fleetcore::{BaseJournal, CommunicationData, SignedMessage};
use methods::JOIN_ID;

use std::{
    collections::HashMap,
    error::Error,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

pub fn handle_join(shared: &SharedData, input_data: &CommunicationData) -> String {
    if input_data.receipt.verify(JOIN_ID).is_err() {
        shared
            .tx
            .send("Attempting to join game with invalid receipt".to_string())
            .unwrap();
        return "Could not verify receipt".to_string();
    }
    // Decode journal
    let data: BaseJournal = input_data.receipt.journal.decode().unwrap();

    // Access game state
    // Look up the game by its ID. If it already exists, get a mutable reference to it.
    // If it doesn't exist, insert a new Game struct
    let mut gmap = shared.gmap.lock().unwrap();
    let game = gmap.entry(data.gameid.clone()).or_insert(Game {
        pmap: HashMap::new(),
        next_player: Some(data.fleet.clone()), // first to join = first to shoot
        next_report: None,                     // No shots fired = No player to report
    });

    // Handle duplicate player
    if game.pmap.contains_key(&data.fleet) {
        let msg = format!(
            "Player \"{}\" is already in game \"{}\". Current players: [{}]\n\n\n\
            \x20",
            data.fleet,
            data.gameid,
            game.pmap.keys().cloned().collect::<Vec<_>>().join(", ")
        );
        // Check wheter it is expected to register invalid actions
        // shared.tx.send(msg.clone()).unwrap();
        return msg;
    }

    // Register the player in the game under their fleet ID (if not duplicate)
    game.pmap.insert(
        data.fleet.clone(),
        Player {
            name: data.fleet.clone(),
            current_state: data.board.clone(),
        },
    );

    // Create unified success message
    let players: Vec<String> = game.pmap.keys().cloned().collect();
    let msg = format!(
        "\
        \x20 Join receipt decoded:\n\
        \x20 ▶ Game ID: {}\n\
        \x20 ▶ Fleet ID: {}\n\
        \x20 ▶ Commitment Hash: {:?}\n\n\
        \x20 Player \"{}\" joined game \"{}\".\n\
        \x20 ▶ Total players: {}\n\
        \x20 ▶ Current players: [{}]\n\n\n\
        \x20",
        data.gameid,
        data.fleet,
        data.board,
        data.fleet,
        data.gameid,
        players.len(),
        players.join(", ")
    );
    let html_msg = msg.replace('\n', "<br>");
    shared.tx.send(html_msg.clone()).unwrap();
    "OK".to_string()
}
