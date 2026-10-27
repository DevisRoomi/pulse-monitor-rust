use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::Utc;
use serde::Serialize;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, Networks, RefreshKind, System};
use tower_http::cors::{Any, CorsLayer};

#[derive(Serialize, Clone)]
struct MetricSample {
    timestamp: String,
    cpu_global: f32,
    cpu_cores: Vec<f32>,
    mem_total_bytes: u64,
    mem_used_bytes: u64,
    swap_total_bytes: u64,
    swap_used_bytes: u64,
    net_in_bytes: u64,
    net_out_bytes: u64,
    uptime_secs: u64,
    process_count: usize,
    disks: Vec<DiskMetric>,
}

#[derive(Serialize, Clone)]
struct DiskMetric {
    mount_point: String,
    total_bytes: u64,
    available_bytes: u64,
}

#[derive(Serialize, Clone)]
struct TopProcess {
    pid: u32,
    name: String,
    cpu_usage: f32,
    mem_bytes: u64,
}

struct EngineState {
    sys: Mutex<System>,
    networks: Mutex<Networks>,
    disks: Mutex<Disks>,
}

#[tokio::main]
async fn main() {
    let mut sys = System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything()),
    );
    sys.refresh_all();

    let networks = Networks::new_with_refreshed_list();
    let disks = Disks::new_with_refreshed_list();

    let state = Arc::new(EngineState {
        sys: Mutex::new(sys),
        networks: Mutex::new(networks),
        disks: Mutex::new(disks),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/v1/ping", get(ping_handler))
        .route("/api/v1/metrics", get(metrics_handler))
        .route("/api/v1/processes", get(processes_handler))
        .route("/api/v1/stream", get(ws_handler))
        .layer(cors)
        .with_state(state);

    let bind_addr = "127.0.0.1:9099";
    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .expect("Failed to bind port 9099");

    println!("Pulse Rust Engine running on http://{}", bind_addr);
    axum::serve(listener, app).await.unwrap();
}

async fn ping_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "SYS_ONLINE",
        "engine": "Pulse Rust Kernel (sysinfo 0.30)",
        "utc": Utc::now().to_rfc3339()
    }))
}

fn sample_metrics(state: &EngineState) -> MetricSample {
    let mut sys = state.sys.lock().unwrap();
    let mut nets = state.networks.lock().unwrap();
    let mut disks = state.disks.lock().unwrap();

    sys.refresh_cpu();
    sys.refresh_memory();
    nets.refresh();
    disks.refresh();

    let mut net_in: u64 = 0;
    let mut net_out: u64 = 0;
    for (_, net_data) in nets.iter() {
        net_in += net_data.received();
        net_out += net_data.transmitted();
    }

    let disk_metrics = disks
        .iter()
        .map(|d| DiskMetric {
            mount_point: d.mount_point().to_string_lossy().to_string(),
            total_bytes: d.total_space(),
            available_bytes: d.available_space(),
        })
        .collect();

    let cores: Vec<f32> = sys.cpus().iter().map(|c| c.cpu_usage()).collect();

    MetricSample {
        timestamp: Utc::now().format("%H:%M:%S%.3f").to_string(),
        cpu_global: sys.global_cpu_info().cpu_usage(),
        cpu_cores: cores,
        mem_total_bytes: sys.total_memory(),
        mem_used_bytes: sys.used_memory(),
        swap_total_bytes: sys.total_swap(),
        swap_used_bytes: sys.used_swap(),
        net_in_bytes: net_in,
        net_out_bytes: net_out,
        uptime_secs: System::uptime(),
        process_count: sys.processes().len(),
        disks: disk_metrics,
    }
}

async fn metrics_handler(State(state): State<Arc<EngineState>>) -> impl IntoResponse {
    let sample = sample_metrics(&state);
    Json(sample)
}

async fn processes_handler(State(state): State<Arc<EngineState>>) -> impl IntoResponse {
    let mut sys = state.sys.lock().unwrap();
    sys.refresh_processes();

    let mut list: Vec<TopProcess> = sys
        .processes()
        .iter()
        .map(|(pid, proc_info)| TopProcess {
            pid: pid.as_u32(),
            name: proc_info.name().to_string(),
            cpu_usage: proc_info.cpu_usage(),
            mem_bytes: proc_info.memory(),
        })
        .collect();

    list.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
    list.truncate(20);

    Json(list)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<EngineState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<EngineState>) {
    loop {
        let sample = sample_metrics(&state);
        if let Ok(json_str) = serde_json::to_string(&sample) {
            if socket.send(Message::Text(json_str)).await.is_err() {
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}
