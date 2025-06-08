use crate::{xy_pos, Game, Player, SharedData};
use fleetcore::{CommunicationData, EncryptedToken, ReportJournal, SignedMessage};
use methods::REPORT_ID;

use std::{
    collections::HashMap,
    error::Error,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

pub fn handle_report(
    shared: &SharedData,
    input_data: &CommunicationData,
    public_key: &[u8],
) -> String {
    if input_data.receipt.verify(REPORT_ID).is_err() {
        shared
            .tx
            .send("Attempting to report with invalid receipt".to_string())
            .unwrap();
        return "Could not verify receipt".to_string();
    }

    // Decode journal
    let data: ReportJournal = input_data.receipt.journal.decode().unwrap();

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

    // Confirm firing player exists and is valid
    let player = match game.pmap.get_mut(&data.fleet) {
        Some(p) => p,
        None => return format!("Player {} not found in game {}", data.fleet, data.gameid),
    };

    // Confirm public key matches
    if player.public_key != public_key {
        return format!("Public key mismatch for player {}", data.fleet);
    }

    // Make sure the shot advertised by the player is correct
    if game.shot_position != Some(data.pos) {
        return format!(
            "Shot {} is not the shot fired by adversary ({})",
            xy_pos(Some(data.pos)),
            xy_pos(game.shot_position)
        );
    }

    // Validate that the stored commitment matches the one in the proof
    if player.current_state != data.board {
        return format!("Reported board does not match stored commitment");
    }

    // Update the stored board state
    player.current_state = data.next_board.clone();

    // Update turn order
    let token_data: &EncryptedToken = input_data.token_data.as_ref().unwrap();
    game.encrypted_token = Some(token_data.enc_token.clone());
    game.turn_commitment = Some(token_data.token_hash.clone());
    game.shot_position = None;

    // Emit a formatted message
    let msg = format!(
        "\
        \x20 Report received.\n\
        \x20 ▶ Player {} reported {} at position {}\n\
        \x20 ▶ Commitment updated.\n\
        \x20 ▶ {} is now allowed to fire.\n\n\n\
        \x20",
        data.fleet,
        data.report,
        xy_pos(Some(data.pos)),
        data.fleet,
    );

    let html_msg = msg.replace('\n', "<br>");
    shared.tx.send(html_msg.clone()).unwrap();

    "OK".to_string()
}
