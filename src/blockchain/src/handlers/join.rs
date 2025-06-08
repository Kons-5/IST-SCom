use crate::{xy_pos, Game, Player, SharedData};
use fleetcore::{BaseJournal, CommunicationData, EncryptedToken, SignedMessage};
use methods::JOIN_ID;
use std::{collections::HashMap, sync::Mutex};

pub fn handle_join(
    shared: &SharedData,
    input_data: &CommunicationData,
    public_key: &[u8],
) -> String {
    // Verify proof
    if input_data.receipt.verify(JOIN_ID).is_err() {
        shared
            .tx
            .send("Attempting to join game with invalid receipt".to_string())
            .unwrap();
        return "Could not verify receipt".to_string();
    }

    // Decode journal
    let data: BaseJournal = input_data.receipt.journal.decode().unwrap();

    // Extract token info (if present)
    let (token_hash_opt, enc_token_opt, rsa_pubkey_opt) = match &input_data.token_data {
        Some(t) => (
            Some(t.token_hash),
            Some(t.enc_token.clone()),
            Some(t.pub_rsa_key.clone()),
        ),
        None => (None, None, None),
    };

    // Access or initialize game. The 1st player joining has the turn.
    let mut gmap = shared.gmap.lock().unwrap();
    let game = gmap.entry(data.gameid.clone()).or_insert_with(|| Game {
        pmap: HashMap::new(),
        shot_position: None,
        pending_win: None,
        encrypted_token: enc_token_opt.clone(),
        turn_commitment: token_hash_opt,
    });

    //println!("reg {:?}\nmeu {:?}", game.turn_commitment, token_hash_opt);

    // Prevent joining mid-game
    if game.shot_position.is_some() {
        return format!(
            "Trying to join a game that's already ended! Game ID: {}, Players: [{}]",
            data.gameid,
            game.pmap.keys().cloned().collect::<Vec<_>>().join(", ")
        );
    }

    // Prevent joining game flagged as won
    if game.pending_win.is_some() {
        return format!("Trying to join a game that's already ended!",);
    }

    // Check for duplicate players
    if let Some(existing_player) = game.pmap.get(&data.fleet) {
        if existing_player.public_key != public_key {
            return format!(
                "Public key mismatch for player \"{}\" in game \"{}\"",
                data.fleet, data.gameid
            );
        }

        return format!(
            "Player \"{}\" is already in game \"{}\".\nCurrent players: [{}]",
            data.fleet,
            data.gameid,
            game.pmap.keys().cloned().collect::<Vec<_>>().join(", ")
        );
    }

    // Add player to the game
    game.pmap.insert(
        data.fleet.clone(),
        Player {
            name: data.fleet.clone(),
            current_state: data.board.clone(),
            public_key: public_key.to_vec(),
            rsa_pubkey: rsa_pubkey_opt.unwrap_or_default(),
        },
    );

    // Format success message
    let players: Vec<String> = game.pmap.keys().cloned().collect();
    let msg = format!(
        "\
        \x20 Join receipt decoded:\n\
        \x20 ▶ Game ID: {}\n\
        \x20 ▶ Fleet ID: {}\n\
        \x20 ▶ Commitment Hash: {:?}\n\n\
        \x20 Player \"{}\" joined game \"{}\".\n\
        \x20 ▶ Total players: {}\n\
        \x20 ▶ Current players: [{}]\n\n",
        data.gameid,
        data.fleet,
        data.board,
        data.fleet,
        data.gameid,
        players.len(),
        players.join(", ")
    );
    shared.tx.send(msg.replace('\n', "<br>")).unwrap();

    "OK".to_string()
}
