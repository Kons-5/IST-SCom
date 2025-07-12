// Remove the following 3 lines to enable compiler checkings
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use percent_encoding;
use serde::{Deserialize, Serialize};

use fleetcore::{Command, CommunicationData, EncryptedToken, SignedMessage};

mod game_actions;
pub use game_actions::{contest, fire, join_game, report, wave, win};

mod signing;
use signing::{import_key_base64, sign_payload};

mod token_gen;
use token_gen::prepare_turn_token;

use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use std::{error::Error, string};

use reqwest::Client;

#[derive(Deserialize)]
pub struct FormData {
    pub button: String,

    // Dilithium
    pub d_pubkey: Option<String>,
    pub d_privkey: Option<String>,

    // Turn-Token
    pub rsa_pubkey: Option<String>,
    pub rsa_privkey: Option<String>,
    pub turn_token: Option<String>,

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

fn generate_receipt<T: serde::Serialize>(input: &T, elf: &[u8]) -> Result<Receipt, String> {
    let env = ExecutorEnv::builder()
        .write(input)
        .map_err(|e| format!("env write error: {:?}", e))?
        .build()
        .map_err(|e| format!("env build error: {:?}", e))?;

    let prover = default_prover();

    let session = prover
        .prove(env, elf)
        .map_err(|e| format!("zkVM proof failed, {:?}", e))?;

    Ok(session.receipt)
}

/// Sends a signed CommunicationData payload to the blockchain, optionally encrypting a turn token.
pub async fn send_receipt(
    action: Command,
    receipt: Receipt,
    idata: &FormData,
    recipient_rsa_pubkey: Option<String>,
) -> String {
    // Encrypt token and compute hash if recipient public RSA key is provided
    let turn_token_b64 = idata.turn_token.as_deref().unwrap_or_default(); // Retrieve token

    let mut enc_token_opt = None;
    let mut token_hash_opt = None;
    let mut recipient_pubkey_bytes = Vec::new();

    if let Some(pubkey_b64) = &recipient_rsa_pubkey {
        if !turn_token_b64.is_empty() {
            if let Some((enc_token, token_hash)) = prepare_turn_token(pubkey_b64, turn_token_b64) {
                enc_token_opt = Some(enc_token);
                token_hash_opt = Some(token_hash);
                recipient_pubkey_bytes = import_key_base64(pubkey_b64);
            }
        }
    }

    // Construct payload
    let token_data = match (enc_token_opt, token_hash_opt) {
        (Some(enc_token), Some(token_hash)) => Some(EncryptedToken {
            enc_token: enc_token,
            token_hash: token_hash.into(),
            pub_rsa_key: recipient_pubkey_bytes,
        }),
        _ => None,
    };

    let payload = CommunicationData {
        cmd: action,
        receipt,
        token_data,
    };

    // Retrieve submitter's keys and sign payload
    let d_pubkey = idata.d_pubkey.as_deref().unwrap_or_default();
    let d_privkey = idata.d_privkey.as_deref().unwrap_or_default();

    let signed = match sign_payload(payload, d_pubkey, d_privkey) {
        Some(signed) => signed,
        None => return "Failed to sign payload".to_string(),
    };

    // Send to blockchain server
    let client = Client::new();
    match client
        .post("http://chain0:3001/chain")
        .json(&signed)
        .send()
        .await
    {
        Ok(resp) => resp
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read response".to_string()),
        Err(_) => "Error sending signed message".to_string(),
    }
}

pub fn unmarshal_data(idata: &FormData) -> Result<(String, String, Vec<u8>, String), String> {
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
) -> Result<(String, String, Vec<u8>, String, String, u8, u8), String> {
    let (gameid, fleetid, board, random) = unmarshal_data(idata)?;
    let (x, y) = get_coordinates(&idata.x, &idata.y)?;
    let targetfleet = idata
        .targetfleet
        .clone()
        .ok_or_else(|| "You must provide a Target Fleet ID".to_string())?;

    Ok((gameid, fleetid, board, random, targetfleet, x, y))
}

pub fn unmarshal_report(
    idata: &FormData,
) -> Result<(String, String, Vec<u8>, String, String, u8, u8), String> {
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

    Ok((gameid, fleetid, board, random, report, x, y))
}
