use crate::{xy_pos, Game, Player, SharedData};
use fleetcore::{CommunicationData, SignedMessage};
use methods::WIN_ID;

use std::{
    collections::HashMap,
    error::Error,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
pub fn handle_win(shared: &SharedData, input_data: &CommunicationData) -> String {
    // TO DO:
    "OK".to_string()
}
