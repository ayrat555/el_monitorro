use crate::db;
use crate::db::{feed_items, feeds};
use crate::sync::rss_reader::{ReadRSS, RssReader};
use log::error;
use serde::{Deserialize, Serialize};

pub const DEFAULT_QUEUE: &'static str = "default";

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncJob {
    feed_id: i32,
}

#[derive(Debug, Fail, Serialize, Deserialize)]
pub enum SyncError {
    #[fail(display = "failed to sync a feed: {}", msg)]
    FeedError { msg: String },
    #[fail(display = "failed to insert data: {}", msg)]
    DbError { msg: String },
}

impl SyncJob {
    pub fn new(feed_id: i32) -> Self {
        SyncJob { feed_id }
    }

    pub fn execute(&self) -> Result<(), SyncError> {
        log::info!("Started processing a feed with id {}", self.feed_id);

        let db_connection = db::establish_connection();
        let feed = feeds::find_one(&db_connection, self.feed_id).unwrap();
        let rss_reader = RssReader {
            url: feed.link.clone(),
        };

        match rss_reader.read_rss() {
            Ok(fetched_feed) => {
                match feed_items::create(&db_connection, feed.id, fetched_feed.items) {
                    Err(err) => {
                        error!(
                            "Error: failed to create feed items for feed with id {}: {:?}",
                            self.feed_id, err
                        );

                        let error = SyncError::DbError {
                            msg: format!("Error: failed to create feed items {:?}", err),
                        };
                        Err(error)
                    }
                    _ => match feeds::set_synced_at(
                        &db_connection,
                        &feed,
                        Some(fetched_feed.title),
                        Some(fetched_feed.description),
                    ) {
                        Err(err) => {
                            error!(
                                "Error: failed to update synced_at for feed with id {}: {:?}",
                                self.feed_id, err
                            );
                            let error = SyncError::DbError {
                                msg: format!("Error: failed to update synced_at {:?}", err),
                            };
                            Err(error)
                        }
                        _ => {
                            log::info!("Successfully processed feed with id {}", self.feed_id);
                            Ok(())
                        }
                    },
                }
            }
            Err(err) => match feeds::set_error(&db_connection, &feed, &format!("{:?}", err)) {
                Err(err) => {
                    error!(
                        "Error: failed to set a sync error to feed with id {} {:?}",
                        self.feed_id, err
                    );
                    let error = SyncError::DbError {
                        msg: format!("Error: failed to set a sync error to feed {:?}", err),
                    };
                    Err(error)
                }
                _ => {
                    let error = SyncError::FeedError {
                        msg: format!("Error: failed to fetch feed items {:?}", err),
                    };
                    Err(error)
                }
            },
        }
    }
}

mod tests {
    use super::SyncJob;
    use crate::db;
    use crate::db::{feed_items, feeds};

    #[test]
    #[ignore]
    fn it_saves_rss_items() {
        let connection = db::establish_connection();
        let link = "https://www.feedforall.com/sample-feed.xml".to_string();

        let feed = feeds::create(&connection, link).unwrap();
        let sync_job = SyncJob { feed_id: feed.id };

        sync_job.execute().unwrap();

        let created_items = feed_items::find(&connection, feed.id).unwrap();

        assert_eq!(created_items.len(), 3);

        let updated_feed = feeds::find_one(&connection, feed.id).unwrap();
        assert!(updated_feed.synced_at.is_some());
        assert!(updated_feed.title.is_some());
        assert!(updated_feed.description.is_some());
    }
}
