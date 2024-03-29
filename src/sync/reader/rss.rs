use crate::db;
use crate::sync::reader::FeedReaderError;
use crate::sync::reader::FetchedFeed;
use crate::sync::reader::FetchedFeedItem;
use crate::sync::reader::ReadFeed;
use chrono::DateTime;
use chrono::Utc;
use rss::Channel;
use rss::Item;

pub struct RssReader {
    pub url: String,
}

impl ReadFeed for RssReader {
    fn read_from_bytes(&self, data: &[u8]) -> Result<FetchedFeed, FeedReaderError> {
        match Channel::read_from(data) {
            Ok(channel) => Ok(FetchedFeed::from(channel)),
            Err(err) => {
                let msg = format!("{err}");
                Err(FeedReaderError { msg })
            }
        }
    }

    fn url(&self) -> String {
        self.url.clone()
    }
}

impl From<Channel> for FetchedFeed {
    fn from(channel: Channel) -> Self {
        let mut items = channel
            .items()
            .iter()
            .filter(|item| item.link().is_some())
            .map(|item| {
                let pub_date: DateTime<Utc> = parse_time(item.pub_date());
                FetchedFeedItem {
                    title: item
                        .title()
                        .map_or_else(|| "".to_string(), |s| s.to_string()),
                    description: item.description().map(|s| s.to_string()),
                    link: item.link().unwrap().to_string(),
                    author: author(item),
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
            feed_type: "rss".to_string(),
            items,
        }
    }
}

fn author(item: &Item) -> Option<String> {
    let author = item.author().map(|s| s.to_string());

    if author.is_some() {
        return author;
    }

    if let Some(dublincore) = item.dublin_core_ext() {
        let joined_creators = dublincore.creators().join(", ").trim().to_string();

        if !joined_creators.is_empty() {
            return Some(joined_creators);
        }
    }

    None
}

fn parse_time(pub_date: Option<&str>) -> DateTime<Utc> {
    match pub_date {
        None => db::current_time(),
        Some(string) => match DateTime::parse_from_rfc2822(string) {
            Ok(date) => date.into(),
            Err(_) => db::current_time(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::FetchedFeed;
    use rss::Channel;
    use std::fs;
    use std::str::FromStr;

    #[test]
    fn it_converts_rss_channel_to_fetched_feed() {
        let xml_feed = fs::read_to_string("./tests/support/rss_feed_example.xml").unwrap();
        let channel = Channel::from_str(&xml_feed).unwrap();

        let fetched_feed: FetchedFeed = channel.into();

        assert_eq!(fetched_feed.title, "FeedForAll Sample Feed".to_string());
    }

    #[test]
    fn it_fetched_author_from_dublincore() {
        let xml_feed = fs::read_to_string("./tests/support/rss_dublincore_author.xml").unwrap();
        let channel = Channel::from_str(&xml_feed).unwrap();

        let fetched_feed: FetchedFeed = channel.into();

        assert_eq!(
            fetched_feed.items[0].author.as_ref().unwrap(),
            "@FabrizioRomano"
        );
    }
}
