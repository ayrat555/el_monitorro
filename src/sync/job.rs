use crate::db;
// use crate::db::feed_items::NewFeedItem;
use crate::db::{feed_items, feeds};
// use crate::models::feed::Feed;
use crate::sync::rss_reader::{ReadRSS, RssReader};
use background_jobs::{Backoff, Job, MaxRetries, Processor};
// use chrono::offset::Utc;
// use chrono::prelude::DateTime;
// use diesel::result::Error as DieselError;
// use diesel::Connection;
// use diesel::PgConnection;
use failure::Error;
// use rss::Channel.
use log::error;
use serde_derive::{Deserialize, Serialize};

const DEFAULT_QUEUE: &'static str = "default";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncJob {
    feed_id: i32,
}

#[derive(Clone, Debug)]
pub struct JobProcessor;

#[derive(Debug, Fail)]
enum SyncError {
    #[fail(display = "failed to sync a feed: {}", msg)]
    FeedError { msg: String },
    #[fail(display = "failed to insert data: {}", msg)]
    DbError { msg: String },
}

impl SyncJob {
    pub fn new(feed_id: i32) -> Self {
        SyncJob { feed_id }
    }

    pub fn execute(&self) -> Result<(), Error> {
        let db_connection = db::establish_connection();
        let feed = feeds::find_one(&db_connection, self.feed_id).unwrap();
        let rss_reader = RssReader {
            url: feed.link.clone(),
        };

        match rss_reader.read_rss() {
            Ok(fetched_feed) => {
                match feed_items::create(&db_connection, feed.id, fetched_feed.items) {
                    Err(err) => {
                        error!("Error: failed to create feed items {:?}", err);
                        let error = SyncError::DbError {
                            msg: format!("Error: failed to create feed items {:?}", err),
                        };
                        Err(error.into())
                    }
                    _ => Ok(()),
                }
            }
            Err(err) => match feeds::set_error(&db_connection, &feed, &format!("{:?}", err)) {
                Err(err) => {
                    error!("Error: failed to set a sync error to feed {:?}", err);
                    let error = SyncError::DbError {
                        msg: format!("Error: failed to set a sync error to feed {:?}", err),
                    };
                    Err(error.into())
                }
                _ => {
                    let error = SyncError::FeedError {
                        msg: format!("Error: failed to fetch feed items {:?}", err),
                    };
                    Err(error.into())
                }
            },
        }
    }
}

impl Job for SyncJob {
    type Processor = JobProcessor;
    type State = ();
    type Future = Result<(), Error>;

    fn run(self, _: Self::State) -> Self::Future {
        self.execute()
    }
}

impl Processor for JobProcessor {
    type Job = SyncJob;
    const NAME: &'static str = "JobProcessor";
    const QUEUE: &'static str = DEFAULT_QUEUE;
    const MAX_RETRIES: MaxRetries = MaxRetries::Count(2);
    const BACKOFF_STRATEGY: Backoff = Backoff::Exponential(2);
}
