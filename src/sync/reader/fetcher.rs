use crate::db;
use crate::sync::reader;
use crate::sync::reader::{FeedReaderError, FetchedFeed, FetchedFeedItem, ReadFeed};
use chrono::{DateTime, Utc};
use feed_rs::model::{Feed, FeedType};
use feed_rs::parser;

pub struct Fetcher {
    pub url: String,
}

impl ReadFeed for Fetcher {
    fn read(&self) -> Result<FetchedFeed, FeedReaderError> {
        let body = reader::read_url(&self.url)?;

        match parser::parse(&body[..]) {
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

        let feed_type = match feed.feed_type {
            FeedType::JSON => "json",
            FeedType::Atom => "atom",
            FeedType::RSS0 => "rss",
            FeedType::RSS1 => "rss",
            FeedType::RSS2 => "rss",
        };

        FetchedFeed {
            title: feed.title.map_or_else(|| "".to_string(), |s| s.content),
            description: feed
                .description
                .map_or_else(|| "".to_string(), |s| s.content),
            feed_type: feed_type.to_string(),
            link: "".to_string(),
            items: items,
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

        let expected_result = FetchedFeed {
            title: "World".to_string(),
            link: "".to_string(),
            description: "NPR world news, international art and culture, world business and financial markets, world economy, and global trends in health, science and technology. Subscribe to the World Story of the Day podcast and RSS feed.".into(),
            feed_type: "json".to_string(),
            items: vec![
                FetchedFeedItem {
                    title: "Trump Says U.S. Will Withdraw From WHO. Does He Have The Authority To Do It?".to_string(),
                    description: Some("In a press conference on Friday, the president said he would immediately sever ties — and funding — to the World Health Organization because of its relationship with China.".to_string()),
                    link: "https://www.npr.org/sections/goatsandsoda/2020/05/29/865816855/trump-says-u-s-will-withdraw-from-who-does-he-have-the-authority-to-do-it?utm_medium=JSONFeed&utm_campaign=world".to_string(),
                    author: Some("Pien Huang".to_string()),
                    guid: Some("865816855".to_string()),
                    publication_date: DateTime::parse_from_rfc3339("2020-05-29T23:30:03Z").unwrap().into() },
                FetchedFeedItem {
                    title: "France Eases Some Pandemic Restrictions And Will Reopen Restaurants, Bars And Parks".to_string(),
                    description: Some("\"It will be so nice to be able to go lie on the grass in a park and have a picnic or to sit at a sidewalk cafe again,\" says a Paris resident. Restaurants and bars will reopen with restrictions June 2.".to_string()),
                    link: "https://www.npr.org/sections/coronavirus-live-updates/2020/05/29/864892887/france-eases-some-pandemic-restrictions-and-will-reopen-restaurants-bars-and-par?utm_medium=JSONFeed&utm_campaign=world".to_string(),
                    author: Some("Eleanor Beardsley".to_string()),
                    guid: Some("864892887".to_string()),
                    publication_date: DateTime::parse_from_rfc3339("2020-05-29T20:00:34Z").unwrap().into() },
                FetchedFeedItem {
                    title: "Moscow Doubles Last Month\'s Coronavirus Death Toll Amid Suspicions Of Undercounting".to_string(),
                    description: Some("Media reports and analysts have questioned the accuracy of Russia\'s mortality figures for the virus. Moscow\'s Health Department now says 1,561 people died in April due to the coronavirus.".to_string()),
                    link: "https://www.npr.org/sections/coronavirus-live-updates/2020/05/29/865044503/moscow-doubles-last-months-coronavirus-death-toll-amid-suspicions-of-undercounti?utm_medium=JSONFeed&utm_campaign=world".to_string(),
                    author: Some("Jason Slotkin".to_string()),
                    guid: Some("865044503".to_string()),
                    publication_date: DateTime::parse_from_rfc3339("2020-05-29T19:35:00Z").unwrap().into() }
            ]
        };

        assert_eq!(expected_result, fetched_feed);
    }

    #[test]
    fn it_converts_rss_feed_to_fetched_feed() {
        let atom_feed = fs::read_to_string("./tests/support/rss_feed_example.xml").unwrap();
        let feed = parser::parse(atom_feed.as_bytes()).unwrap();

        let fetched_feed: FetchedFeed = feed.into();

        let expected_result = FetchedFeed {
            title: "FeedForAll Sample Feed".into(),
            link: "".into(),
            description: "RSS is a fascinating technology. The uses for RSS are expanding daily. Take a closer look at how various industries are using the benefits of RSS in their businesses.".into(),
            feed_type: "rss".into(),
            items: vec![
                FetchedFeedItem {
                    title: "RSS Solutions for Restaurants".into(),
                    description: Some("<b>FeedForAll </b>helps Restaurant\'s communicate with customers. Let your customers know the latest specials or events.<br>\n<br>\nRSS feed uses include:<br>\n<i><font color=\"#FF0000\">Daily Specials <br>\nEntertainment <br>\nCalendar of Events </i></font>".into()),
                    link: "http://www.feedforall.com/restaurant.htm".into(),
                    author: Some("".into()),
                    guid: Some("6d47a5c95ff5dfd84f2adb6d0d7adf18".into()),
                    publication_date: DateTime::parse_from_rfc3339("2004-10-19T15:09:11Z").unwrap().into() },
                FetchedFeedItem {
                    title: "RSS Solutions for Schools and Colleges".into(),
                    description: Some("FeedForAll helps Educational Institutions communicate with students about school wide activities, events, and schedules.<br>\n<br>\nRSS feed uses include:<br>\n<i><font color=\"#0000FF\">Homework Assignments <br>\nSchool Cancellations <br>\nCalendar of Events <br>\nSports Scores <br>\nClubs/Organization Meetings <br>\nLunches Menus </i></font>".into()),
                    link: "http://www.feedforall.com/schools.htm".into(),
                    author: Some("".into()),
                    guid: Some("d9c417b0f272d224ffe4e9c6d8e52a86".into()),
                    publication_date: DateTime::parse_from_rfc3339("2004-10-19T15:09:09Z").unwrap().into() },
                FetchedFeedItem {
                    title: "RSS Solutions for Computer Service Companies".into(),
                    description: Some("FeedForAll helps Computer Service Companies communicate with clients about cyber security and related issues. <br>\n<br>\nUses include:<br>\n<i><font color=\"#0000FF\">Cyber Security Alerts <br>\nSpecials<br>\nJob Postings </i></font>".into()),
                    link: "http://www.feedforall.com/computer-service.htm".into(),
                    author: Some("".into()),
                    guid: Some("f40702952140e6a4322ffdb1b5304fe".into()),
                    publication_date: DateTime::parse_from_rfc3339("2004-10-19T15:09:07Z").unwrap().into() }
            ]
        };

        assert_eq!(expected_result, fetched_feed);
    }

    #[test]
    fn it_converts_atom_feed_to_fetched_feed() {
        let atom_feed = fs::read_to_string("./tests/support/atom_feed_example.xml").unwrap();
        let feed = parser::parse(atom_feed.as_bytes()).unwrap();

        let fetched_feed: FetchedFeed = feed.into();

        let expected_result = FetchedFeed {
            title: "Example Feed".into(),
            link: "".into(),
            description: "".to_string(),
            feed_type: "atom".to_string(),
            items: vec![FetchedFeedItem {
                title: "Atom-Powered Robots Run Amok".to_string(),
                description: Some("Some text.".into()),
                link: "http://example.org/2003/12/13/atom03".into(),
                author: Some("".into()),
                guid: Some("urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a".into()),
                publication_date: DateTime::parse_from_rfc3339("2003-12-13T18:30:02Z")
                    .unwrap()
                    .into(),
            }],
        };

        assert_eq!(expected_result, fetched_feed);
    }
}
