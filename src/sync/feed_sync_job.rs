use crate::bot::telegram_client::Api;
use crate::db;
use crate::db::{feed_items, feeds, telegram};
use crate::models::feed::Feed;
use crate::sync::reader::atom::AtomReader;
use crate::sync::reader::json::JsonReader;
use crate::sync::reader::rss::RssReader;
use crate::sync::reader::FeedReaderError;
use crate::sync::reader::ReadFeed;
use crate::sync::FetchedFeed;
use chrono::Duration;
use diesel::pg::PgConnection;
use fang::typetag;
use fang::Error as FangError;
use fang::Runnable;
use log::error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct FeedSyncJob {
    feed_id: i64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FeedSyncError {
    FeedError { msg: String },
    DbError { msg: String },
    StaleError,
}

#[typetag::serde]
impl Runnable for FeedSyncJob {
    fn run(&self, connection: &PgConnection) -> Result<(), FangError> {
        self.sync_feed(connection);

        Ok(())
    }

    fn task_type(&self) -> String {
        "sync".to_string()
    }
}

impl FeedSyncJob {
    pub fn new(feed_id: i64) -> Self {
        FeedSyncJob { feed_id }
    }

    pub fn sync_feed(&self, db_connection: &PgConnection) {
        let feed_sync_result = self.execute(db_connection);

        match feed_sync_result {
            Err(FeedSyncError::StaleError) => {
                error!("Feed can not be processed for a long time {}", self.feed_id);

                self.remove_feed_and_notify_subscribers(db_connection);
            }
            Err(error) => error!("Failed to process feed {}: {:?}", self.feed_id, error),
            Ok(_) => (),
        }
    }

    fn remove_feed_and_notify_subscribers(&self, db_connection: &PgConnection) {
        let feed = feeds::find(db_connection, self.feed_id).unwrap();
        let chats = telegram::find_chats_by_feed_id(db_connection, self.feed_id).unwrap();

        let message = format!("{} can not be processed. It was removed.", feed.link);

        for chat in chats.into_iter() {
            match Api::send_message(chat.id, message.clone()) {
                Ok(_) => (),
                Err(error) => {
                    error!("Failed to send a message: {:?}", error);
                }
            }
        }

        match feeds::remove_feed(db_connection, self.feed_id) {
            Ok(_) => info!("Feed was removed: {}", self.feed_id),
            Err(err) => error!("Failed to remove feed: {} {}", self.feed_id, err),
        }
    }

    fn execute(&self, db_connection: &PgConnection) -> Result<(), FeedSyncError> {
        let feed = match feeds::find(db_connection, self.feed_id) {
            None => {
                let error = FeedSyncError::FeedError {
                    msg: format!("Error: feed not found {:?}", self.feed_id),
                };
                return Err(error);
            }
            Some(found_feed) => found_feed,
        };

        match read_feed(&feed) {
            Ok(fetched_feed) => {
                match feed_items::create(db_connection, feed.id, fetched_feed.items) {
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
                        db_connection,
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
                        _ => Ok(()),
                    },
                }
            }
            Err(err) => {
                let created_at_or_last_synced_at = if feed.synced_at.is_some() {
                    feed.synced_at.unwrap()
                } else {
                    feed.created_at
                };

                if db::current_time() - Duration::hours(6) < created_at_or_last_synced_at {
                    let error = set_error(db_connection, &feed, err);

                    Err(error)
                } else {
                    Err(FeedSyncError::StaleError)
                }
            }
        }
    }
}

fn set_error(
    db_connection: &PgConnection,
    feed: &Feed,
    sync_error: FeedReaderError,
) -> FeedSyncError {
    match feeds::set_error(db_connection, feed, &format!("{:?}", sync_error)) {
        Err(err) => {
            error!(
                "Error: failed to set a sync error to feed with id {} {:?}",
                feed.id, err
            );

            FeedSyncError::DbError {
                msg: format!("Error: failed to set a sync error to feed {:?}", err),
            }
        }
        _ => FeedSyncError::FeedError {
            msg: format!("Error: failed to fetch feed items {:?}", sync_error),
        },
    }
}

fn read_feed(feed: &Feed) -> Result<FetchedFeed, FeedReaderError> {
    match feed.feed_type.as_str() {
        "rss" => RssReader {
            url: feed.link.clone(),
        }
        .read(),

        "atom" => AtomReader {
            url: feed.link.clone(),
        }
        .read(),

        "json" => JsonReader {
            url: feed.link.clone(),
        }
        .read(),
        &_ => Err(FeedReaderError {
            msg: "Unknown feed type".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::FeedSyncError::FeedError;
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

        sync_job.execute(&connection).unwrap();

        let created_items = feed_items::find(&connection, feed.id).unwrap();

        assert_eq!(created_items.len(), 3);

        let updated_feed = feeds::find(&connection, feed.id).unwrap();
        assert!(updated_feed.synced_at.is_some());
        assert!(updated_feed.title.is_some());
        assert!(updated_feed.description.is_some());
    }

    #[test]
    fn it_returns_error_feed_is_not_found() {
        let connection = db::establish_connection();
        let sync_job = FeedSyncJob { feed_id: 5 };

        let result = sync_job.execute(&connection);

        assert_eq!(
            Err(FeedError {
                msg: "Error: feed not found 5".to_string()
            }),
            result
        );
    }
}
