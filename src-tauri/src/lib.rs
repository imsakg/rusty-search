use std::io::Read;
use std::time::Duration;
use std::{fs::File, io::Write};

use anyhow::Result;
use crawler::run_crawler;
use spider::auto_encoder::is_binary_file;
use spider::configuration::{Configuration, WaitForIdleNetwork};
use spider::features::chrome_common::RequestInterceptConfiguration;
use spider::hashbrown::HashSet;
use spider::tokio;
use spider::website::Website;
use std::sync::{Arc, Mutex};
use tauri::{path::BaseDirectory, Manager};
use tauri::{Emitter, Listener};
use tauri_plugin_log::fern::colors::ColoredLevelConfig;
use tokio::io::AsyncWriteExt;
use tokio::time::Instant;

use spider_utils::spider_transformations::transformation::content::{
    transform_content, ReturnFormat, TransformConfig,
};

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

#[tauri::command(async)]
async fn start_crawler(
    target_url: String,
    handle: tauri::AppHandle,
    reader: tauri::ipc::Channel<String>,
) -> tauri::Result<()> {
    let target_url = Url::parse(&target_url).unwrap();

    log::debug!("Starting crawling with new url: {}", target_url);

    let crawled = run_crawler(&target_url.to_string()).await.unwrap();
    println!("{:?}", crawled);

    // let app_handle = handle.app_handle().clone();

    // let event_id = app_handle.listen("status-changed", move |event| {
    //     let website = website.clone();
    //     let app_handle2 = handle.app_handle().clone();
    //     let reader_guard = reader_guard.clone();
    //     let worker_task = worker_task.clone();
    //     tauri::async_runtime::spawn(async move {
    //         reader_guard
    //             .lock()
    //             .await
    //             .send("Crawler Stopped".to_string())
    //             .unwrap();
    //         worker_task.abort();
    //         website.lock().await.unsubscribe();
    //         website.lock().await.stop();
    //     });
    // });

    // app_handle.unlisten(event_id);
    Ok(())
}

#[tauri::command]
fn add_new_url(state: tauri::State<'_, Mutex<AppState>>, url: String) -> tauri::Result<bool> {
    if let Ok(url) = Url::parse(&url) {
        log::debug!("Parsed URL: {}", url);
        let mut state = state.lock().unwrap();
        if state.url_list.contains(&url) {
            log::debug!("URL already exists: {}", url);
            return Ok(false);
        }
        log::debug!("Added new URL: {}", &url);
        state.url_list.insert(url);
        Ok(true)
    } else {
        log::error!("Could not parse URL: {}", url);
        Ok(false)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    tauri::Builder::default()
        .setup(|app| {
            // if cfg!(debug_assertions) {
            //     app.handle().plugin(
            //         tauri_plugin_log::Builder::default()
            //             .with_colors(ColoredLevelConfig::new())
            //             .level(log::LevelFilter::Debug)
            //             .build(),
            //     )?;
            // }
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
                        log::debug!("Parsed URL loading: {}", url);
                        if state.url_list.insert(url) {
                            log::debug!("URL loaded");
                        } else {
                            log::warn!("URL already exists!");
                        }
                    } else {
                        log::error!("Could not parse URL: {}", url);
                    }
                }
                log::debug!("Loaded {} sites", state.url_list.len());
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
        .invoke_handler(tauri::generate_handler![
            greet,
            get_sitelist,
            add_new_url,
            start_crawler
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
