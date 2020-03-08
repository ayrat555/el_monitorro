use crate::db;
use crate::db::feed_items::NewFeedItem;
use crate::db::{feed_items, feeds};
use crate::models::feed::Feed;
use crate::sync::rss_reader::{FetchedFeed, ReadRSS, RssReader};
use background_jobs::{Backoff, Job, MaxRetries, Processor};
use chrono::offset::Utc;
use chrono::prelude::DateTime;
use diesel::result::Error as DieselError;
use diesel::Connection;
use diesel::PgConnection;
use failure::Error;
use rss::Channel;
use serde_derive::{Deserialize, Serialize};

const DEFAULT_QUEUE: &'static str = "default";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncJob {
    feed_id: i32,
}

#[derive(Clone, Debug)]
pub struct JobProcessor;

impl SyncJob {
    pub fn new(feed_id: i32) -> Self {
        SyncJob { feed_id }
    }

    pub fn execute(&self) {
        let db_connection = db::establish_connection();
        let feed = feeds::find_one(&db_connection, self.feed_id).unwrap();
        let rss_reader = RssReader {
            url: feed.link.clone(),
        };

        match rss_reader.read_rss() {
            Ok(fetched_feed) => {
                feed_items::create(&db_connection, feed.id, fetched_feed.items);
                ()
            }
            Err(err) => {
                feeds::set_error(&db_connection, &feed, &format!("{:?}", err));
                ()
            }
        };
    }
}

impl Job for SyncJob {
    type Processor = JobProcessor;
    type State = ();
    type Future = Result<(), Error>;

    fn run(self, _: Self::State) -> Self::Future {
        Ok(())
    }
}

impl Processor for JobProcessor {
    type Job = SyncJob;
    const NAME: &'static str = "JobProcessor";
    const QUEUE: &'static str = DEFAULT_QUEUE;
    const MAX_RETRIES: MaxRetries = MaxRetries::Count(2);
    const BACKOFF_STRATEGY: Backoff = Backoff::Exponential(2);
}
