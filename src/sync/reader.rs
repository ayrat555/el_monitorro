use self::atom::AtomReader;
use self::json::JsonReader;
use self::rss::RssReader;
use chrono::{DateTime, Utc};
use dotenv::dotenv;
use once_cell::sync::OnceCell;
use std::env;
use std::io::BufRead;
use std::time::Duration;

pub mod atom;
pub mod json;
pub mod rss;

static REQUEST_TIMEOUT: OnceCell<u64> = OnceCell::new();

#[derive(Debug)]
pub struct FeedReaderError {
    pub msg: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FetchedFeedItem {
    pub title: String,
    pub description: Option<String>,
    pub link: String,
    pub author: Option<String>,
    pub guid: Option<String>,
    pub publication_date: DateTime<Utc>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FetchedFeed {
    pub title: String,
    pub link: String,
    pub description: String,
    pub feed_type: String,
    pub items: Vec<FetchedFeedItem>,
}

pub trait ReadFeed {
    fn read(&self) -> Result<FetchedFeed, FeedReaderError>;
}

pub fn read_url(url: &str) -> Result<impl std::io::Read + Send + BufRead, FeedReaderError> {
    match http_client().post(url).call() {
        Err(error) => {
            let msg = format!("{:?}", error);

            return Err(FeedReaderError { msg });
        }
        Ok(response) => Ok(std::io::BufReader::new(response.into_reader())),
    }
}

pub fn validate_rss_url(url: &str) -> Result<String, FeedReaderError> {
    let rss_reader = RssReader {
        url: url.to_string(),
    };

    if rss_reader.read().is_ok() {
        return Ok("rss".to_string());
    }

    let atom_reader = AtomReader {
        url: url.to_string(),
    };

    if atom_reader.read().is_ok() {
        return Ok("atom".to_string());
    }

    let json_reader = JsonReader {
        url: url.to_string(),
    };

    if json_reader.read().is_ok() {
        return Ok("json".to_string());
    }

    Err(FeedReaderError {
        msg: "Url is not a feed".to_string(),
    })
}

fn request_timeout_seconds() -> &'static u64 {
    REQUEST_TIMEOUT.get_or_init(|| {
        dotenv().ok();

        let timeout_str = env::var("REQUEST_TIMEOUT").unwrap_or_else(|_| "5".to_string());
        let timeout: u64 = timeout_str.parse().unwrap();

        timeout
    })
}

fn http_client() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(*request_timeout_seconds()))
        .redirects(10)
        .user_agent("el_monitorro/0.1.0")
        .build()
}
