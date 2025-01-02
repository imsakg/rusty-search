use std::io::Read;
use std::{fs::File, io::Write};

use crawler::{run_crawler, Content};
use futures::StreamExt;
use spider::hashbrown::{HashMap, HashSet};
use spider::tokio;
use spider::website::Website;
use std::sync::{Arc, Mutex};
use tauri::{path::BaseDirectory, Manager};
use tauri::{Emitter, Listener};
use tauri_plugin_log::fern::colors::ColoredLevelConfig;
use tokio::time::Instant;

use serde::{Deserialize, Serialize};
use surrealdb::RecordId;
use surrealdb::Surreal;

use surrealdb::engine::local::{Db, SurrealKv};


use url::Url;

pub mod crawler;
pub mod types;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_sitelist(state: tauri::State<'_, Mutex<types::AppState>>) -> Vec<String> {
    let state = state.lock().unwrap();

    state.url_list.iter().map(|url| url.to_string()).collect()
}

#[tauri::command]
fn write_state_to_db(
    state: tauri::State<'_, Mutex<types::AppState>>,
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
async fn query_db(
    state: tauri::State<'_, Mutex<types::AppState>>,
    query: String,
) -> tauri::Result<Vec<types::QueryResult>> {
    let db = state.lock().unwrap().db.clone();
    let db = db.lock().await;
    let query_str = format!(r#"
        SELECT
            id,
            url.address AS address,
            search::highlight('<b>', '</b>', 1, true) AS highlight,
            search::score(0) * 2 + search::score(1) * 1 AS score
        FROM pages
        WHERE url.address @0@ '{query}'
        OR content @1@ '{query}'
        ORDER BY score DESC
        LIMIT 10;
    "#);

    let mut result = db.query(query_str).await.unwrap();
    let result: Vec<types::QueryResult> = result.take(0).unwrap();
    Ok(result)
}

#[tauri::command(async)]
async fn start_crawler(
    target_url: String,
    handle: tauri::AppHandle,
    reader: tauri::ipc::Channel<String>,
) -> tauri::Result<()> {
    let target_url = Url::parse(&target_url).unwrap();
    let website_rc = Arc::new(tokio::sync::Mutex::new(Website::new(target_url.as_str())));

    let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    let tx = Arc::new(tx);
    let tx2 = tx.clone();

    let app_handle = handle.clone();
    let website_handle = website_rc.clone();

    let crawler_task_handle = tokio::spawn(async move {
        let response = run_crawler(website_handle.clone()).await.unwrap();
        tx2.send("continue".to_string()).await.unwrap();
        response
    });

    let crawler_task_guard = Arc::new(tokio::sync::Mutex::new(crawler_task_handle));
    let crawler_task_guard2 = crawler_task_guard.clone();

    let event_id = handle.listen("status-changed", move |event| {
        let tx = tx.clone();
        tokio::spawn(async move {
            tx.send(event.payload().to_string()).await.unwrap();
        });
    });

    let website_handle = website_rc.clone();

    let shutdown_task_handler = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let msg = msg.trim_matches('"');
            let mut guard = website_handle.lock().await;
            match msg {
                "continue" => {
                    guard.unsubscribe();
                    guard.stop();
                    guard.clear_all().await;
                    break;
                }
                "shutdown" => {
                    println!("Shutting down crawler");
                    guard.unsubscribe();
                    guard.stop();
                    guard.clear_all().await;
                    crawler_task_guard2.lock().await.abort();
                    break;
                }
                _ => {
                    println!("Unknown message: {}", msg);
                }
            }
        }
    });

    println!("Starting crawler for {}", target_url);

    let t1 = Instant::now();
    shutdown_task_handler.await.unwrap();
    if let Ok(crawled) = Arc::try_unwrap(crawler_task_guard)
        .map_err(|e| anyhow::anyhow!("{:?}", e))
        .unwrap()
        .into_inner()
        .await
    {
        let tt = t1.elapsed();
        println!("Crawled {} pages in {:?}", crawled.len(), tt);

        for (url, content) in crawled.into_iter() {
            if let Some(content) = content {
            match content {
                Content::Text(text) => {
                    // save url to URLs table
                    let db = handle.state::<Mutex<types::AppState>>().lock().unwrap().db.clone();
                    let db = db.lock().await;
                    // check if URL already exists
                    
                    let response: Vec<types::Url> = db.query(format!( "SELECT * FROM urls WHERE address = '{}'", &url)).await.unwrap().take(0).unwrap();
                    
                    if let Some(url_record) = response.first() {
                        // url_id = url_record.clone().id.unwrap();
                        println!("URL already exists: {:?}", url_record);
                    } else {
                    let url = types::Url { id:None, address: url };
                    let response: Option<types::Record> = db.create("urls").content(url.clone()).await.unwrap();
                    let url_id = response.unwrap().id;

                    // save page to Pages table
                    let page = types::Page {
                        content: text,
                        id: None,
                        url: url_id
                    };
                    let response: Option<types::Record> = db.create("pages").content(page).await.unwrap();
                    if let Some(page_record) = response {
                        log::debug!("Saved Page: {:?}", page_record);
                    } else { 
                        log::error!("Failed to save page");
                    }
                }
                    }
                Content::Binary => {
                    log::warn!("{}: Binary content", url);
                }
            }
            } else {
                log::warn!("{}: No content", url);
            }
            
        }

    } else {
        println!("Crawler task failed");
    }

    // let visited_pages = website_rc.lock().await.get_all_links_visited().await;

    // for page in visited_pages.iter() {
    //     println!("{}", page);
    // }
    // println!("Visited {} pages", visited_pages.len());

    handle.unlisten(event_id);
    Ok(())
}

