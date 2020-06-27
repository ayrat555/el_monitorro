use self::atom::AtomReader;
use self::json::JsonReader;
use self::rss::RssReader;
use chrono::{DateTime, Utc};
use isahc::config::RedirectPolicy;
use isahc::prelude::*;
use std::io;
use std::time::Duration;

pub mod atom;
pub mod json;
pub mod rss;

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

pub fn read_url(url: &str) -> Result<Vec<u8>, FeedReaderError> {
    let client = match HttpClient::builder()
        .timeout(Duration::from_secs(5))
        .default_header("User-Agent", "el_monitorro/0.1.0")
        .redirect_policy(RedirectPolicy::Limit(10))
        .build()
    {
        Ok(cl) => cl,
        Err(er) => {
            let msg = format!("{:?}", er);

            return Err(FeedReaderError { msg });
        }
    };

    match client.get(url) {
        Ok(mut response) => {
            let mut writer: Vec<u8> = vec![];

            if let Err(err) = io::copy(response.body_mut(), &mut writer) {
                let msg = format!("{:?}", err);

                return Err(FeedReaderError { msg });
            }

            Ok(writer)
        }
        Err(error) => {
            let msg = format!("{:?}", error);

            Err(FeedReaderError { msg })
        }
    }
}

pub fn validate_rss_url(url: &str) -> Result<String, FeedReaderError> {
    let rss_reader = RssReader {
        url: url.to_string(),
    };

    if let Ok(_) = rss_reader.read() {
        return Ok("rss".to_string());
    }

    let atom_reader = AtomReader {
        url: url.to_string(),
    };

    if let Ok(_) = atom_reader.read() {
        return Ok("atom".to_string());
    }

    let json_reader = JsonReader {
        url: url.to_string(),
    };

    if let Ok(_) = json_reader.read() {
        return Ok("json".to_string());
    }

    Err(FeedReaderError {
        msg: "Url is not a feed".to_string(),
    })
}
