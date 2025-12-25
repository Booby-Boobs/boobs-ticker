use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, Emitter};
use tokio::time;
use rdev::{listen, Event, EventType};
use rand::seq::SliceRandom;


#[derive(Clone, serde::Serialize)]
struct TickerData {
    soul: f64,
    news: Vec<String>,
    total_time: u64, // in minutes
    keys_total: u64,
    clicks_total: u64,
    mouse_total: f64, // pixel distance
}

#[derive(Clone)]
struct AppState {
    soul: f64,
    last_activity: Instant,
    keys_pressed: u64,
    clicks: u64,
    mouse_moves: u64,
    news: Vec<String>,
    start_time: Instant,
    keys_total: u64,
    clicks_total: u64,
    mouse_total: f64, // pixel distance
    last_mouse_x: f64,
    last_mouse_y: f64,
}



#[tauri::command]
fn boost_energy(state: tauri::State<Arc<Mutex<AppState>>>) {
    let mut s = state.lock().unwrap();
    s.soul += 5.0;
    s.soul = s.soul.min(100.0);
}

#[tauri::command]
fn annoy_energy(state: tauri::State<Arc<Mutex<AppState>>>) {
    let mut s = state.lock().unwrap();
    s.soul -= 5.0;
    // No clamp here, allow below 0
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[tokio::main]
pub async fn run() {
    let mut news: Vec<String> = serde_json::from_str(include_str!("../news.json")).unwrap_or_default();
    let mut rng = rand::thread_rng();
    news.shuffle(&mut rng);

    let initial_energy: f64 = std::env::var("INITIAL_ENERGY")
        .unwrap_or_else(|_| "100".to_string())
        .parse()
        .unwrap_or(100.0);

    let state = Arc::new(Mutex::new(AppState {
        soul: initial_energy,
        last_activity: Instant::now(),
        keys_pressed: 0,
        clicks: 0,
        mouse_moves: 0,
        news,
        start_time: Instant::now(),
        keys_total: 0,
        clicks_total: 0,
        mouse_total: 0.0,
        last_mouse_x: 0.0,
        last_mouse_y: 0.0,
    }));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_positioner::init())
        .manage(state.clone())
        .invoke_handler(tauri::generate_handler![boost_energy, annoy_energy])
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let state_clone = state.clone();

            // Position window at bottom
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x: 0, y: 1040 }));
            }

            // Start monitoring
            tokio::spawn(async move {
                monitor_inputs(state_clone, app_handle).await;
            });

            // Tray
            let _tray = tauri::tray::TrayIconBuilder::new()
                .on_tray_icon_event(|_tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { .. } = event {
                        std::process::exit(0);
                    }
                })
                .build(app)
                .unwrap();

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn monitor_inputs(state: Arc<Mutex<AppState>>, app_handle: AppHandle) {
    let state_clone = state.clone();
    let app_handle_clone = app_handle.clone();

    // Input listener in separate thread
    std::thread::spawn(move || {
        let callback = move |event: Event| {
            let mut s = state_clone.lock().unwrap();
            match event.event_type {
                EventType::KeyPress(_) => {
                    s.keys_pressed += 1;
                    s.keys_total += 1;
                    s.last_activity = Instant::now();
                }
                EventType::ButtonPress(_) => {
                    s.clicks += 1;
                    s.clicks_total += 1;
                    s.last_activity = Instant::now();
                }
                EventType::MouseMove { x, y } => {
                    let x = x as f64;
                    let y = y as f64;
                    if s.last_mouse_x == 0.0 && s.last_mouse_y == 0.0 {
                        s.last_mouse_x = x;
                        s.last_mouse_y = y;
                    } else {
                        let dx = (x - s.last_mouse_x).abs();
                        let dy = (y - s.last_mouse_y).abs();
                        s.mouse_total += dx + dy; // Manhattan distance in pixels
                        s.last_mouse_x = x;
                        s.last_mouse_y = y;
                    }
                    s.mouse_moves += 1;
                    s.last_activity = Instant::now();
                }
                _ => {}
            }
        };
        match listen(callback) {
            Ok(_) => {},
            Err(e) => {
                eprintln!("Failed to listen to inputs: {:?}", e);
                std::process::exit(1);
            }
        }
    });

    // Ticker loop
    let mut interval = time::interval(Duration::from_millis(100));
    loop {
        interval.tick().await;
        let mut s = state.lock().unwrap();

            s.soul -= (s.keys_pressed as f64 * 1.0 + s.clicks as f64 * 1.0 + s.mouse_moves as f64 * 0.004) / 10.0; // key 10%, click 10%, mouse 4x

        s.soul = s.soul.clamp(0.0, 100.0); // cap between 0 and 100

        // Reset counters
        s.keys_pressed = 0;
        s.clicks = 0;
        s.mouse_moves = 0;



        let elapsed = s.start_time.elapsed().as_secs() / 60;
        let data = TickerData {
            soul: s.soul,
            news: s.news.clone(),
            total_time: elapsed,
            keys_total: s.keys_total,
            clicks_total: s.clicks_total,
            mouse_total: s.mouse_total,
        };

        app_handle_clone.emit("ticker-update", data).unwrap();
    }
}


