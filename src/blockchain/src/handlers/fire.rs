use crate::{rotate_player_to_back, xy_pos, Game, Player, SharedData};
use fleetcore::{CommunicationData, FireJournal};
use methods::FIRE_ID;

use std::{
    collections::HashMap,
    error::Error,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

pub fn handle_fire(
    shared: &SharedData,
    input_data: &CommunicationData,
    public_key: &[u8],
) -> String {
    if input_data.receipt.verify(FIRE_ID).is_err() {
        shared
            .tx
            .send("Attempting to fire with invalid receipt".to_string())
            .unwrap();
        return "Could not verify receipt".to_string();
    }

    // Decode journal
    let data: FireJournal = input_data.receipt.journal.decode().unwrap();

    // Confirm game exists
    let mut gmap = shared.gmap.lock().unwrap();
    let game = match gmap.get_mut(&data.gameid) {
        Some(g) => g,
        None => {
            return format!(
                "Game {} not found\n\n\n\
         \x20",
                data.gameid
            )
        }
    };

    // Confirm firing player exists and is valid
    let player = match game.pmap.get(&data.fleet) {
        Some(p) => p,
        None => {
            return format!(
                "Player {} not found\n\n\n\
         \x20",
                data.fleet
            )
        }
    };

    // Confirm public key matches
    if player.public_key != public_key {
        return format!("Public key mismatch for player {}", data.fleet);
    }

    // Validate commitment hash
    if data.board != player.current_state {
        return "Fleet commitment does not match recorded state\n\n\n\
         \x20"
            .to_string();
    }

    // Validate player's turn
    if game.next_player.as_ref() != Some(&data.fleet) {
        // Check if this player is expected to report
        if game.next_report.as_ref() == Some(&data.fleet) {
            return format!(
                "It's not {}'s turn to fire: you must report the last shot before firing.\n\n\n\
                \x20",
                data.fleet
            );
        } else {
            return format!(
                "It's not {}'s turn to fire\n\n\n\
            \x20",
                data.fleet
            );
        }
    }

    // Validate target's existence
    if !game.pmap.contains_key(&data.target) {
        return format!(
            "Target {} does not exist\n\n\n\
        \x20",
            data.target
        );
    }

    // Update game state
    game.next_player = None;
    game.next_report = Some(data.target.clone());
    game.shot_position = data.pos.clone();

    // Rotate current player to back of queue
    rotate_player_to_back(game, &data.fleet);

    let msg = format!(
        "\
        \x20 Shots fired!\n\
        \x20 ▶ {} fired at position {} targeting {} in game {}\n\n\n\
        \x20",
        data.fleet,
        xy_pos(data.pos),
        data.target,
        data.gameid
    );

    let html_msg = msg.replace('\n', "<br>");
    shared.tx.send(html_msg.clone()).unwrap();
    html_msg
}
