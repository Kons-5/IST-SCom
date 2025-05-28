use crate::{xy_pos, Game, Player, SharedData};
use fleetcore::{CommunicationData, SignedMessage};
use methods::WAVE_ID;

use std::{
    collections::HashMap,
    error::Error,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
pub fn handle_wave(shared: &SharedData, input_data: &CommunicationData) -> String {
    // TO DO:
    "OK".to_string()
}
