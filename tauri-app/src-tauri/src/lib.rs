use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, Emitter};
use tokio::time;
use rdev::{listen, Event, EventType};


#[derive(Clone, serde::Serialize)]
struct TickerData {
    soul: f64,
    news: Vec<String>,
}

#[derive(Clone)]
struct AppState {
    soul: f64,
    last_activity: Instant,
    keys_pressed: u64,
    clicks: u64,
    mouse_moves: u64,
    news: Vec<String>,
}



#[tauri::command]
fn boost_energy(state: tauri::State<Arc<Mutex<AppState>>>) {
    let mut s = state.lock().unwrap();
    s.soul += 5.0;
    s.soul = s.soul.min(100.0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[tokio::main]
pub async fn run() {
    let news: Vec<String> = serde_json::from_str(include_str!("../news.json")).unwrap_or_default();

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
    }));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_positioner::init())
        .manage(state.clone())
        .invoke_handler(tauri::generate_handler![boost_energy])
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
                    s.last_activity = Instant::now();
                }
                EventType::ButtonPress(_) => {
                    s.clicks += 1;
                    s.last_activity = Instant::now();
                }
                EventType::MouseMove { .. } => {
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
        let now = Instant::now();
        let inactive_duration = now.duration_since(s.last_activity);

            s.soul -= (s.keys_pressed as f64 * 0.2 + s.clicks as f64 * 1.0 + s.mouse_moves as f64 * 0.001) / 10.0; // key 2%, click 10%

        s.soul = s.soul.clamp(0.0, 100.0); // cap between 0 and 100

        // Reset counters
        s.keys_pressed = 0;
        s.clicks = 0;
        s.mouse_moves = 0;



        let data = TickerData {
            soul: s.soul,
            news: s.news.clone(),
        };

        app_handle_clone.emit("ticker-update", data).unwrap();
    }
}


