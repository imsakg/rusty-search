use std::io::Read;
use std::{fs::File, io::Write};

use anyhow::Result;
use spider::hashbrown::HashSet;
use std::sync::Mutex;
use tauri::{path::BaseDirectory, Manager};
use tauri_plugin_log::fern::colors::ColoredLevelConfig;

use url::Url;

pub mod crawler;

#[derive(Default)]
struct AppState {
    url_list: HashSet<Url>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_sitelist(state: tauri::State<'_, Mutex<AppState>>) -> Vec<String> {
    let state = state.lock().unwrap();

    state.url_list.iter().map(|url| url.to_string()).collect()
}

#[tauri::command]
fn write_state_to_db(
    state: tauri::State<'_, Mutex<AppState>>,
    app_handle: tauri::AppHandle,
) -> bool {
    let state = state.lock().unwrap();
    let path = app_handle
        .path()
        .resolve("resources/sitelist.txt", BaseDirectory::Resource)
        .unwrap();
    let mut file = File::open(&path).unwrap();
    for url in state.url_list.iter() {
        file.write_all(url.to_string().as_bytes()).unwrap();
        file.write_all("\n".as_bytes()).unwrap();
    }
    true
}

#[tauri::command]
fn add_new_url(state: tauri::State<'_, Mutex<AppState>>, url: String) -> tauri::Result<bool> {
    let url = Url::parse(&url).unwrap();
    let mut state = state.lock().unwrap();
    if state.url_list.contains(&url) {
        log::info!("URL already exists: {}", url);
        return Ok(false);
    }
    log::info!("Added new URL: {}", &url);
    state.url_list.insert(url);
    Ok(true)
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
                let urls: Vec<&str> = buf.split("\n").collect();
                let guard = app.state::<Mutex<AppState>>();
                let mut state = guard.lock().unwrap();
                for url in urls {
                    if let Ok(url) = Url::parse(url) {
                        log::info!("Parsed URL loading: {}", url);
                        if state.url_list.insert(url) {
                            log::info!("URL loaded");
                        } else {
                            log::warn!("URL already exists!");
                        }
                    } else {
                        log::error!("Could not parse URL: {}", url);
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
        .invoke_handler(tauri::generate_handler![greet, get_sitelist, add_new_url])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
