use crate::{xy_pos, Game, Player, SharedData};
use fleetcore::{CommunicationData, EncryptedToken, FireJournal};
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
        None => return format!("Game {} not found\n", data.gameid),
    };

    if game.turn_commitment != Some(data.token_commitment) {
        return "Invalid token: not your turn.\n".to_string();
    }

    // Verify if the player has reported before firing
    if game.shot_position.is_some() {
        return "You must report the last shot before firing.\n".to_string();
    }

    // Confirm firing player exists and is valid
    let player = match game.pmap.get(&data.fleet) {
        Some(p) => p,
        None => return format!("Player {} not found\n", data.fleet),
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

    // Validate target's existence
    if !game.pmap.contains_key(&data.target) {
        return format!(
            "Target {} does not exist\n\n\n\
        \x20",
            data.target
        );
    }

    // Update game state with new token
    let token_data: &EncryptedToken = input_data.token_data.as_ref().unwrap();
    game.encrypted_token = Some(token_data.enc_token.clone());
    game.turn_commitment = Some(token_data.token_hash.clone());
    game.shot_position = Some(data.pos);

    let msg = format!(
        "\
        \x20 Shots fired!\n\
        \x20 ▶ {} fired at position {} targeting {} in game {}\n\n\n\
        \x20",
        data.fleet,
        xy_pos(Some(data.pos)),
        data.target,
        data.gameid
    );

    let html_msg = msg.replace('\n', "<br>");
    shared.tx.send(html_msg.clone()).unwrap();

    "OK".to_string()
}
