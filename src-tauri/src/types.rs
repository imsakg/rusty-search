use serde::{Deserialize, Serialize};
use spider::hashbrown::HashSet;
use std::sync::Arc;
use surrealdb::sql::Thing;
use surrealdb::{engine::local::Db, RecordId, Surreal};
use tokio::sync::Mutex;

#[derive(Debug, Deserialize)]
pub struct Record {
    #[allow(dead_code)]
    pub id: Thing,
}

// Define the URL structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Url {
    pub id: Option<Thing>,
    pub address: String,
}

// Define the Page structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    pub id: Option<Thing>,
    pub content: String,
    pub url: Thing,
}

pub struct AppState {
    pub url_list: HashSet<url::Url>,
    pub db: Arc<Mutex<Surreal<Db>>>,
}
