use std::env;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, Emitter};
use tokio::time;
use device_query::{DeviceQuery, DeviceState, Keycode, MouseState};


#[derive(Clone, serde::Serialize)]
pub struct TickerData {
    pub soul: f64,
    pub total_time: u64, // in minutes
    pub keys_total: u64,
    pub clicks_total: u64,
    pub mouse_total: f64, // pixel distance
}

#[derive(Clone)]
pub struct AppState {
    pub soul: f64,
    pub last_activity: Instant,
    pub keys_pressed: u64,
    pub clicks: u64,
    pub mouse_moves: u64,
    pub start_time: Instant,
    pub keys_total: u64,
    pub clicks_total: u64,
    pub mouse_total: f64, // pixel distance
    pub last_mouse_x: f64,
    pub last_mouse_y: f64,
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
    let initial_energy: f64 = env::var("INITIAL_ENERGY")
        .unwrap_or_else(|_| "100".to_string())
        .parse()
        .unwrap_or(100.0);

    let state = Arc::new(Mutex::new(AppState {
        soul: initial_energy,
        last_activity: Instant::now(),
        keys_pressed: 0,
        clicks: 0,
        mouse_moves: 0,
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
        .plugin(tauri_plugin_macos_permissions::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
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

    // Input listener in separate thread using device_query
    std::thread::spawn(move || {
        let device_state = DeviceState::new();
        let mut last_keys: Vec<Keycode> = vec![];
        let mut last_mouse_state: Option<MouseState> = None;

        loop {
            let keys: Vec<Keycode> = device_state.get_keys();
            let mouse: MouseState = device_state.get_mouse();

            let mut s = state_clone.lock().unwrap();

            // Detect new key presses
            if keys.len() > last_keys.len() {
                s.keys_pressed += 1;
                s.keys_total += 1;
                s.last_activity = Instant::now();
            }

            // Detect mouse clicks
            if let Some(last_mouse) = &last_mouse_state {
                for &button in &[1, 2, 3] { // 1: Left, 2: Right, 3: Middle
                    let current_pressed = mouse.button_pressed.get(button).copied().unwrap_or(false);
                    let last_pressed = last_mouse.button_pressed.get(button).copied().unwrap_or(false);
                    if current_pressed && !last_pressed {
                        s.clicks += 1;
                        s.clicks_total += 1;
                        s.last_activity = Instant::now();
                    }
                }
            }

            // Detect mouse movement
            if let Some(last_mouse) = &last_mouse_state {
                let dx = (mouse.coords.0 as f64 - last_mouse.coords.0 as f64).abs();
                let dy = (mouse.coords.1 as f64 - last_mouse.coords.1 as f64).abs();
                if dx > 0.0 || dy > 0.0 {
                    s.mouse_total += dx + dy;
                    s.mouse_moves += 1;
                    s.last_activity = Instant::now();
                }
            }

            last_keys = keys;
            last_mouse_state = Some(mouse);

            drop(s); // Unlock mutex

            std::thread::sleep(Duration::from_millis(50));
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
            total_time: elapsed,
            keys_total: s.keys_total,
            clicks_total: s.clicks_total,
            mouse_total: s.mouse_total,
        };

        app_handle_clone.emit("ticker-update", data).unwrap();
    }
}


