use crate::{Game, SharedData};
use fleetcore::{BaseJournal, CommunicationData, EncryptedToken};
use methods::CONTEST_ID;

pub fn handle_contest(
    shared: &SharedData,
    input_data: &CommunicationData,
    public_key: &[u8],
) -> String {
    if input_data.receipt.verify(CONTEST_ID).is_err() {
        shared
            .tx
            .send("Attempting to contest a win with invalid receipt".to_string())
            .unwrap();
        return "Could not verify receipt".to_string();
    }

    // Decode journal
    let data: BaseJournal = input_data.receipt.journal.decode().unwrap();

    // Confirm game exists
    let mut gmap = shared.gmap.lock().unwrap();
    let game = match gmap.get_mut(&data.gameid) {
        Some(g) => g,
        None => return format!("Game {} not found\n", data.gameid),
    };

    // Verify if the player has reported before firing
    if game.shot_position.is_some() {
        return "You must report the last shot before firing.\n".to_string();
    }

    // Confirm firing player exists and is valid
    let player = match game.pmap.get(&data.fleet) {
        Some(p) => p,
        None => return format!("Player {} not found\n", data.fleet),
    };

    if player.public_key != public_key {
        return format!("Public key mismatch for player {}", data.fleet);
    }

    if data.board != player.current_state {
        return "Fleet commitment does not match recorded state\n".to_string();
    }

    if let Some(pending) = &game.pending_win {
        if pending.claimant == data.fleet {
            return "You cannot contest your own victory\n".to_string();
        }

        // Contest is valid
        let claimant = pending.claimant.clone();
        let gid = data.gameid.clone();
        let challenger = data.fleet.clone();

        // Not pending anymore
        game.pending_win = None;

        let msg = format!(
            "Victory claim by {} has been successfully contested by {} in game {}!\n\n\n",
            claimant, challenger, gid
        );
        shared.tx.send(msg.replace('\n', "<br>")).unwrap();

        return "OK".to_string();
    }

    "No active victory claim to contest.".to_string()
}
