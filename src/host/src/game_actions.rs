// src/game_actions.rs

use crate::{
    generate_receipt, send_receipt, unmarshal_data, unmarshal_fire, unmarshal_report, FormData,
};

use fleetcore::{BaseInputs, Command, FireInputs, TokenAuth};
use methods::{CONTEST_ELF, FIRE_ELF, JOIN_ELF, REPORT_ELF, WAVE_ELF, WIN_ELF};

use risc0_zkvm::sha::Digest;
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};

use rand::{seq::IteratorRandom, SeedableRng};
use reqwest::get;

use base64::engine::general_purpose;
use base64::Engine;
use rsa::pkcs1v15::Pkcs1v15Encrypt;
use rsa::{pkcs8::DecodePrivateKey, RsaPrivateKey};
use serde::Deserialize;

pub async fn join_game(idata: FormData) -> String {
    // This contains the game ID, Fleet ID, the board vector, and the random nonce
    let (gameid, fleetid, board, random) = match unmarshal_data(&idata) {
        Ok(values) => values,
        Err(err) => return err,
    };

    // Create the zkVM input struct
    let input = BaseInputs {
        gameid: gameid,
        fleet: fleetid,
        board: board,
        random: random,
        token_auth: None,
    };

    // Generate Receipt
    let receipt = match generate_receipt(&input, JOIN_ELF) {
        Ok(r) => r,
        Err(e) => return format!("Proof generation failed: {e}"),
    };

    // Send your own pubkey to register on the blockchain
    let rsa_pubkey = match &idata.rsa_pubkey {
        Some(k) if !k.is_empty() => k.clone(),
        _ => return "Missing RSA public key".to_string(),
    };

    println!(
        "{:?}\n {:?}\n {:?}\n\n",
        idata.fleetid, idata.rsa_pubkey, idata.rsa_privkey,
    );

    // Send the receipt
    send_receipt(Command::Join, receipt, &idata, Some(rsa_pubkey)).await
}

pub async fn fire(idata: FormData) -> String {
    let (gameid, fleetid, board, random, targetfleet, x, y) = match unmarshal_fire(&idata) {
        Ok(values) => values,
        Err(err) => return err,
    };

    println!(
        "{:?}\n {:?}\n {:?}\n\n",
        idata.fleetid, idata.rsa_pubkey, idata.rsa_privkey,
    );

    let gameid_clone = gameid.clone();
    let targetfleet_clone = targetfleet.clone();

    // Create the zkVM input struct
    let input = FireInputs {
        gameid: gameid,
        fleet: fleetid,
        board: board,
        random: random,
        target: targetfleet,
        pos: y * 10 + x,
        token_auth: match build_token_auth(&gameid_clone, &idata).await {
            Ok(auth) => Some(auth),
            Err(e) => return e,
        },
    };

    // Generate Receipt
    let receipt = match generate_receipt(&input, FIRE_ELF) {
        Ok(r) => r,
        Err(e) => return format!("Proof generation failed: {e}"),
    };

    // Fetch target public key
    let rsa_pubkey = match fetch_rsa_pubkey(&gameid_clone, &targetfleet_clone).await {
        Ok(k) => k,
        Err(e) => return e,
    };

    // Send the receipt
    send_receipt(Command::Fire, receipt, &idata, Some(rsa_pubkey)).await
}

pub async fn report(idata: FormData) -> String {
    let (gameid, fleetid, board, random, report_value, x, y) = match unmarshal_report(&idata) {
        Ok(values) => values,
        Err(err) => return err,
    };

    let gameid_clone = gameid.clone();

    // Create the zkVM input struct
    let input = FireInputs {
        gameid: gameid,
        fleet: fleetid,
        board: board,
        random: random,
        target: report_value,
        pos: y * 10 + x,
        token_auth: match build_token_auth(&gameid_clone, &idata).await {
            Ok(auth) => Some(auth),
            Err(e) => return e,
        },
    };

    // Generate Receipt
    let receipt = match generate_receipt(&input, REPORT_ELF) {
        Ok(r) => r,
        Err(e) => return format!("Proof generation failed: {e}"),
    };

    // Send your own pubkey
    let rsa_pubkey = match &idata.rsa_pubkey {
        Some(k) if !k.is_empty() => k.clone(),
        _ => return "Missing RSA public key".to_string(),
    };

    // Send the receipt
    send_receipt(Command::Report, receipt, &idata, Some(rsa_pubkey)).await
}

