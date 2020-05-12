use crate::db;
use chrono::offset::Utc;
use chrono::prelude::DateTime;
use mockall::*;
use rss::Channel;
use rss::Error;

pub struct RssReader {
    pub url: String,
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

#[automock]
pub trait ReadRSS {
    fn read_rss(&self) -> Result<FetchedFeed, Error>;
}

impl ReadRSS for RssReader {
    fn read_rss(&self) -> Result<FetchedFeed, Error> {
        match Channel::from_url(&self.url) {
            Ok(channel) => Ok(FetchedFeed::from(channel)),
            Err(err) => Err(err),
        }
    }
}

impl From<Channel> for FetchedFeed {
    fn from(channel: Channel) -> Self {
        let mut items = channel
            .items()
            .into_iter()
            .filter(|item| item.link().is_some())
            .map(|item| {
                let pub_date: DateTime<Utc> = parse_time(item.pub_date());
                FetchedFeedItem {
                    title: item.title().map(|s| s.to_string()),
                    description: item.description().map(|s| s.to_string()),
                    link: item.link().map(|s| s.to_string()),
                    author: item.author().map(|s| s.to_string()),
                    guid: item.guid().map(|s| s.value().to_string()),
                    publication_date: pub_date,
                }
            })
            .collect::<Vec<FetchedFeedItem>>();

        items.dedup_by(|a, b| a.link == b.link && a.title == b.title);

        FetchedFeed {
            title: channel.title().to_string(),
            link: channel.link().to_string(),
            description: channel.description().to_string(),
            items: items,
        }
    }
}

fn parse_time(pub_date: Option<&str>) -> DateTime<Utc> {
    match pub_date {
        None => db::current_time(),
        Some(string) => DateTime::from(DateTime::parse_from_rfc2822(string).unwrap()),
    }
}

#[cfg(test)]
mod tests {
    use super::{FetchedFeed, MockReadRSS, ReadRSS};
    use rss::Channel;
    use std::fs;
    use std::str::FromStr;

    #[test]
    fn it_mocks_read_rss_trait() {
        let mut mock = MockReadRSS::new();
        let xml_feed = fs::read_to_string("./tests/support/rss_feed_example.xml").unwrap();

        mock.expect_read_rss().returning(move || {
            let channel = Channel::from_str(&xml_feed).unwrap();
            Ok(FetchedFeed::from(channel))
        });

        assert!(mock.read_rss().is_ok());
    }

    // #[test]
    // fn parse_time_sucees_parses_string() {
    //     let string = Some("Tue, 12 May 2020 16:08:48 GMT".to_string());

    //     let result = super::parse_time(string);

    //     eprintln!("{}", result);

    //     assert!(result == db::current_time());
    // }
}
