// src/game_actions.rs

use fleetcore::{BaseInputs, Command, FireInputs};
use methods::{FIRE_ELF, JOIN_ELF, REPORT_ELF, WAVE_ELF, WIN_ELF};

use crate::{unmarshal_data, unmarshal_fire, unmarshal_report, send_receipt, FormData};

pub async fn join_game(idata: FormData) -> String {
    // This contains the game ID, Fleet ID, the board (?), and the random nonce
    let (gameid, fleetid, board, random) = match unmarshal_data(&idata) {
        Ok(values) => values,
        Err(err) => return err,
    };

    // Create the zkVM input struct
    let input = BaseInputs {
        gameid,
        fleet: fleetid,
        board,
        random,
    };

    // Create environment used by the zkVM
    let env = match ExecutorEnv::builder().write(&input).build() {
        Ok(env) => env,
        Err(e) => return format!("Failed to build ExecutorEnv: {}", e),
    };

    // Run the zkVM
    let session = match default_executor_from_elf(env, JOIN_ELF).and_then(|exec| exec.run()) {
        Ok(session) => session,
        Err(e) => return format!("Failed to run zkVM: {}", e),
    };

    // Uncomment the following line when you are ready to send the receipt
    //send_receipt(Command::Fire, receipt).await
    // Comment out the following line when you are ready to send the receipt
    "OK".to_string()
}

pub async fn fire(idata: FormData) -> String {
    let (gameid, fleetid, board, random, targetfleet, x, y) = match unmarshal_fire(&idata) {
        Ok(values) => values,
        Err(err) => return err,
    };
    // TO DO: Rebuild the receipt
    // Uncomment the following line when you are ready to send the receipt
    //send_receipt(Command::Fire, receipt).await
    // Comment out the following line when you are ready to send the receipt
    "OK".to_string()
}

pub async fn report(idata: FormData) -> String {
    let (gameid, fleetid, board, random, _report, x, y) = match unmarshal_report(&idata) {
        Ok(values) => values,
        Err(err) => return err,
    };
    // TO DO: Rebuild the receipt

    // Uncomment the following line when you are ready to send the receipt
    //send_receipt(Command::Fire, receipt).await
    // Comment out the following line when you are ready to send the receipt
    "OK".to_string()
}

pub async fn wave(idata: FormData) -> String {
    let (gameid, fleetid, board, random) = match unmarshal_data(&idata) {
        Ok(values) => values,
        Err(err) => return err,
    };
    // TO DO: Rebuild the receipt

    // Uncomment the following line when you are ready to send the receipt
    //send_receipt(Command::Fire, receipt).await
    // Comment out the following line when you are ready to send the receipt
    "OK".to_string()
}

pub async fn win(idata: FormData) -> String {
    let (gameid, fleetid, board, random) = match unmarshal_data(&idata) {
        Ok(values) => values,
        Err(err) => return err,
    };
    // TO DO: Rebuild the receipt

    // Uncomment the following line when you are ready to send the receipt
    //send_receipt(Command::Fire, receipt).await
    // Comment out the following line when you are ready to send the receipt
    "OK".to_string()
}
