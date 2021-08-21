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
use diesel::prelude::*;
use diesel::result::Error;
use fang::typetag;
use fang::Error as FangError;
use fang::Runnable;
use log::error;
use serde::{Deserialize, Serialize};

const SYNC_FAILURE_LIMIT_IN_HOURS: i64 = 48;

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

impl From<Error> for FeedSyncError {
    fn from(error: Error) -> Self {
        let msg = format!("{:?}", error);

        FeedSyncError::DbError { msg }
    }
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

        let api = Api::default();

        for chat in chats.into_iter() {
            match api.send_text_message(chat.id, message.clone()) {
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

        match self.read_feed(&feed) {
            Ok(fetched_feed) => self.create_feed_items(db_connection, feed, fetched_feed),
            Err(err) => self.check_staleness(err, db_connection, feed),
        }
    }

    fn create_feed_items(
        &self,
        db_connection: &PgConnection,
        feed: Feed,
        fetched_feed: FetchedFeed,
    ) -> Result<(), FeedSyncError> {
        let previous_last_item = feed_items::get_latest_item(db_connection, self.feed_id);

        db_connection.transaction::<_, FeedSyncError, _>(|| {
            match feed_items::create(db_connection, feed.id, fetched_feed.items) {
                Err(err) => self.format_sync_error(err),
                _ => {
                    let new_last_item = feed_items::get_latest_item(db_connection, self.feed_id);

                    match (previous_last_item, new_last_item) {
                        (None, Some(_)) => {
                            telegram::set_subscriptions_has_updates(db_connection, feed.id)?;
                        }
                        (Some(actual_old), Some(actual_new)) => {
                            if !(actual_old.link == actual_new.link
                                && actual_old.title == actual_new.title)
                            {
                                telegram::set_subscriptions_has_updates(db_connection, feed.id)?;
                            }
                        }
                        (_, _) => (),
                    };

                    self.set_synced_at(
                        db_connection,
                        feed,
                        fetched_feed.title,
                        fetched_feed.description,
                    )
                }
            }
        })
    }

    fn format_sync_error(&self, err: Error) -> Result<(), FeedSyncError> {
        error!(
            "Error: failed to create feed items for feed with id {}: {:?}",
            self.feed_id, err
        );

        let error = FeedSyncError::DbError {
            msg: format!("Error: failed to create feed items {:?}", err),
        };
        Err(error)
    }

    fn set_synced_at(
        &self,
        db_connection: &PgConnection,
        feed: Feed,
        title: String,
        description: String,
    ) -> Result<(), FeedSyncError> {
        match feeds::set_synced_at(db_connection, &feed, Some(title), Some(description)) {
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
        }
    }

    fn check_staleness(
        &self,
        err: FeedReaderError,
        db_connection: &PgConnection,
        feed: Feed,
    ) -> Result<(), FeedSyncError> {
        let created_at_or_last_synced_at = if feed.synced_at.is_some() {
            feed.synced_at.unwrap()
        } else {
            feed.created_at
        };

        if db::current_time() - Duration::hours(SYNC_FAILURE_LIMIT_IN_HOURS)
            < created_at_or_last_synced_at
        {
            let error = self.set_error(db_connection, &feed, err);

            Err(error)
        } else {
            Err(FeedSyncError::StaleError)
        }
    }

    fn set_error(
        &self,
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

    fn read_feed(&self, feed: &Feed) -> Result<FetchedFeed, FeedReaderError> {
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
}

#[cfg(test)]
mod tests {
    use super::FeedSyncError::FeedError;
    use super::FeedSyncJob;
    use crate::db;
    use crate::db::{feed_items, feeds};
    use diesel::Connection;
    use mockito::mock;

    #[test]
    fn it_saves_rss_items() {
        let response = std::fs::read_to_string("./tests/support/rss_feed_example.xml").unwrap();
        let path = "/feed";
        let _m = mock("GET", path)
            .with_status(200)
            .with_body(response)
            .create();
        let link = format!("{}{}", mockito::server_url(), path);
        let connection = db::establish_connection();

        connection.test_transaction::<(), (), _>(|| {
            let feed = feeds::create(&connection, link, "rss".to_string()).unwrap();
            let sync_job = FeedSyncJob { feed_id: feed.id };

            sync_job.execute(&connection).unwrap();

            let created_items = feed_items::find(&connection, feed.id).unwrap();

            assert_eq!(created_items.len(), 9);

            let updated_feed = feeds::find(&connection, feed.id).unwrap();
            assert!(updated_feed.synced_at.is_some());
            assert!(updated_feed.title.is_some());
            assert!(updated_feed.description.is_some());

            Ok(())
        })
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
