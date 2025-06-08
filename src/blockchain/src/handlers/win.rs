use crate::states::PendingWin;
use crate::{xy_pos, Game, Player, SharedData};
use fleetcore::{BaseJournal, CommunicationData};
use methods::WIN_ID;
use std::time::Instant;

use std::{
    collections::HashMap,
    error::Error,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

pub fn handle_win(
    shared: &SharedData,
    input_data: &CommunicationData,
    public_key: &[u8],
) -> String {
    if input_data.receipt.verify(WIN_ID).is_err() {
        shared
            .tx
            .send("Attempting victory with invalid receipt".to_string())
            .unwrap();
        return "Could not verify receipt".to_string();
    }

    // Decode journal
    let data: BaseJournal = input_data.receipt.journal.decode().unwrap();

    // Confirm game exists
    let mut gmap = shared.gmap.lock().unwrap();
    let game = match gmap.get_mut(&data.gameid) {
        Some(g) => g,
        None => return format!("Game {} not found\n", data.gameid),
    };

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

    // Check if there's a win claim already
    if game.pending_win.is_some() {
        return "There's a victory claim already<br>".to_string();
    }

    // Change the game state
    game.pending_win = Some(PendingWin {
        claimant: data.fleet.clone(),
        board: data.board,
        time: Instant::now(),
    });

    let msg = format!(
        "Player {} has claimed victory in game {}!\nAnyone may now contest the claim.\n\n\
        \x20",
        data.fleet, data.gameid
    );
    let html_msg = msg.replace('\n', "<br>");
    shared.tx.send(html_msg.clone()).unwrap();

    "OK".to_string()
}
