use crate::{xy_pos, Game, Player, SharedData};
use fleetcore::{CommunicationData, ReportJournal, SignedMessage};
use methods::REPORT_ID;

use std::{
    collections::HashMap,
    error::Error,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

pub fn handle_report(shared: &SharedData, input_data: &CommunicationData) -> String {
    if input_data.receipt.verify(REPORT_ID).is_err() {
        shared
            .tx
            .send("Attempting to report with invalid receipt".to_string())
            .unwrap();
        return "Could not verify receipt".to_string();
    }

    let data: ReportJournal = input_data.receipt.journal.decode().unwrap();

    let mut gmap = shared.gmap.lock().unwrap();
    let game = match gmap.get_mut(&data.gameid) {
        Some(g) => g,
        None => return format!("Game {} not found", data.gameid),
    };

    let player = match game.pmap.get_mut(&data.fleet) {
        Some(p) => p,
        None => return format!("Player {} not found in game {}", data.fleet, data.gameid),
    };

    // Ensure this player is the one expected to report
    if game.next_report.as_ref() != Some(&data.fleet) {
        return format!("It is not {}'s turn to report", data.fleet);
    }

    // Make sure the shot advertised by the player is correct
    if game.shot_position != data.pos{
        return format!("Shot {} is not the shot fired by adversary ({})",
            xy_pos(data.pos),
            data.report,
            xy_position(game.shot_position));
    }

    // Validate that the stored commitment matches the one in the proof
    if player.current_state != data.board {
        return format!("Reported board does not match stored commitment");
    }

    // Update the stored board state
    player.current_state = data.next_board.clone();

    // Update turn order
    game.next_report = None;
    game.next_player = Some(data.fleet.clone()); // This player can now fire

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
        xy_pos(data.pos),
        data.fleet,
    );

    let html_msg = msg.replace('\n', "<br>");
    shared.tx.send(html_msg.clone()).unwrap();
    html_msg
}
