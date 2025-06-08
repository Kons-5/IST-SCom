use crate::{xy_pos, Game, Player, SharedData};
use fleetcore::{BaseJournal, CommunicationData, EncryptedToken};
use methods::WAVE_ID;

use std::{
    collections::HashMap,
    error::Error,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

pub fn handle_wave(
    shared: &SharedData,
    input_data: &CommunicationData,
    public_key: &[u8],
) -> String {
    if input_data.receipt.verify(WAVE_ID).is_err() {
        shared
            .tx
            .send("Attempting to wave game with invalid receipt".to_string())
            .unwrap();
        return "Could not verify receipt".to_string();
    }
    // Decode journal
    let data: BaseJournal = input_data.receipt.journal.decode().unwrap();

    // Confirm game exists
    let mut gmap = shared.gmap.lock().unwrap();
    let game = match gmap.get_mut(&data.gameid) {
        Some(g) => g,
        None => return format!("Game {} not found", data.gameid),
    };

    // Validate player's turn
    if game.turn_commitment != Some(data.token_commitment) {
        return "Invalid token: not your turn.".to_string();
    }

    // Verify if the player has reported before firing
    if game.shot_position.is_some() {
        return "You must report the last shot before firing.\n".to_string();
    }

    // Confirm firing player exists and is valid
    let player = match game.pmap.get_mut(&data.fleet) {
        Some(p) => p,
        None => return format!("Player {} not found in game {}", data.fleet, data.gameid),
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

    // Update turn order
    let token_data: &EncryptedToken = input_data.token_data.as_ref().unwrap();
    game.encrypted_token = Some(token_data.enc_token.clone());
    game.turn_commitment = Some(token_data.token_hash.clone());

    // Build message
    let recipient = input_data
        .token_data
        .as_ref()
        .and_then(|t| {
            game.pmap
                .iter()
                .find(|(_, p)| p.rsa_pubkey == t.pub_rsa_key)
                .map(|(id, _)| id)
        })
        .cloned()
        .unwrap_or_else(|| "(unknown recipient)".to_string());

    let msg = format!(
        "\
        \x20 {} waved their turn.\n\
        \x20 ▶ Token passed to: {}\n\n\n\
        \x20",
        data.fleet, recipient,
    );

    let html_msg = msg.replace('\n', "<br>");
    shared.tx.send(html_msg.clone()).unwrap();

    "OK".to_string()
}
