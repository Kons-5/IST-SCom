// src/game_actions.rs

use crate::{
    generate_receipt, send_receipt, unmarshal_data, unmarshal_fire, unmarshal_report, FormData,
};

use fleetcore::{BaseInputs, Command, FireInputs};
use methods::{FIRE_ELF, JOIN_ELF, REPORT_ELF, WAVE_ELF, WIN_ELF};

use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};

pub async fn join_game(idata: FormData) -> String {
    // This contains the game ID, Fleet ID, the board vector, and the random nonce
    let (gameid, fleetid, board, random, pubkey, privkey) = match unmarshal_data(&idata) {
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
    send_receipt(Command::Join, receipt, pubkey, privkey).await
}

pub async fn fire(idata: FormData) -> String {
    let (gameid, fleetid, board, random, pubkey, privkey, targetfleet, x, y) =
        match unmarshal_fire(&idata) {
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
        pos: y * 10 + x,
    };

    // Generate Receipt
    let receipt = generate_receipt(&input, FIRE_ELF);

    // Send the receipt
    send_receipt(Command::Fire, receipt, pubkey, privkey).await
}

pub async fn report(idata: FormData) -> String {
    let (gameid, fleetid, board, random, pubkey, privkey, _report, x, y) =
        match unmarshal_report(&idata) {
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
        pos: y * 10 + x,
    };

    // Generate Receipt
    let receipt = generate_receipt(&input, REPORT_ELF);

    // Send the receipt
    send_receipt(Command::Report, receipt, pubkey, privkey).await
}

pub async fn wave(idata: FormData) -> String {
    let (gameid, fleetid, board, random, pubkey, privkey) = match unmarshal_data(&idata) {
        Ok(values) => values,
        Err(err) => return err,
    };

    let input = BaseInputs {
        gameid,
        fleet: fleetid,
        board,
        random,
    };

    let receipt = generate_receipt(&input, WAVE_ELF);
    send_receipt(Command::Wave, receipt, pubkey, privkey).await
}

pub async fn win(idata: FormData) -> String {
    let (gameid, fleetid, board, random, pubkey, privkey) = match unmarshal_data(&idata) {
        Ok(values) => values,
        Err(err) => return err,
    };

    let input = BaseInputs {
        gameid,
        fleet: fleetid,
        board,
        random,
    };

    let receipt = generate_receipt(&input, WIN_ELF);
    send_receipt(Command::Win, receipt, pubkey, privkey).await
}
