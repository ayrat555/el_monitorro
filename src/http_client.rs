use crate::config::Config;
use isahc::config::RedirectPolicy;
use isahc::prelude::*;
use isahc::HttpClient;
use std::sync::OnceLock;
use std::time::Duration;

static CLIENT: OnceLock<HttpClient> = OnceLock::new();

pub fn client() -> &'static HttpClient {
    CLIENT.get_or_init(init_client)
}

fn init_client() -> HttpClient {
    HttpClient::builder()
        .redirect_policy(RedirectPolicy::Limit(10))
        .timeout(request_timeout_seconds())
        .build()
        .unwrap()
}

fn request_timeout_seconds() -> Duration {
    let secs = Config::request_timeout_in_seconds();

    Duration::from_secs(secs)
}
