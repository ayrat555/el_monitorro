use crate::models::feed::Feed;
use crate::sync::rss_reader::ReadRSS;
use background_jobs::{Backoff, Job, MaxRetries, Processor};
use failure::Error;
use serde_derive::{Deserialize, Serialize};

use crate::db;
use crate::db::feeds;

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

    pub fn execute(&self) -> Feed {
        let connection = db::establish_connection();
        feeds::find_one(&connection, self.feed_id).unwrap()
        // let rss_reader = RssReader { url: feed.link };

        // let result = rss_reader.read_rss();

        // match result {
        //     Ok(data) ->

        // }
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
