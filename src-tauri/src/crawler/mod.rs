use spider::website::Website;
use spider::{configuration::Configuration, tokio};

use std::time::Duration;

use spider::auto_encoder::is_binary_file;
use spider::configuration::WaitForIdleNetwork;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use spider::hashbrown::HashMap;
use spider_utils::spider_transformations::transformation::content::{
    transform_content, ReturnFormat, TransformConfig,
};

#[derive(Debug)]
pub enum Content {
    Text(String),
    Binary,
}

pub async fn run_crawler(target_url: &str) -> Result<HashMap<String, Option<Content>>> {
    let website_rc = Arc::new(tokio::sync::Mutex::new(Website::new(target_url)));
    let website = website_rc.clone();
    website.lock().await.with_config(
        Configuration::new()
            .with_limit(1000)
            .with_block_assets(true)
            .with_caching(true)
            .with_stealth(true)
            .with_fingerprint(true)
            .with_wait_for_idle_network(Some(WaitForIdleNetwork::new(Some(Duration::from_millis(
                500,
            )))))
            .to_owned(),
    );

    let mut rx2 = website.lock().await.subscribe(0).unwrap();

    let map = Arc::new(Mutex::new(HashMap::new()));

    let map_guard = map.clone();
    tokio::spawn(async move {
        let conf = TransformConfig {
            return_format: ReturnFormat::Markdown,
            readability: true,
            filter_images: true,
            clean_html: true,
            filter_svg: true,
            main_content: true,
        };
        while let Ok(res) = rx2.recv().await {
            let content;
            let markup;
            let mut map = map_guard.lock().unwrap();

            if let Some(b) = res.get_bytes() {
                if is_binary_file(b) || b.len() > 1024 * 1024 {
                    content = Some(Content::Binary);
                } else {
                    markup =
                        transform_content(&res, &conf, &Some("SHIFT_JIS".into()), &None, &None);
                    content = Some(Content::Text(markup));
                }
                map.insert(res.get_url().to_string(), content);
            } else {
                content = None::<Content>;
                map.insert(res.get_url().to_string(), content);
            }
        }
    });

    website.lock().await.crawl_smart().await;
    website.lock().await.unsubscribe();
    let map = Arc::try_unwrap(map).unwrap().into_inner().unwrap();
    anyhow::Result::Ok(map)
}
