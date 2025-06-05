use crate::{xy_pos, Game, Player, SharedData};
use fleetcore::{BaseJournal, CommunicationData, SignedMessage};
use methods::JOIN_ID;

use std::{
    collections::HashMap,
    error::Error,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

pub fn handle_join(
    shared: &SharedData,
    input_data: &CommunicationData,
    public_key: &[u8],
) -> String {
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
        shot_position: None,
        player_order: vec![data.fleet.clone()],
        next_player: Some(data.fleet.clone()), // First to join = First to shoot
        next_report: None,                     // No shots fired = No player to report
        pending_win: None,
    });

    if !game.shot_position.is_none() {
        return format!(
            "Trying to join an ongoing game (\"{}\").\n Current players: [{}]\n\n\n\
              \x20",
            data.gameid,
            game.pmap.keys().cloned().collect::<Vec<_>>().join(", ")
        );
    }

    // Handle duplicate player
    if let Some(existing_player) = game.pmap.get(&data.fleet) {
        if existing_player.public_key != public_key {
            return format!(
                "Public key mismatch for player \"{}\" in game \"{}\"",
                data.fleet, data.gameid
            );
        }

        return format!(
            "Player \"{}\" is already in game \"{}\". Current players: [{}]\n\n\n\
                \x20",
            data.fleet,
            data.gameid,
            game.pmap.keys().cloned().collect::<Vec<_>>().join(", ")
        );
    }

    // Register the player in the game under their fleet ID (if not duplicate)
    game.pmap.insert(
        data.fleet.clone(),
        Player {
            name: data.fleet.clone(),
            current_state: data.board.clone(),
            public_key: public_key.to_vec(),
        },
    );

    // Create success message
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
