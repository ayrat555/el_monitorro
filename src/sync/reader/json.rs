use crate::db;
use crate::sync::reader::{FeedReaderError, FetchedFeed, FetchedFeedItem, ReadFeed};
use chrono::{DateTime, Utc};
use feed_rs::model::Feed;
use feed_rs::parser;
use serde_json::Value;

pub struct JsonReader {
    pub url: String,
}

impl ReadFeed for JsonReader {
    fn read_from_bytes(&self, data: &[u8]) -> Result<FetchedFeed, FeedReaderError> {
        match serde_json::from_slice::<Value>(data) {
            Ok(_) => (),
            Err(err) => {
                let msg = format!("{:?}", err);
                return Err(FeedReaderError { msg });
            }
        }

        match parser::parse(data) {
            Ok(feed) => {
                let mut fetched_feed = FetchedFeed::from(feed);
                fetched_feed.link = self.url.clone();

                Ok(fetched_feed)
            }
            Err(err) => {
                let msg = format!("{:?}", err);
                Err(FeedReaderError { msg })
            }
        }
    }

    fn url(&self) -> String {
        self.url.clone()
    }
}

impl From<Feed> for FetchedFeed {
    fn from(feed: Feed) -> Self {
        let mut items = feed
            .entries
            .into_iter()
            .filter(|item| !item.links.is_empty())
            .map(|item| {
                let pub_date: DateTime<Utc> = parse_time(item.published, item.updated);
                FetchedFeedItem {
                    title: item.title.map_or_else(|| "".to_string(), |s| s.content),
                    description: item.summary.map(|s| s.content),
                    link: item.links.first().unwrap().href.clone(),
                    author: Some(
                        item.authors
                            .into_iter()
                            .map(|person| person.name)
                            .collect::<Vec<String>>()
                            .join(", "),
                    ),
                    guid: Some(item.id),
                    publication_date: pub_date,
                }
            })
            .collect::<Vec<FetchedFeedItem>>();

        items.dedup_by(|a, b| a.link == b.link && a.title == b.title);

        FetchedFeed {
            title: feed.title.map_or_else(|| "".to_string(), |s| s.content),
            description: feed
                .description
                .map_or_else(|| "".to_string(), |s| s.content),
            feed_type: "json".to_string(),
            link: "".to_string(),
            items,
        }
    }
}

fn parse_time(pub_date: Option<DateTime<Utc>>, updated: Option<DateTime<Utc>>) -> DateTime<Utc> {
    match pub_date {
        None => match updated {
            Some(value) => value,
            None => db::current_time(),
        },
        Some(value) => value,
    }
}

#[cfg(test)]
mod tests {
    use super::{FetchedFeed, FetchedFeedItem};
    use chrono::DateTime;
    use feed_rs::parser;
    use std::fs;

    #[test]
    fn it_converts_json_feed_to_fetched_feed() {
        let json_feed = fs::read_to_string("./tests/support/json_feed_example.json").unwrap();
        let feed = parser::parse(json_feed.as_bytes()).unwrap();

        let fetched_feed: FetchedFeed = feed.into();

        let expected_result = FetchedFeed { title: "World".to_string(), link: "".to_string(), description: "NPR world news, international art and culture, world business and financial markets, world economy, and global trends in health, science and technology. Subscribe to the World Story of the Day podcast and RSS feed.".to_string(), feed_type: "json".to_string(), items: vec![FetchedFeedItem { title: "Trump Says U.S. Will Withdraw From WHO. Does He Have The Authority To Do It?".to_string(), description: Some("In a press conference on Friday, the president said he would immediately sever ties — and funding — to the World Health Organization because of its relationship with China.".to_string()), link: "https://www.npr.org/sections/goatsandsoda/2020/05/29/865816855/trump-says-u-s-will-withdraw-from-who-does-he-have-the-authority-to-do-it?utm_medium=JSONFeed&utm_campaign=world".to_string(), author: Some("Pien Huang".to_string()), guid: Some("865816855".to_string()), publication_date: DateTime::parse_from_rfc3339("2020-05-29T23:30:03Z").unwrap().into() }, FetchedFeedItem { title: "France Eases Some Pandemic Restrictions And Will Reopen Restaurants, Bars And Parks".to_string(), description: Some("\"It will be so nice to be able to go lie on the grass in a park and have a picnic or to sit at a sidewalk cafe again,\" says a Paris resident. Restaurants and bars will reopen with restrictions June 2.".to_string()), link: "https://www.npr.org/sections/coronavirus-live-updates/2020/05/29/864892887/france-eases-some-pandemic-restrictions-and-will-reopen-restaurants-bars-and-par?utm_medium=JSONFeed&utm_campaign=world".to_string(), author: Some("Eleanor Beardsley".to_string()), guid: Some("864892887".to_string()), publication_date: DateTime::parse_from_rfc3339("2020-05-29T20:00:34Z").unwrap().into() }, FetchedFeedItem { title: "Moscow Doubles Last Month\'s Coronavirus Death Toll Amid Suspicions Of Undercounting".to_string(), description: Some("Media reports and analysts have questioned the accuracy of Russia\'s mortality figures for the virus. Moscow\'s Health Department now says 1,561 people died in April due to the coronavirus.".to_string()), link: "https://www.npr.org/sections/coronavirus-live-updates/2020/05/29/865044503/moscow-doubles-last-months-coronavirus-death-toll-amid-suspicions-of-undercounti?utm_medium=JSONFeed&utm_campaign=world".to_string(), author: Some("Jason Slotkin".to_string()), guid: Some("865044503".to_string()), publication_date: DateTime::parse_from_rfc3339("2020-05-29T19:35:00Z").unwrap().into() }] };

        assert_eq!(expected_result, fetched_feed);
    }
}
