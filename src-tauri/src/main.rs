// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tokio::main]
async fn main() {
    // rusty_search_lib::crawler::run_crawler().await;
    return rusty_search_lib::run();
}
