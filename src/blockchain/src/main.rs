// Remove the following 3 lines to enable compiler checkings
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use futures::stream::StreamExt;
use rand::{seq::IteratorRandom, SeedableRng};
use risc0_zkvm::Digest;

use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

use axum::{
    extract::{Extension, Query},
    response::{sse::Event, sse::Sse, Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};

use std::{
    collections::HashMap,
    error::Error,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Instant,
};

use serde::{Deserialize, Serialize};

mod states;
use states::{Game, Player, SharedData};

mod handlers;
use handlers::{handle_contest, handle_fire, handle_join, handle_report, handle_wave, handle_win};

mod authenticate;
use authenticate::{authenticate, verify_signature};

use fleetcore::{Command, CommunicationData, SignedMessage};

use base64::{engine::general_purpose, Engine as _};

#[tokio::main]
async fn main() {
    // Create a broadcast channel for log messages
    let (tx, _rx) = broadcast::channel::<String>(100);
    let shared = SharedData {
        tx: tx,
        gmap: Arc::new(Mutex::new(HashMap::new())),
        rng: Arc::new(Mutex::new(rand::rngs::StdRng::from_entropy())),
    };

    // Build our application with a route
    let app = Router::new()
        .route("/", get(index))
        .route("/logs", get(logs))
        .route("/chain", post(smart_contract))
        .route("/key", get(get_rsa_key))
        .route("/players", get(get_player_list))
        .route("/token", get(get_token_data))
        .layer(Extension(shared));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    println!("Listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Handler to serve the HTML page
async fn index() -> Html<&'static str> {
    Html(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Blockchain Emulator</title>
        </head>
        <body>
            <h1>Registered Transactions</h1>
            <ul id="logs"></ul>
            <script>
                const eventSource = new EventSource('/logs');
                eventSource.onmessage = function(event) {
                    const logs = document.getElementById('logs');
                    const log = document.createElement('li');
                    log.innerHTML = event.data;
                    logs.appendChild(log);
                };
            </script>
        </body>
        </html>
        "#,
    )
}

#[derive(Serialize)]
pub struct TokenData {
    pub enc_token: String,
    pub token_hash: [u8; 32],
}

#[axum::debug_handler]
pub async fn get_token_data(
    Query(params): Query<HashMap<String, String>>,
    Extension(shared): Extension<SharedData>,
) -> impl IntoResponse {
    let gameid = match params.get("gameid") {
        Some(id) => id,
        None => return "Missing gameid".to_string().into_response(),
    };

    let gmap = shared.gmap.lock().unwrap();
    let game = match gmap.get(gameid) {
        Some(g) => g,
        None => return "Game not found".to_string().into_response(),
    };

    match (&game.encrypted_token, &game.turn_commitment) {
        (Some(enc), Some(hash)) => Json(TokenData {
            enc_token: enc.clone(),
            token_hash: (*hash).into(),
        })
        .into_response(),
        _ => "No token available".to_string().into_response(),
    }
}

#[derive(Deserialize)]
struct Params {
    gameid: String,
    fleetid: String,
}

async fn get_rsa_key(
    Extension(shared): Extension<SharedData>,
    Query(params): Query<Params>,
) -> Result<String, String> {
    let gmap = shared.gmap.lock().unwrap();
    let game = gmap
        .get(&params.gameid)
        .ok_or_else(|| "Game not found".to_string())?;
    let player = game
        .pmap
        .get(&params.fleetid)
        .ok_or_else(|| "Fleet not found".to_string())?;

    Ok(base64::engine::general_purpose::STANDARD.encode(&player.rsa_pubkey))
}

#[derive(Deserialize)]
struct GameQuery {
    gameid: String,
}

async fn get_player_list(
    Extension(shared): Extension<SharedData>,
    Query(query): Query<GameQuery>,
) -> Json<Vec<String>> {
    let gmap = shared.gmap.lock().unwrap();
    let game = match gmap.get(&query.gameid) {
        Some(g) => g,
        None => return Json(vec![]),
    };

    Json(game.pmap.keys().cloned().collect())
}

// Handler to manage SSE connections
#[axum::debug_handler]
async fn logs(Extension(shared): Extension<SharedData>) -> impl IntoResponse {
    let rx = BroadcastStream::new(shared.tx.subscribe());

    let stream = rx.map(|result| match result {
        Ok(msg) => Ok(Event::default().data(msg)),
        Err(_) => Err(Box::<dyn Error + Send + Sync>::from("Error")),
    });

    Sse::new(stream)
}

async fn smart_contract(
    Extension(shared): Extension<SharedData>,
    Json(signed): Json<SignedMessage<CommunicationData>>,
) -> String {
    if let Err(err) = authenticate(&shared, &signed) {
        return err;
    }

    let input = &signed.payload;
    let pk = &signed.public_key;

    match input.cmd {
        Command::Join => handle_join(&shared, input, pk),
        Command::Fire => handle_fire(&shared, input, pk),
        Command::Report => handle_report(&shared, input, pk),
        Command::Wave => handle_wave(&shared, input, pk),
        Command::Win => handle_win(&shared, input, pk),
        Command::Contest => handle_contest(&shared, input, pk),
    }
}

// -----------------------------------------------------------------------------
// AUXILIARY FUNCTIONS
// -----------------------------------------------------------------------------

fn xy_pos(pos: Option<u8>) -> String {
    match pos {
        Some(p) => {
            let x = p % 10;
            let y = p / 10;
            format!("{}{}", (x + 65) as char, y)
        }
        None => "None".to_string(),
    }
}