pub async fn wave(idata: FormData) -> String {
    let (gameid, fleetid, board, random) = match unmarshal_data(&idata) {
        Ok(values) => values,
        Err(err) => return err,
    };

    let gameid_clone = gameid.clone();
    let fleetid_clone = fleetid.clone();

    let input = BaseInputs {
        gameid: gameid,
        fleet: fleetid,
        board: board,
        random: random,
        token_auth: match build_token_auth(&gameid_clone, &idata).await {
            Ok(auth) => Some(auth),
            Err(e) => return e,
        },
    };

    let receipt = match generate_receipt(&input, WAVE_ELF) {
        Ok(r) => r,
        Err(e) => return format!("Proof generation failed: {e}"),
    };

    let target = match pick_random_other_player(&gameid_clone, &fleetid_clone).await {
        Some(id) => id,
        None => return "No valid player to pass token to".to_string(),
    };

    let rsa_pubkey = match fetch_rsa_pubkey(&gameid_clone, &target).await {
        Ok(k) => k,
        Err(e) => return e,
    };

    send_receipt(Command::Wave, receipt, &idata, Some(rsa_pubkey)).await
}

pub async fn win(idata: FormData) -> String {
    let (gameid, fleetid, board, random) = match unmarshal_data(&idata) {
        Ok(values) => values,
        Err(err) => return err,
    };

    let input = BaseInputs {
        gameid: gameid,
        fleet: fleetid,
        board: board,
        random: random,
        token_auth: None,
    };

    let receipt = match generate_receipt(&input, WIN_ELF) {
        Ok(r) => r,
        Err(e) => return format!("Proof generation failed: {e}"),
    };

    send_receipt(Command::Win, receipt, &idata, None).await
}

pub async fn contest(idata: FormData) -> String {
    let (gameid, fleetid, board, random) = match unmarshal_data(&idata) {
        Ok(values) => values,
        Err(err) => return err,
    };

    let input = BaseInputs {
        gameid: gameid,
        fleet: fleetid,
        board: board,
        random: random,
        token_auth: None,
    };

    let receipt = match generate_receipt(&input, CONTEST_ELF) {
        Ok(r) => r,
        Err(e) => return format!("Proof generation failed: {e}"),
    };

    send_receipt(Command::Contest, receipt, &idata, None).await
}

#[derive(Deserialize)]
struct TokenData {
    enc_token: String,
    token_hash: [u8; 32],
}

async fn build_token_auth(gameid: &str, idata: &FormData) -> Result<TokenAuth, String> {
    let token_url = format!("http://chain0:3001/token?gameid={}", gameid);
    let token_data: TokenData = reqwest::get(&token_url)
        .await
        .map_err(|_| "Failed to fetch token".to_string())?
        .json()
        .await
        .map_err(|_| "Invalid token data".to_string())?;

    let rsa_privkey_b64 = idata
        .rsa_privkey
        .as_ref()
        .ok_or("Missing RSA private key")?;

    let privkey_bytes = general_purpose::STANDARD
        .decode(rsa_privkey_b64)
        .map_err(|_| "Base64 decode failed")?;

    let priv_pem = String::from_utf8(privkey_bytes).map_err(|_| "Invalid UTF-8 PEM")?;

    let privkey = RsaPrivateKey::from_pkcs8_pem(&priv_pem).map_err(|_| "Invalid RSA key format")?;

    let encrypted_bytes = general_purpose::STANDARD
        .decode(&token_data.enc_token)
        .map_err(|_| "Bad enc token")?;

    let decrypted_token = privkey
        .decrypt(Pkcs1v15Encrypt, &encrypted_bytes)
        .map_err(|_| "Decrypt failed")?;

    let digest = Digest::try_from(token_data.token_hash.as_slice())
        .map_err(|_| "Invalid token hash length")?;

    println!(
        "DENTRO!!\n{:?}\n {:?}\n {:?}\n{:?}\n\n",
        idata.fleetid, idata.rsa_pubkey, idata.rsa_privkey, decrypted_token
    );

    Ok(TokenAuth {
        token: decrypted_token,
        expected_hash: digest,
    })
}

async fn fetch_rsa_pubkey(gameid: &str, fleetid: &str) -> Result<String, String> {
    let url = format!(
        "http://chain0:3001/key?gameid={}&fleetid={}",
        gameid, fleetid
    );
    let resp = reqwest::get(&url).await.map_err(|_| "❌ Fetch failed")?;
    resp.text().await.map_err(|_| "Invalid RSA key".to_string())
}

pub async fn pick_random_other_player(gameid: &str, self_id: &str) -> Option<String> {
    let url = format!("http://chain0:3001/players?gameid={}", gameid);

    let response = reqwest::get(&url).await.ok()?;
    let players: Vec<String> = response.json().await.ok()?;

    let mut rng = rand::rngs::StdRng::from_entropy();
    players
        .into_iter()
        .filter(|id| id != self_id)
        .choose(&mut rng)
}
