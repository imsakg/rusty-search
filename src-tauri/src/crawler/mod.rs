use serde::{Deserialize, Serialize};
use spider::chromiumoxide::handler::blockers::intercept_manager::NetworkInterceptManager;
use spider::features::chrome_common::RequestInterceptConfiguration;
use spider::website::Website;
use spider::{configuration::Configuration, tokio};
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use tokio::sync::MutexGuard;

use std::time::{Duration, Instant};

use anyhow::Result;
use spider::auto_encoder::is_binary_file;
use spider::configuration::WaitForIdleNetwork;
use spider::hashbrown::HashMap;
use spider::tokio::sync::Mutex;
use spider_utils::spider_transformations::transformation::content::{
    transform_content, ReturnFormat, TransformConfig,
};
use std::sync::Arc;

use crate::types;

#[derive(Debug, Deserialize, Serialize)]
pub enum Content {
    Text(String),
    Binary,
}

pub async fn run_crawler(
    crawler: Arc<Mutex<Website>>,
    db: Arc<Mutex<Surreal<Db>>>,
) -> Result<Instant> {
    let t1 = std::time::Instant::now();
    let mut rx2 = crawler.lock().await.subscribe(0).unwrap();

    let task_handle = tokio::spawn(async move {
        let conf = TransformConfig {
            return_format: ReturnFormat::Markdown,
            readability: true,
            filter_images: true,
            clean_html: true,
            filter_svg: true,
            main_content: true,
        };
        let db = db.lock().await;
        while let Ok(res) = rx2.recv().await {
            let markup;

            if let Some(b) = res.get_bytes() {
                if b.len() > 1024 * 1024 || is_binary_file(b) {
                    // Some(Content::Binary);
                } else {
                    let response: Vec<types::Url> = db
                        .query(format!(
                            "SELECT * FROM urls WHERE address = '{}'",
                            &res.get_url()
                        ))
                        .await
                        .unwrap()
                        .take(0)
                        .unwrap();

                    if let Some(url_record) = response.first() {
                        // url_id = url_record.clone().id.unwrap();
                        println!("URL already exists: {:?}", url_record);
                    } else {
                        markup =
                            transform_content(&res, &conf, &Some("SHIFT_JIS".into()), &None, &None);

                        let url = types::Url {
                            id: None,
                            address: res.get_url().to_string(),
                        };

                        let t1 = std::time::Instant::now();
                        let response: Option<types::Record> =
                            db.create("urls").content(url.clone()).await.unwrap();
                        let url_id = response.unwrap().id;

                        // save page to Pages table
                        let page = types::Page {
                            content: markup,
                            id: None,
                            url: url_id,
                        };
                        let response: Option<types::Record> =
                            db.create("pages").content(page).await.unwrap();
                        if let Some(page_record) = response {
                            log::debug!("Saved Page: {:?}", page_record);
                        } else {
                            log::error!("Failed to save page");
                        }

                        println!("Saved URL: {:?} in {:?}", url, t1.elapsed());
                    }
                }
            } else {
                //  None::<Content>;
            }
        }
    });

    crawler.lock().await.crawl_smart().await;
    crawler.lock().await.unsubscribe();
    task_handle.await?;

    Ok(t1)
}
