// Remove the following 3 lines to enable compiler checkings
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use axum::{
    extract::Form,
    response::Html,
    routing::{get, post},
    Router,
    Json
};
use serde_json::json;
use nanoid::nanoid;
use tokio::signal;

use host::{fire, join_game, report, wave, win, FormData};
use std::net::SocketAddr;

mod signing;
use signing::{export_key_base64, generate_keypair};

async fn index() -> Html<String> {
    render_html(None, None, None, None, None, None, None, None).await
}

fn process_input_data(input_data: FormData) -> FormData {
    match &input_data.random {
        Some(random) if !random.is_empty() => input_data,
        _ => FormData {
            random: Some(nanoid!(12)),
            ..input_data
        },
    }
}

async fn generate_keys() -> Json<serde_json::Value> {
    let (sk, pk) = generate_keypair();
    Json(json!({
        "privkey": export_key_base64(&sk),
        "pubkey": export_key_base64(&pk),
    }))
}

async fn submit(Form(input_data): Form<FormData>) -> Html<String> {
    let gameid = input_data.gameid.clone();
    let fleetid = input_data.fleetid.clone();
    let input_pubkey = input_data.pubkey.clone();
    let input_privkey = input_data.privkey.clone();
    let data = process_input_data(input_data);
    let pubkey = data.pubkey.clone();
    let privkey = data.privkey.clone();
    let random = data.random.clone();
    let board = data.board.clone();
    let shots = data.shots.clone();
    let response_text = match data.button.as_str() {
        "Join" => join_game(data).await,
        "Fire" => fire(data).await,
        "Report" => report(data).await,
        "Wave" => wave(data).await,
        "Win" => win(data).await,
        _ => "Unknown button pressed".to_string(),
    };

    render_html(
        pubkey,
        privkey,
        gameid,
        fleetid,
        random,
        board,
        shots,
        Some(response_text),
    )
    .await
}

async fn render_html(
    pubkey: Option<String>,
    privkey: Option<String>,
    gameid: Option<String>,
    fleetid: Option<String>,
    random: Option<String>,
    board: Option<String>,
    shots: Option<String>,
    response: Option<String>,
) -> Html<String> {
    let pubkey = pubkey.unwrap_or("".to_string());
    let privkey = privkey.unwrap_or("".to_string());
    let fleetid = fleetid.unwrap_or("".to_string());
    let gameid = gameid.unwrap_or("".to_string());
    let response_html = if let Some(response) = response {
        if response == "OK" {
            if gameid != "" {
                format!(
                    "Playing Game: <b>{}</b> with fleet's ID: <b>{}</b> ",
                    gameid, fleetid
                )
            } else {
                "Not in game".to_string()
            }
        } else {
            format!("<p style='color:red'>{}</p>", response)
        }
    } else {
        "".to_string()
    };
    let random = random.unwrap_or("".to_string());

    let board = board.unwrap_or("".to_string());
    let shots = shots.unwrap_or("".to_string());

    let path = "page.html";
    let html = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read {}: {}", path, e);
            return Html("Internal Server Error".to_string());
        }
    };
    // let html = std::fs::read_to_string(path).unwrap();
    let html = html.replace("{response_html}", &response_html);
    let html = html.replace("{gameid}", &gameid);
    let html = html.replace("{fleetid}", &fleetid);
    let html = html.replace("{random}", &random);
    let html = html.replace("{board}", &board);
    let html = html.replace("{shots}", &shots);
    let html = html.replace("{pubkey}", &pubkey);
    let html = html.replace("{privkey}", &privkey);

    Html(html)
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/submit", post(submit))
        .route("/generate_keys", get(generate_keys));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
