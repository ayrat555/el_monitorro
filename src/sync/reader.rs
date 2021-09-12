use self::atom::AtomReader;
use self::json::JsonReader;
use self::rss::RssReader;
use crate::config::Config;
use chrono::{DateTime, Utc};
use isahc::config::RedirectPolicy;
use isahc::{prelude::*, Request};
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
    fn read(&self) -> Result<FetchedFeed, FeedReaderError> {
        let body = read_url(&self.url())?;

        self.read_from_bytes(&body)
    }

    fn read_from_bytes(&self, data: &[u8]) -> Result<FetchedFeed, FeedReaderError>;

    fn url(&self) -> String;
}

pub fn read_url(url: &str) -> Result<Vec<u8>, FeedReaderError> {
    let client = match Request::get(url)
        .timeout(Duration::from_secs(request_timeout_seconds()))
        .header("User-Agent", "el_monitorro/0.2.2")
        .redirect_policy(RedirectPolicy::Limit(10))
        .body(())
    {
        Ok(cl) => cl,
        Err(er) => {
            let msg = format!("{:?}", er);

            return Err(FeedReaderError { msg });
        }
    };

    match client.send() {
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
    let data = read_url(url)?;

    let rss_reader = RssReader {
        url: url.to_string(),
    };

    if rss_reader.read_from_bytes(&data).is_ok() {
        return Ok("rss".to_string());
    }

    let atom_reader = AtomReader {
        url: url.to_string(),
    };

    if atom_reader.read_from_bytes(&data).is_ok() {
        return Ok("atom".to_string());
    }

    let json_reader = JsonReader {
        url: url.to_string(),
    };

    if json_reader.read_from_bytes(&data).is_ok() {
        return Ok("json".to_string());
    }

    Err(FeedReaderError {
        msg: "Url is not a feed".to_string(),
    })
}

fn request_timeout_seconds() -> u64 {
    Config::request_timeout_in_seconds()
}
