// src/game_actions.rs

use fleetcore::{BaseInputs, Command, FireInputs, validate_battleship_board};
use methods::{FIRE_ELF, JOIN_ELF, REPORT_ELF, WAVE_ELF, WIN_ELF};

use crate::{unmarshal_data, unmarshal_fire, unmarshal_report, send_receipt, FormData};

// TODO: Ask about re-import
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};

pub async fn join_game(idata: FormData) -> String {
    // This contains the game ID, Fleet ID, the board vector, and the random nonce
    let (gameid, fleetid, board, random) = match unmarshal_data(&idata) {
        Ok(values) => values,
        Err(err) => return err,
    };

    // Check fleet validity
    // TODO: Ask about ship validity
    /* if !validate_battleship_board(&board) {
        return "Invalid fleet layout.".to_string();
    } */

    // Create the zkVM input struct
    let input = BaseInputs {
        gameid,
        fleet: fleetid,
        board,
        random,
    };

    // Generate Receipt
    let receipt = generate_join_receipt(&input);

    // Send the receipt
    send_receipt(Command::Join, receipt).await
}

fn generate_join_receipt(input: &BaseInputs) -> Receipt {
    // Build the Executor environment
    // TODO: Ask about unwrap behaviour
    let env = ExecutorEnv::builder()
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    // Get the default prover
    let prover = default_prover();

    // Run the proof and return the receipt
    // This is an implicit return
    prover.prove(env, JOIN_ELF).unwrap().receipt
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