#[tauri::command]
fn add_new_url(state: tauri::State<'_, Mutex<types::AppState>>, url: String) -> tauri::Result<bool> {
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
            let db = futures::executor::block_on(async {
                // check if directory already exists
                let query;
                if let Ok(_) = tokio::fs::metadata("./db/").await {
                    log::debug!("Database directory exists");
                    query = None;
                }else {
                    query =Some( r#"
                        -- Define the URL table with a unique constraint on the address
                        DEFINE TABLE urls SCHEMAFULL;
                        DEFINE FIELD address ON TABLE urls TYPE string;
                        DEFINE INDEX urlAddressIndex ON TABLE urls COLUMNS address UNIQUE;

                        -- Define the Page table with a reference to the URL and content
                        DEFINE TABLE pages SCHEMAFULL;
                        DEFINE FIELD url ON TABLE pages TYPE record<urls>;
                        DEFINE FIELD content ON TABLE pages TYPE string;

                        -- Define the ScrapedPages table with a base URL and an array of pages
                        DEFINE TABLE scrapedPages SCHEMAFULL;
                        DEFINE FIELD base ON TABLE scrapedPages TYPE record<url>;
                        DEFINE FIELD pages ON TABLE scrapedPages TYPE array<record<page>>;

                        -- Define a full-text search analyzer
                        DEFINE ANALYZER custom_analyzer TOKENIZERS blank FILTERS lowercase;

                        -- Define full-text search indexes on the content and url fields of the Page table
                        DEFINE ANALYZER page_analyzer TOKENIZERS blank,class,camel,punct FILTERS lowercase, ngram(1,3);


                        DEFINE INDEX page_url ON pages FIELDS url.address SEARCH ANALYZER page_analyzer BM25(1.2,0.75);
                        DEFINE INDEX page_content ON pages FIELDS content SEARCH ANALYZER page_analyzer BM25(1.2,0.75) HIGHLIGHTS;
                    "#);
                }

                
                let db = Surreal::new::<SurrealKv>("./db/").await.unwrap();
                db.use_ns("test").use_db("test").await.unwrap();
                if let Some(query) = query {
                    db.query(query).await.unwrap();
                }

                Arc::new(tokio::sync::Mutex::new(db))
            });

            app.manage(Mutex::new(types::AppState {
                url_list: HashSet::new(),
                db,
            }));

            // Load sites from file
            {
                let resource_path = app
                    .path()
                    .resolve("resources/sitelist.txt", BaseDirectory::Resource)?;

                let mut buf: String = String::new();
                File::open(&resource_path)?.read_to_string(&mut buf)?;
                let urls: Vec<&str> = buf.split("\n").collect();
                let guard = app.state::<Mutex<types::AppState>>();
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
            start_crawler,
            query_db
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
