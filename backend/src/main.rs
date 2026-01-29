use axum::{routing::get, Json, Router};
use serde::Serialize;
use sysinfo::{System, Disks, Components};
use tower_http::cors::{Any, CorsLayer};
use std::net::SocketAddr;

#[derive(Serialize)]
struct DiskInfo {
    name: String,
    total_space: f64,     // Diubah ke f64 agar presisi untuk GB
    available_space: f64,
}

#[derive(Serialize)]
struct TempInfo {
    label: String,
    temperature: f32,
}

#[derive(Serialize)]
struct SystemStats {
    cpu_usage: f32,
    ram_used: u64,
    ram_total: u64,
    disks: Vec<DiskInfo>,
    temperatures: Vec<TempInfo>,
}

async fn get_stats() -> Json<SystemStats> {
    let mut sys = System::new_all();
    sys.refresh_all();

    // Mengambil data Disk
    let disks_data = Disks::new_with_refreshed_list();
    let disks = disks_data.iter().map(|disk| {
        DiskInfo {
            name: disk.name().to_string_lossy().into_owned(),
            // Mengonversi byte ke GB dengan presisi desimal
            total_space: disk.total_space() as f64 / 1024.0 / 1024.0 / 1024.0,
            available_space: disk.available_space() as f64 / 1024.0 / 1024.0 / 1024.0,
        }
    }).collect();

    // Mengambil data Suhu
    let components_list = Components::new_with_refreshed_list();
    let temperatures = components_list.iter().map(|comp| {
        TempInfo {
            label: comp.label().to_string(),
            temperature: comp.temperature(),
        }
    }).collect();

    let stats = SystemStats {
        cpu_usage: sys.global_cpu_info().cpu_usage(),
        ram_used: sys.used_memory() / 1024 / 1024, // Dalam MB
        ram_total: sys.total_memory() / 1024 / 1024,
        disks,
        temperatures,
    };

    Json(stats)
}

#[tokio::main]
async fn main() {
    // Setup CORS agar Svelte (biasanya port 5173) bisa mengakses API ini
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/stats", get(get_stats))
        .layer(cors);

    // Menjalankan di port 3001 sesuai permintaan kode Svelte kamu
    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    println!("🚀 Backend Advanced jalan di http://localhost:3001");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}