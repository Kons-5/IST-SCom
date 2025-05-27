use crate::{SharedData, Player, Game, xy_pos, rotate_player_to_back};
use fleetcore::{CommunicationData, BaseJournal};
use methods::WAVE_ID;

use std::{
    collections::HashMap,
    error::Error,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

pub fn handle_wave(shared: &SharedData, input_data: &CommunicationData) -> String {
    if input_data.receipt.verify(WAVE_ID).is_err() {
        shared.tx.send("Attempting to wave game with invalid receipt".to_string()).unwrap();
        return "Could not verify receipt".to_string();
    }
    // Decode journal
    let data: BaseJournal = input_data.receipt.journal.decode().unwrap();


    // Confirm game exists
    let mut gmap = shared.gmap.lock().unwrap();
    let game = match gmap.get_mut(&data.gameid) {
        Some(g) => g,
        None => {
            return format!(
                "Game {} not found\n\n\n\
                \x20",
                data.gameid
            );
        }
    };

    // Confirm player exists
    let player = match game.pmap.get(&data.fleet) {
        Some(p) => p,
        None => {
            return format!(
                "Player {} not found\n\n\n\
                \x20",
                data.fleet
            );
        }
    };

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
                "It's not {}'s turn: you must report the last shot before waving.\n\n\n\
                \x20",
                data.fleet
            );
        } else {
            return format!("It's not {}'s turn\n\n\n\
            \x20",
            data.fleet);
        }
    }

    // Rotate current player to back of queue
    rotate_player_to_back(game, &data.fleet);

    // Assign next player: first in the updated queue
    let next = game.player_order[0].clone();
    game.next_player = Some(next.clone());

    // Build message
    let msg = match game.next_player {
        Some(ref next) => format!(
            "\
            \x20 {} waved their turn.\n\
            \x20 ▶ Next player is {}.\n\n\n\
            \x20",
            data.fleet, next
        ),
        None => format!(
            "\
            \x20 {} waved their turn.\n\
            \x20 ▶ No remaining players with active fleets.\n\n\n\
            \x20",
            data.fleet
        ),
    };

    let html_msg = msg.replace('\n', "<br>");
    shared.tx.send(html_msg.clone()).unwrap();
    html_msg
}
