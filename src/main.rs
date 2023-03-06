use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router, Server};
use std::sync::{Arc, Mutex};
use byte_unit::Byte;
use sysinfo::{CpuExt, System, SystemExt};
use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};

type Snapshot = Vec<f32>;
type Memshot = Memory;

#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel::<Snapshot>(1);
    let (tx_mem, _) = broadcast::channel::<Memshot>(1);

    let app_state = AppState {
        tx: tx.clone(),
        tx_mem: tx_mem.clone()
    };

    let router = Router::new()
        .route("/", get(root_get))
        .route("/index.mjs", get(indexmjs_get))
        .route("/index.css", get(indexcss_get))
        .route("/realtime/cpus", get(realtime_cpus_get))
        .route("/realtime/memory", get(realtime_memory_get))
        .with_state(app_state.clone());

    // Update CPU usage in bg
    tokio::task::spawn_blocking(move || {
        let mut sys = System::new();
        loop {
            sys.refresh_memory();
            sys.refresh_cpu();
            let v: Vec<_> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
            let memory = init_memory(&sys);
            let _ = tx_mem.send(memory);
            let _ = tx.send(v);
            std::thread::sleep(System::MINIMUM_CPU_UPDATE_INTERVAL);
        }
    });

    let server = Server::bind(&"0.0.0.0:8082".parse().unwrap()).serve(router.into_make_service());
    let addr = server.local_addr();
    println!("Listening on {addr}");
    server.await.unwrap();
}

fn init_memory(sys: &System) -> Memory {
    Memory {
        total_mem: bytes_to_gygabites(sys.total_memory() as u128),
        available_mem:  bytes_to_gygabites(sys.available_memory() as u128),
        used_mem: bytes_to_gygabites(sys.used_memory() as u128),
        total_mem_used: bytes_to_gygabites(sys.used_memory() as u128 - sys.available_memory() as u128),
        total_swap: bytes_to_gygabites(sys.total_swap() as u128),
        used_swap: bytes_to_gygabites(sys.used_swap() as u128)
    }
}

fn bytes_to_gygabites(sys_mem: u128) -> String {
    let gb_total = Byte::from_bytes(sys_mem);
    let adjusted_byte = gb_total.get_appropriate_unit(false);
    adjusted_byte.to_string()
}

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<Snapshot>,
    tx_mem: broadcast::Sender<Memshot>
}

#[derive(Clone, Deserialize, Serialize)]
struct Memory {
    total_mem: String,
    available_mem: String,
    used_mem: String,
    total_mem_used: String,
    total_swap: String,
    used_swap: String,
}

#[axum::debug_handler]
async fn root_get() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("src/index.html").await.unwrap();
    Html(markup)
}

#[axum::debug_handler]
async fn indexmjs_get() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("src/index.mjs").await.unwrap();

    Response::builder()
        .header("content-type", "application/javascript;charset=utf-8")
        .body(markup)
        .unwrap()
}

#[axum::debug_handler]
async fn indexcss_get() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("src/index.css").await.unwrap();

    Response::builder()
        .header("content-type", "text/css;charset=utf-8")
        .body(markup)
        .unwrap()
}

#[axum::debug_handler]
async fn realtime_cpus_get(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |ws: WebSocket| async {
        realtime_cpus_stream(state, ws).await;
    })
}


#[axum::debug_handler]
async fn realtime_memory_get(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |ws: WebSocket| async {
        realtime_memory_stream(state, ws).await;
    })
}

async fn realtime_cpus_stream(app_state: AppState, mut ws: WebSocket) {
    let mut rx = app_state.tx.subscribe();

    while let Ok(msg) = rx.recv().await {
        let payload = serde_json::to_string(&msg).unwrap();
        ws.send(Message::Text(payload)).await.unwrap();
    }
}

async fn realtime_memory_stream(app_state: AppState, mut ws: WebSocket) {
    let mut rx = app_state.tx_mem.subscribe();

    while let Ok(msg) = rx.recv().await {

        let payload = serde_json::to_string(&msg).unwrap();
        ws.send(Message::Text(payload)).await.unwrap();
    }
}
