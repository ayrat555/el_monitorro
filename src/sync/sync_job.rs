use crate::db;
use crate::db::feeds;
use crate::models::feed::Feed;
use crate::sync::feed_sync_job;
use chrono::offset::Utc;
use chrono::DateTime;

pub struct SyncJob {
    last_synced_at: DateTime<Utc>,
}

impl SyncJob {
    pub fn new(last_synced_at: DateTime<Utc>) -> Self {
        SyncJob { last_synced_at }
    }

    pub fn execute(&self) {
        let db_connection = db::establish_connection();

        let mut unsynced_feeds: Vec<Feed>;
        let mut page = 1;

        loop {
            unsynced_feeds =
                feeds::find_unsynced_feeds(&db_connection, self.last_synced_at, page, 100).unwrap();
            page += 1;

            for feed in &unsynced_feeds {
                feed_sync_job::enqueue_job(feed.id);
            }

            if unsynced_feeds == [] {
                break;
            }
        }
    }
}
