// src/game_actions.rs

use crate::{unmarshal_data, unmarshal_fire, unmarshal_report, send_receipt, FormData, generate_receipt};

use fleetcore::{BaseInputs, Command, FireInputs};
use methods::{FIRE_ELF, JOIN_ELF, REPORT_ELF, WAVE_ELF, WIN_ELF};

// TODO: Ask about re-import
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};

pub async fn join_game(idata: FormData) -> String {
    // This contains the game ID, Fleet ID, the board vector, and the random nonce
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

    // Generate Receipt
    let receipt = generate_receipt(&input, JOIN_ELF);

    // Send the receipt
    send_receipt(Command::Join, receipt).await
}

pub async fn fire(idata: FormData) -> String {
    let (gameid, fleetid, board, random, targetfleet, x, y) = match unmarshal_fire(&idata) {
        Ok(values) => values,
        Err(err) => return err,
    };

    // Create the zkVM input struct
    let input = FireInputs {
        gameid,
        fleet: fleetid,
        board,
        random,
        target: targetfleet,
        pos: y * 10 + x
    };

    // Generate Receipt
    let receipt = generate_receipt(&input, FIRE_ELF);

    // Send the receipt
    send_receipt(Command::Fire, receipt).await
}

pub async fn report(idata: FormData) -> String {
    let (gameid, fleetid, board, random, _report, x, y) = match unmarshal_report(&idata) {
        Ok(values) => values,
        Err(err) => return err,
    };

    // Create the zkVM input struct
    let input = FireInputs {
        gameid,
        fleet: fleetid,
        board,
        random,
        target: _report,
        pos: y * 10 + x
    };

    // Generate Receipt
    let receipt = generate_receipt(&input, REPORT_ELF);

    // Send the receipt
    send_receipt(Command::Report, receipt).await
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
