pub mod rss;

use crate::db;
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct FeedReaderError {
    pub msg: String,
}

#[derive(Debug, Clone)]
pub struct FetchedFeedItem {
    pub title: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub author: Option<String>,
    pub guid: Option<String>,
    pub publication_date: DateTime<Utc>,
}

#[derive(Debug)]
pub struct FetchedFeed {
    pub title: String,
    pub link: String,
    pub description: String,
    pub items: Vec<FetchedFeedItem>,
}

pub trait ReadFeed {
    fn read(&self) -> Result<FetchedFeed, FeedReaderError>;
}

pub fn parse_time(pub_date: Option<&str>) -> DateTime<Utc> {
    match pub_date {
        None => db::current_time(),
        Some(string) => match DateTime::parse_from_rfc2822(string) {
            Ok(date) => date.into(),
            Err(_) => db::current_time(),
        },
    }
}
