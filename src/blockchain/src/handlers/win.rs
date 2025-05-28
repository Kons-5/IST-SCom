use crate::{SharedData, Player, Game, xy_pos};
use fleetcore::{CommunicationData, BaseJournal};
use methods::WIN_ID;

use std::{
    collections::HashMap,
    error::Error,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

pub fn handle_win(shared: &SharedData, input_data: &CommunicationData) -> String {
    if input_data.receipt.verify(WIN_ID).is_err() {
        shared.tx.send("Attempting victory with invalid receipt".to_string()).unwrap();
        return "Could not verify receipt".to_string();
    }

    // Decode journal
    let data: BaseJournal = input_data.receipt.journal.decode().unwrap();

    // Confirm game exists
    let mut gmap = shared.gmap.lock().unwrap();
     let game = match gmap.get_mut(&data.gameid) {
         Some(g) => g,
         None => return format!("Game {} not found\n\n\n\
         \x20",
         data.gameid),
     };

     // Confirm firing player exists and is valid
     let player = match game.pmap.get(&data.fleet) {
         Some(p) => p,
         None => return format!("Player {} not found\n\n\n\
         \x20",
         data.fleet),
     };

     // Validate commitment hash
     if data.board != player.current_state {
         return "Fleet commitment does not match recorded state\n\n\n\
         \x20"
         .to_string();
     }

     let msg = format!(
         "Player {} has claimed victory in game {}!\n\n\n
         \x20",
         data.fleet,
         data.gameid
     );
     let html_msg = msg.replace('\n', "<br>");
     shared.tx.send(html_msg.clone()).unwrap();

     // Remove the game from the blockchain
     gmap.remove(&data.gameid);
     html_msg
}
