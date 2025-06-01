// Remove the following 3 lines to enable compiler checkings
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use futures::stream::StreamExt;
use rand::{seq::IteratorRandom, SeedableRng};
use risc0_zkvm::Digest;

use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};
use tokio_stream::wrappers::BroadcastStream;

use axum::{
    extract::Extension,
    response::{sse::Event, Html, IntoResponse},
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

mod states;
use states::{Game, PendingWin, Player, SharedData};

mod handlers;
use handlers::{handle_contest, handle_fire, handle_join, handle_report, handle_wave, handle_win};

mod authenticate;
use authenticate::{authenticate, verify_signature};

use fleetcore::{Command, CommunicationData, SignedMessage};

#[tokio::main]
async fn main() {
    // Create a broadcast channel for log messages
    let (tx, _rx) = broadcast::channel::<String>(100);
    let shared = SharedData {
        tx: tx,
        gmap: Arc::new(Mutex::new(HashMap::new())),
        rng: Arc::new(Mutex::new(rand::rngs::StdRng::from_entropy())),
    };

    // Spawn a background task to periodically finalize unchallenged victory claims
    tokio::spawn(check_pending_wins(shared.clone()));

    // Build our application with a route
    let app = Router::new()
        .route("/", get(index))
        .route("/logs", get(logs))
        .route("/chain", post(smart_contract))
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

// Handler to manage SSE connections
#[axum::debug_handler]
async fn logs(Extension(shared): Extension<SharedData>) -> impl IntoResponse {
    let rx = BroadcastStream::new(shared.tx.subscribe());
    let stream = rx.filter_map(|result| async move {
        match result {
            Ok(msg) => Some(Ok(Event::default().data(msg))),
            Err(_) => Some(Err(Box::<dyn Error + Send + Sync>::from("Error"))),
        }
    });

    axum::response::sse::Sse::new(stream)
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

fn xy_pos(pos: u8) -> String {
    let x = pos % 10;
    let y = pos / 10;
    format!("{}{}", (x + 65) as char, y)
}

/// Moves the current player to the back of the queue, without updating next_player.
fn rotate_player_to_back(game: &mut Game, player_id: &str) {
    if let Some(pos) = game.player_order.iter().position(|id| id == player_id) {
        let who = game.player_order.remove(pos);
        game.player_order.push(who);
    }
}

async fn check_pending_wins(shared: SharedData) {
    loop {
        {
            let mut gmap = shared.gmap.lock().unwrap();
            let now = Instant::now();

            let mut to_finalize = vec![];

            for (gid, game) in gmap.iter() {
                if let Some(pending) = &game.pending_win {
                    // Verify if the timeout has elapsed (300 seconds)
                    if now.duration_since(pending.time) > Duration::from_secs(300) {
                        to_finalize.push((gid.clone(), pending.claimant.clone()));
                    }
                }
            }

            for (gid, winner) in to_finalize {
                gmap.remove(&gid);
                let msg = format!(
                    "Victory claim by {} in game {} has been finalized (no contest).",
                    winner, gid
                );
                shared.tx.send(msg.replace('\n', "<br>")).unwrap();
            }
        }

        sleep(Duration::from_secs(15)).await;
    }
}
