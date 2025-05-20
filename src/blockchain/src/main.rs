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
    extract::Extension,
    response::{sse::Event, Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};

use fleetcore::{BaseJournal, Command, FireJournal, CommunicationData, ReportJournal};
use methods::{FIRE_ID, JOIN_ID, REPORT_ID, WAVE_ID, WIN_ID};

use std::{
    collections::HashMap,
    error::Error,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

struct Player {
    name: String,                       // Player ID
    current_state: Digest,              // Commitment hash
}
struct Game {
    pmap: HashMap<String, Player>,      // All players in the game
    next_player: Option<String>,        // player allowed to fire
    next_report: Option<String>,        // player expected to report
}

#[derive(Clone)]
struct SharedData {
    tx: broadcast::Sender<String>,
    gmap: Arc<Mutex<HashMap<String, Game>>>,
    rng: Arc<Mutex<rand::rngs::StdRng>>,
}

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
        .layer(Extension(shared));

    // Run our app with hyper
    //let addr = SocketAddr::from(([127, 0, 0, 1], 3001));

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

fn xy_pos(pos: u8) -> String {
    let x = pos % 10;
    let y = pos / 10;
    format!("{}{}", (x + 65) as char, y)
}

async fn smart_contract(
    Extension(shared): Extension<SharedData>,
    Json(input_data): Json<CommunicationData>,
) -> String {
    match input_data.cmd {
        Command::Join => handle_join(&shared, &input_data),
        Command::Fire => handle_fire(&shared, &input_data),
        Command::Report => handle_report(&shared, &input_data),
        Command::Wave => handle_wave(&shared, &input_data),
        Command::Win => handle_win(&shared, &input_data),
    }
}

fn handle_join(shared: &SharedData, input_data: &CommunicationData) -> String {
    if input_data.receipt.verify(JOIN_ID).is_err() {
        shared.tx.send("Attempting to join game with invalid receipt".to_string()).unwrap();
        return "Could not verify receipt".to_string();
    }
    // Decode journal
    let data: BaseJournal = input_data.receipt.journal.decode().unwrap();

    // Access game state
    // Look up the game by its ID. If it already exists, get a mutable reference to it.
    // If it doesn't exist, insert a new Game struct
    let mut gmap = shared.gmap.lock().unwrap();
    let game = gmap.entry(data.gameid.clone()).or_insert(Game {
        pmap: HashMap::new(),
        next_player: Some(data.fleet.clone()),                 // first to join = first to shoot
        next_report: None,                                     // No shots fired = No player to report
    });

    // Handle duplicate player
    if game.pmap.contains_key(&data.fleet) {
        let msg = format!(
            "Player \"{}\" is already in game \"{}\". Current players: [{}]\n\n\n\
            \x20",
            data.fleet,
            data.gameid,
            game.pmap.keys().cloned().collect::<Vec<_>>().join(", ")
        );
        // Check wheter it is expected to register invalid actions
        // shared.tx.send(msg.clone()).unwrap();
        return msg;
    }

    // Register the player in the game under their fleet ID (if not duplicate)
    game.pmap.insert(
        data.fleet.clone(),
        Player {
            name: data.fleet.clone(),
            current_state: data.board.clone(),
        },
    );

    // Create unified success message
    let players: Vec<String> = game.pmap.keys().cloned().collect();
    let msg = format!(
        "\
        \x20 Join receipt decoded:\n\
        \x20 ▶ Game ID: {}\n\
        \x20 ▶ Fleet ID: {}\n\
        \x20 ▶ Commitment Hash: {:?}\n\n\
        \x20 Player \"{}\" joined game \"{}\".\n\
        \x20 ▶ Total players: {}\n\
        \x20 ▶ Current players: [{}]\n\n\n\
        \x20",
        data.gameid,
        data.fleet,
        data.board,
        data.fleet,
        data.gameid,
        players.len(),
        players.join(", ")
    );
    let html_msg = msg.replace('\n', "<br>");
    shared.tx.send(html_msg.clone()).unwrap();
    "OK".to_string()
}

fn handle_fire(shared: &SharedData, input_data: &CommunicationData) -> String {
    if input_data.receipt.verify(FIRE_ID).is_err() {
        shared.tx.send("Attempting to fire with invalid receipt".to_string()).unwrap();
        return "Could not verify receipt".to_string();
    }

    // Decode journal
    let data: FireJournal = input_data.receipt.journal.decode().unwrap();

    // Confirm game exists
    let mut gmap = shared.gmap.lock().unwrap();
     let game = match gmap.get_mut(&data.gameid) {
         Some(g) => g,
         None => return format!("Game {} not found\n\n\n\
         \x20",
         data.gameid),
     };

     // Confirm firing player exists and is valid
     let player = match game.pmap.get(&data.fleet) {
         Some(p) => p,
         None => return format!("Player {} not found\n\n\n\
         \x20",
         data.fleet),
     };

     // Validate commitment hash
     if data.board != player.current_state {
         return "Fleet commitment does not match recorded state\n\n\n\
         \x20"
         .to_string();
     }

    // Validate player's turn
    if game.next_player.as_ref() != Some(&data.fleet) {
        // Check if this player is expected to report
        if game.next_report.as_ref() == Some(&data.fleet) {
            return format!(
                "It's not {}'s turn to fire: you must report the last shot before firing.\n\n\n\
                \x20",
                data.fleet
            );
        } else {
            return format!("It's not {}'s turn to fire\n\n\n\
            \x20",
            data.fleet);
        }
    }

    // Validate target's existence
    if !game.pmap.contains_key(&data.target) {
        return format!("Target {} does not exist\n\n\n\
        \x20",
        data.target);
    }

    // Update game state
    game.next_player = None;
    game.next_report = Some(data.target.clone());

    let msg = format!(
        "\
        \x20 Shots fired!\n\
        \x20 ▶ {} fired at position {} targeting {} in game {}\n\n\n\
        \x20",
        data.fleet, xy_pos(data.pos), data.target, data.gameid
        );

    let html_msg = msg.replace('\n', "<br>");
    shared.tx.send(html_msg.clone()).unwrap();
    msg
}

fn handle_report(shared: &SharedData, input_data: &CommunicationData) -> String {
    // TO DO:
    "OK".to_string()
}

fn handle_wave(shared: &SharedData, input_data: &CommunicationData) -> String {
    // TO DO:
    "OK".to_string()
}

fn handle_win(shared: &SharedData, input_data: &CommunicationData) -> String {
    // TO DO:
    "OK".to_string()
}
