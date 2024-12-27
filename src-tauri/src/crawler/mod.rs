use spider::website::Website;
use spider::{configuration, tokio};
use std::time::Instant;

const TARGET_SITE_URL: &str = "https://hackmd.io/@vprelovac/SJytVOEekx#Domains";

pub async fn run_crawler() {
    let config = configuration::Configuration::new().with_limit(10000);

    let mut website = Website::new(TARGET_SITE_URL);
    website.crawl().await;

    for link in website.get_links() {
        println!("- {:?}", link.as_ref());
    }
}
