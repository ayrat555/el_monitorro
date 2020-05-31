use crate::db;
use crate::db::{feed_items, feeds};
use crate::models::feed::Feed;
use crate::sync::reader::atom::AtomReader;
use crate::sync::reader::json::JsonReader;
use crate::sync::reader::rss::RssReader;
use crate::sync::reader::FeedReaderError;
use crate::sync::reader::ReadFeed;
use crate::sync::FetchedFeed;
use log::error;

#[derive(Debug)]
pub struct FeedSyncJob {
    feed_id: i64,
}

#[derive(Debug, Fail)]
pub enum FeedSyncError {
    #[fail(display = "failed to sync a feed: {}", msg)]
    FeedError { msg: String },
    #[fail(display = "failed to insert data: {}", msg)]
    DbError { msg: String },
}

impl FeedSyncJob {
    pub fn new(feed_id: i64) -> Self {
        FeedSyncJob { feed_id }
    }

    pub fn execute(&self) -> Result<(), FeedSyncError> {
        log::info!("Started processing a feed with id {}", self.feed_id);

        let db_connection = db::establish_connection();
        let feed = feeds::find(&db_connection, self.feed_id).unwrap();

        match read_feed(&feed) {
            Ok(fetched_feed) => {
                match feed_items::create(&db_connection, feed.id, fetched_feed.items) {
                    Err(err) => {
                        error!(
                            "Error: failed to create feed items for feed with id {}: {:?}",
                            self.feed_id, err
                        );

                        let error = FeedSyncError::DbError {
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
                            let error = FeedSyncError::DbError {
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
                    let error = FeedSyncError::DbError {
                        msg: format!("Error: failed to set a sync error to feed {:?}", err),
                    };
                    Err(error)
                }
                _ => {
                    let error = FeedSyncError::FeedError {
                        msg: format!("Error: failed to fetch feed items {:?}", err),
                    };
                    Err(error)
                }
            },
        }
    }
}

fn read_feed(feed: &Feed) -> Result<FetchedFeed, FeedReaderError> {
    if feed.feed_type == "rss".to_string() {
        RssReader {
            url: feed.link.clone(),
        }
        .read()
    } else if feed.feed_type == "atom".to_string() {
        AtomReader {
            url: feed.link.clone(),
        }
        .read()
    } else {
        JsonReader {
            url: feed.link.clone(),
        }
        .read()
    }
}

#[cfg(test)]
mod tests {
    use super::FeedSyncJob;
    use crate::db;
    use crate::db::{feed_items, feeds};

    #[test]
    #[ignore]
    fn it_saves_rss_items() {
        let connection = db::establish_connection();
        let link = "https://www.feedforall.com/sample-feed.xml".to_string();

        let feed = feeds::create(&connection, link, "rss".to_string()).unwrap();
        let sync_job = FeedSyncJob { feed_id: feed.id };

        sync_job.execute().unwrap();

        let created_items = feed_items::find(&connection, feed.id).unwrap();

        assert_eq!(created_items.len(), 3);

        let updated_feed = feeds::find(&connection, feed.id).unwrap();
        assert!(updated_feed.synced_at.is_some());
        assert!(updated_feed.title.is_some());
        assert!(updated_feed.description.is_some());
    }
}
