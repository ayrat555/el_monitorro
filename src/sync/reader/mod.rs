use self::atom::AtomReader;
use self::rss::RssReader;
use crate::isahc::ResponseExt;
use chrono::{DateTime, Utc};

pub mod atom;
pub mod rss;

#[derive(Debug)]
pub struct FeedReaderError {
    pub msg: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FetchedFeedItem {
    pub title: Option<String>,
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

pub fn read_url(url: &str) -> Result<String, FeedReaderError> {
    match isahc::get(url) {
        Ok(mut response) => match response.text() {
            Ok(body) => Ok(body),
            Err(error) => {
                let msg = format!("{:?}", error);

                Err(FeedReaderError { msg })
            }
        },
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
    match rss_reader.read() {
        Ok(_) => Ok("rss".to_string()),
        Err(_) => {
            let atom_reader = AtomReader {
                url: url.to_string(),
            };

            match atom_reader.read() {
                Ok(_) => Ok("atom".to_string()),
                Err(_) => Err(FeedReaderError {
                    msg: "Url is not a feed".to_string(),
                }),
            }
        }
    }
}
