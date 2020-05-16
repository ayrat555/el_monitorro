use crate::db;
use crate::sync::reader;
use crate::sync::reader::{FeedReaderError, FetchedFeed, FetchedFeedItem, ReadFeed};
use atom_syndication::Feed as AtomFeed;
use chrono::{DateTime, FixedOffset, Utc};
use std::str::FromStr;

pub struct AtomReader {
    pub url: String,
}

impl ReadFeed for AtomReader {
    fn read(&self) -> Result<FetchedFeed, FeedReaderError> {
        let body = reader::read_url(&self.url)?;

        match AtomFeed::from_str(&body) {
            Ok(atom_feed) => Ok(FetchedFeed::from(atom_feed)),
            Err(err) => {
                let msg = format!("{}", err);
                Err(FeedReaderError { msg })
            }
        }
    }
}

impl From<AtomFeed> for FetchedFeed {
    fn from(feed: AtomFeed) -> Self {
        let mut items = feed
            .entries()
            .into_iter()
            .filter(|item| item.links().first().is_some())
            .map(|item| {
                let base_date = match item.published() {
                    None => Some(item.updated()),
                    _ => item.published(),
                };

                let pub_date: DateTime<Utc> = parse_time(base_date);

                FetchedFeedItem {
                    title: Some(item.title().to_string()),
                    description: item.summary().map(|s| s.to_string()),
                    link: Some(item.links().first().unwrap().href().to_string()),
                    author: Some(
                        item.authors()
                            .into_iter()
                            .map(|person| person.name.to_string())
                            .collect::<Vec<String>>()
                            .join(", "),
                    ),
                    guid: Some(item.id().to_string()),
                    publication_date: pub_date,
                }
            })
            .collect::<Vec<FetchedFeedItem>>();

        items.dedup_by(|a, b| a.link == b.link && a.title == b.title);

        FetchedFeed {
            title: feed.title().to_string(),
            link: feed.links().first().unwrap().href().to_string(),
            description: feed
                .subtitle()
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            items: items,
        }
    }
}

fn parse_time(pub_date: Option<&DateTime<FixedOffset>>) -> DateTime<Utc> {
    match pub_date {
        None => db::current_time(),
        Some(date_time) => (*date_time).into(),
    }
}

#[cfg(test)]
mod tests {
    use super::{FetchedFeed, FetchedFeedItem};
    use atom_syndication::Feed as AtomFeed;
    use chrono::DateTime;
    use std::fs;
    use std::str::FromStr;

    #[test]
    fn it_converts_atom_feed_to_fetched_feed() {
        let xml_feed = fs::read_to_string("./tests/support/atom_feed_example.xml").unwrap();
        let channel = AtomFeed::from_str(&xml_feed).unwrap();

        let fetched_feed: FetchedFeed = channel.into();

        let expected_result = FetchedFeed {
            title: "Example Feed".to_string(),
            link: "http://example.org/".to_string(),
            description: "".to_string(),
            items: vec![FetchedFeedItem {
                title: Some("Atom-Powered Robots Run Amok".to_string()),
                description: Some("Some text.".to_string()),
                link: Some("http://example.org/2003/12/13/atom03".to_string()),
                author: Some("".to_string()),
                guid: Some("urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a".to_string()),
                publication_date: DateTime::parse_from_rfc3339("2003-12-13T18:30:02Z")
                    .unwrap()
                    .into(),
            }],
        };

        assert_eq!(expected_result, fetched_feed);
    }
}
