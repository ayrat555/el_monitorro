use crate::db;
use crate::sync::reader::{FeedReaderError, FetchedFeed, FetchedFeedItem, ReadFeed};
use atom_syndication::Entry;
use atom_syndication::Feed as AtomFeed;
use atom_syndication::Link;
use chrono::{DateTime, FixedOffset, Utc};

pub struct AtomReader {
    pub url: String,
}

impl ReadFeed for AtomReader {
    fn read_from_bytes(&self, data: &[u8]) -> Result<FetchedFeed, FeedReaderError> {
        match AtomFeed::read_from(data) {
            Ok(atom_feed) => {
                let mut feed = FetchedFeed::from(atom_feed);

                if feed.link.is_empty() {
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

    fn url(&self) -> String {
        self.url.clone()
    }
}

impl From<AtomFeed> for FetchedFeed {
    fn from(feed: AtomFeed) -> Self {
        let mut items = feed
            .entries()
            .iter()
            .filter(|item| item.links().first().is_some())
            .map(|item| {
                let base_date = match item.published() {
                    None => Some(item.updated()),
                    _ => item.published(),
                };

                let pub_date: DateTime<Utc> = parse_time(base_date);

                FetchedFeedItem {
                    title: item.title().to_string(),
                    description: parse_description(item),
                    link: find_link(item.links(), "alternate")
                        .unwrap()
                        .href()
                        .to_string(),
                    author: Some(
                        item.authors()
                            .iter()
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
            link: find_link(feed.links(), "self")
                .map_or_else(|| "".to_string(), |s| s.href.to_string()),
            description: feed
                .subtitle()
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            items,
            feed_type: "atom".to_string(),
        }
    }
}

fn parse_description(item: &Entry) -> Option<String> {
    if let Some(value) = item.summary() {
        return Some(value.to_string());
    }

    if let Some(content) = item.content() {
        if let Some(value) = content.value() {
            return Some(value.to_string());
        }
    }

    None
}

fn find_link<'a>(links: &'a [Link], link_type: &str) -> Option<&'a Link> {
    let alternate_link = links.iter().find(|link| link.rel == link_type);

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

    #[test]
    fn it_uses_content_as_description_if_summarry_is_not_set() {
        let xml_feed = fs::read_to_string("./tests/support/atom_feed_content.xml").unwrap();
        let channel = AtomFeed::from_str(&xml_feed).unwrap();

        let fetched_feed: FetchedFeed = channel.into();
        let item = fetched_feed.items.first().unwrap();

        assert_eq!(item.description, Some("30/12/2020".to_string()));
    }
}
