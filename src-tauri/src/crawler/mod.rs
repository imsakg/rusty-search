use spider::website::Website;
use spider::{configuration::Configuration, tokio};

use std::time::Duration;

use anyhow::Result;
use spider::auto_encoder::is_binary_file;
use spider::configuration::WaitForIdleNetwork;
use spider::hashbrown::HashMap;
use spider::tokio::sync::Mutex;
use spider_utils::spider_transformations::transformation::content::{
    transform_content, ReturnFormat, TransformConfig,
};
use std::sync::Arc;

#[derive(Debug)]
pub enum Content {
    Text(String),
    Binary,
}

pub async fn run_crawler(crawler: Arc<Mutex<Website>>) -> Result<HashMap<String, Option<Content>>> {
    crawler.lock().await.with_config(
        Configuration::new()
            .with_limit(10000)
            .with_block_assets(true)
            .with_caching(true)
            .with_stealth(true)
            .with_fingerprint(true)
            .with_wait_for_idle_network(Some(WaitForIdleNetwork::new(Some(Duration::from_millis(
                500,
            )))))
            .to_owned(),
    );

    let mut rx2 = crawler.lock().await.subscribe(0).unwrap();

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
            let mut map = map_guard.lock().await;

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

    crawler.lock().await.crawl_smart().await;
    crawler.lock().await.unsubscribe();
    let map = Arc::try_unwrap(map).unwrap().into_inner();
    anyhow::Result::Ok(map)
}
