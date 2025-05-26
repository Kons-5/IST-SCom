use crate::{SharedData, Player, Game, xy_pos};
use fleetcore::CommunicationData;
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
