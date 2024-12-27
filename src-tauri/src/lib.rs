use std::fs::File;
use std::io::Read;

use std::sync::Mutex;
use tauri::{path::BaseDirectory, Manager};
use tauri_plugin_log::fern::colors::ColoredLevelConfig;
use url::Url;

pub mod crawler;

#[derive(Default)]
struct AppState {
    url_list: Vec<Url>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .with_colors(ColoredLevelConfig::new())
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            // Initialize the app state
            app.manage(Mutex::new(AppState::default()));

            // Load sites from file
            {
                let resource_path = app
                    .path()
                    .resolve("resources/sitelist.txt", BaseDirectory::Resource)?;

                let mut buf: String = String::new();
                File::open(&resource_path)?.read_to_string(&mut buf)?;
                let sites: Vec<&str> = buf.split("\n").collect();
                let guard = app.state::<Mutex<AppState>>();
                let mut state = guard.lock().unwrap();
                for site in sites {
                    if let Ok(url) = Url::parse(site) {
                        state.url_list.push(url);
                    } else {
                        log::error!("Could not parse URL: {}", site);
                    }
                }
                log::info!("Loaded {} sites", state.url_list.len());
            }

            // Window management
            {
                let window = app.get_webview_window("main").unwrap();
                window.center().unwrap();
            }

            // window.open_devtools();
            // window.close_devtools();
            // let handler_clone = app.handle().clone();
            // app.listen("track_stream", move |event| {
            //     let handler_clone = handler_clone.to_owned();
            //     let payload = event.payload();
            //     let parsed_payload: StartStreamPayload =
            //         serde_json::from_str(payload).expect("Could not parse track stream payload");
            //     tauri::async_runtime::spawn(async move {
            //         let _result =
            //             chat::setup_chat_monitor(&handler_clone, parsed_payload.streamer).await;
            //     });
            // });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
