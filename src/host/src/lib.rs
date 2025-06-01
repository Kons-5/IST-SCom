// Remove the following 3 lines to enable compiler checkings
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use percent_encoding;
use serde::{Deserialize, Serialize};
mod game_actions;
mod key_pairs;

use fleetcore::{Command, CommunicationData, SignedMessage};
pub use game_actions::{fire, join_game, report, wave, win};
use key_pairs::{import_key_base64, sign_message, prepare_turn_token};

use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use std::{error::Error, string};

fn generate_receipt<T: serde::Serialize>(input: &T, elf: &[u8]) -> Result<Receipt, String> {
    let env = ExecutorEnv::builder()
        .write(input)
        .map_err(|e| format!("env write error: {:?}", e))?
        .build()
        .map_err(|e| format!("env build error: {:?}", e))?;

    let prover = default_prover();

    let session = prover
        .prove(env, elf)
        .map_err(|e| format!("zkVM proof failed: {:?}", e))?;

    Ok(session.receipt)
}

async fn send_receipt(action: Command, receipt: Receipt, idata: &FormData, recipient_rsa_pubkey: Option<String>,) -> String {

    let d_pubkey = idata.d_pubkey.clone().unwrap();
    let d_privkey = idata.d_privkey.clone().unwrap();

    let pk = import_key_base64(&d_pubkey);
    let sk = import_key_base64(&d_privkey);

    let (enc_token, r_hash) = match recipient_rsa_pubkey {
        Some(ref pubkey) => {
            prepare_turn_token(pubkey).unwrap_or((String::new(), [0u8; 32]))
        }
        None => (String::new(), [0u8; 32]),
    };

    let payload = CommunicationData {
        cmd: action,
        receipt,
        enc_token: if enc_token.is_empty() { None } else { Some(enc_token) },
        r_hash: if r_hash == [0u8; 32] { None } else { Some(r_hash) },
        pub_rsa_key: import_key_base64(recipient_rsa_pubkey.as_deref().unwrap_or("")),
    };

    let payload_bytes = match serde_json::to_vec(&payload) {
        Ok(b) => b,
        Err(_) => return "Failed to serialize payload".to_string(),
    };

    let signature = sign_message(&payload_bytes, &sk);

    let signed = SignedMessage {
        payload,
        signature,
        public_key: pk,
    };

    let client = reqwest::Client::new();
    let res = client
        .post("http://chain0:3001/chain")
        .json(&signed)
        .send()
        .await;

    match res {
        Ok(response) => response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read response".to_string()),
        Err(_) => "Error sending signed message".to_string(),
    }
}

#[derive(Deserialize)]
pub struct FormData {
    pub button: String,

    // Dilithium
    pub d_pubkey: Option<String>,
    pub d_privkey: Option<String>,

    // RSA
    pub rsa_pubkey: Option<String>,
    pub rsa_privkey: Option<String>,

    pub gameid: Option<String>,
    pub fleetid: Option<String>,
    pub targetfleet: Option<String>,

    pub x: Option<String>,
    pub y: Option<String>,
    pub rx: Option<String>,
    pub ry: Option<String>,

    pub report: Option<String>,
    pub board: Option<String>,
    pub shots: Option<String>,
    pub random: Option<String>,
}

pub fn unmarshal_data(
    idata: &FormData,
) -> Result<(String, String, Vec<u8>, String), String> {
    let gameid = idata
        .gameid
        .clone()
        .ok_or_else(|| "You must provide a Game ID".to_string())
        .and_then(|id| {
            if id.is_empty() {
                Err("Game ID cannot be an empty string".to_string())
            } else {
                Ok(id)
            }
        })?;

    let fleetid = idata
        .fleetid
        .clone()
        .ok_or_else(|| "You must provide a Fleet ID".to_string())
        .and_then(|id| {
            if id.is_empty() {
                Err("Fleet ID cannot be an empty string".to_string())
            } else {
                Ok(id)
            }
        })?;

    let random: String = idata
        .random
        .clone()
        .ok_or_else(|| "You must provide a Random Seed".to_string())?;

    let board = idata
        .board
        .as_ref()
        .ok_or_else(|| "You must provide a Board Placement".to_string())
        .and_then(|id| {
            percent_encoding::percent_decode_str(id)
                .decode_utf8()
                .map_err(|_| "Invalid Board Placement".to_string())
                .map(|decoded| {
                    decoded
                        .split(',')
                        .map(|s| {
                            s.parse::<u8>()
                                .map_err(|_| "Invalid number in Board Placement".to_string())
                        })
                        .collect::<Result<Vec<u8>, String>>()
                })
        })??;

    let rsa_pubkey = match &idata.rsa_pubkey {
        Some(k) if !k.is_empty() => k.clone(),
        _ => return Err("Missing RSA public key".to_string()),
    };

    let rsa_privkey = match &idata.rsa_privkey {
        Some(k) if !k.is_empty() => k.clone(),
        _ => return Err("Missing RSA public key".to_string()),
    };

    Ok((gameid, fleetid, board, random))
}

fn get_coordinates(x: &Option<String>, y: &Option<String>) -> Result<(u8, u8), String> {
    let x: u8 = x
        .as_ref()
        .ok_or_else(|| "You must provide an X coordinate".to_string())
        .and_then(|id| {
            if let Some(first_char) = id.chars().next() {
                if ('A'..='J').contains(&first_char) {
                    Ok(first_char as u8 - b'A')
                } else {
                    Err("X coordinate must be between A and J".to_string())
                }
            } else {
                Err("Invalid X coordinate".to_string())
            }
        })?;

    let y: u8 = y
        .as_ref()
        .ok_or_else(|| "You must provide a Y coordinate".to_string())
        .and_then(|id| {
            if let Some(first_char) = id.chars().next() {
                if ('0'..='9').contains(&first_char) {
                    Ok(first_char as u8 - b'0')
                } else {
                    Err("Y coordinate must be between 0 and 9".to_string())
                }
            } else {
                Err("Invalid Y coordinate".to_string())
            }
        })?;

    Ok((x, y))
}

pub fn unmarshal_fire(
    idata: &FormData,
) -> Result<
    (
        String,
        String,
        Vec<u8>,
        String,
        String,
        u8,
        u8,
    ),
    String,
> {
    let (gameid, fleetid, board, random) = unmarshal_data(idata)?;
    let (x, y) = get_coordinates(&idata.x, &idata.y)?;
    let targetfleet = idata
        .targetfleet
        .clone()
        .ok_or_else(|| "You must provide a Target Fleet ID".to_string())?;

    Ok((
        gameid,
        fleetid,
        board,
        random,
        targetfleet,
        x,
        y,
    ))
}

pub fn unmarshal_report(
    idata: &FormData,
) -> Result<
    (
        String,
        String,
        Vec<u8>,
        String,
        String,
        u8,
        u8,
    ),
    String,
> {
    let (gameid, fleetid, board, random) = unmarshal_data(idata)?;
    let (x, y) = get_coordinates(&idata.rx, &idata.ry)?;
    let report = idata
        .report
        .clone()
        .ok_or_else(|| "You must provide a Report value".to_string())
        .and_then(|r| {
            if r == "Hit" || r == "Miss" {
                Ok(r)
            } else {
                Err("Report must be either 'Hit' or 'Miss'".to_string())
            }
        })?;

    Ok((
        gameid, fleetid, board, random, report, x, y,
    ))
}
