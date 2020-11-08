use crate::db;
use crate::sync::reader;
use crate::sync::reader::{FeedReaderError, FetchedFeed, FetchedFeedItem, ReadFeed};
use atom_syndication::Feed as AtomFeed;
use atom_syndication::Link;
use chrono::{DateTime, FixedOffset, Utc};

pub struct AtomReader {
    pub url: String,
}

impl ReadFeed for AtomReader {
    fn read(&self) -> Result<FetchedFeed, FeedReaderError> {
        let body = reader::read_url(&self.url)?;

        match AtomFeed::read_from(&body[..]) {
            Ok(atom_feed) => {
                let mut feed = FetchedFeed::from(atom_feed);

                if feed.link == "".to_string() {
                    feed.link = self.url.clone();
                }

                Ok(feed)
            }
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
                    title: item.title().to_string(),
                    description: item.summary().map(|s| s.to_string()),
                    link: find_link(item.links()).unwrap().href().to_string(),
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
            link: find_link(feed.links()).map_or_else(|| "".to_string(), |s| s.href.to_string()),
            description: feed
                .subtitle()
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            items: items,
            feed_type: "atom".to_string(),
        }
    }
}

fn find_link<'a>(links: &'a [Link]) -> Option<&'a Link> {
    let alternate_link = links.into_iter().find(|link| link.rel == "alternate");

    if alternate_link.is_some() {
        alternate_link
    } else {
        links.first()
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
            feed_type: "atom".to_string(),
            items: vec![FetchedFeedItem {
                title: "Atom-Powered Robots Run Amok".to_string(),
                description: Some("Some text.".to_string()),
                link: "http://example.org/2003/12/13/atom03".to_string(),
                author: Some("".to_string()),
                guid: Some("urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a".to_string()),
                publication_date: DateTime::parse_from_rfc3339("2003-12-13T18:30:02Z")
                    .unwrap()
                    .into(),
            }],
        };

        assert_eq!(expected_result, fetched_feed);
    }

    #[test]
    fn it_uses_alternte_link() {
        let xml_feed = fs::read_to_string("./tests/support/atom_feed_alternat_link.xml").unwrap();
        let channel = AtomFeed::from_str(&xml_feed).unwrap();

        let fetched_feed: FetchedFeed = channel.into();
        let item = fetched_feed.items.first().unwrap();

        assert_eq!(item.link, "https://www.sekolahdasar.net/2020/11/latihan-soal-ulangan-pts-kelas-4-bahasa-jawa.html".to_string());
    }
}
