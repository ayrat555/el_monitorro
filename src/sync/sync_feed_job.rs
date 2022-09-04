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
use diesel::result::Error;
use fang::typetag;
use fang::FangError;
use fang::Queueable;
use fang::Runnable;
use log::error;
use serde::{Deserialize, Serialize};

const SYNC_FAILURE_LIMIT_IN_HOURS: i64 = 48;

#[derive(Serialize, Deserialize, Debug)]
pub struct SyncFeedJob {
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

impl From<FeedSyncError> for FangError {
    fn from(error: FeedSyncError) -> Self {
        let msg = format!("{:?}", error);
        FangError { description: msg }
    }
}

#[typetag::serde]
impl Runnable for SyncFeedJob {
    fn run(&self, _queue: &dyn Queueable) -> Result<(), FangError> {
        let mut db_connection = crate::db::pool().get()?;

        self.sync_feed(&mut db_connection)
    }

    fn uniq(&self) -> bool {
        true
    }

    fn task_type(&self) -> String {
        super::JOB_TYPE.to_string()
    }
}

impl SyncFeedJob {
    pub fn new(feed_id: i64) -> Self {
        Self { feed_id }
    }

    pub fn sync_feed(&self, db_connection: &mut PgConnection) -> Result<(), FangError> {
        let feed_sync_result = self.execute(db_connection);

        match feed_sync_result {
            Err(FeedSyncError::StaleError) => {
                error!("Feed can not be processed for a long time {}", self.feed_id);

                self.remove_feed_and_notify_subscribers(db_connection)?;
            }
            Err(error) => error!("Failed to process feed {}: {:?}", self.feed_id, error),
            Ok(_) => (),
        };

        Ok(())
    }

    fn remove_feed_and_notify_subscribers(
        &self,
        db_connection: &mut PgConnection,
    ) -> Result<(), FangError> {
        let feed = feeds::find(db_connection, self.feed_id).ok_or(FeedSyncError::DbError {
            msg: "Feed not found :(".to_string(),
        })?;
        let chats = telegram::find_chats_by_feed_id(db_connection, self.feed_id)?;

        let message = format!("{} can not be processed. It was removed.", feed.link);

        let api = Api::default();

        for chat in chats.into_iter() {
            api.send_text_message(chat.id, message.clone())?;
        }

        feeds::remove_feed(db_connection, self.feed_id)?;
        Ok(())
    }

    fn execute(&self, db_connection: &mut PgConnection) -> Result<(), FeedSyncError> {
        let feed = feeds::find(db_connection, self.feed_id).ok_or(FeedSyncError::DbError {
            msg: "Feed not found :(".to_string(),
        })?;

        match self.read_feed(&feed) {
            Ok(fetched_feed) => self.maybe_upsert_feed_items(db_connection, feed, fetched_feed),
            Err(err) => self.check_staleness(err, db_connection, feed),
        }
    }

    fn maybe_upsert_feed_items(
        &self,
        db_connection: &mut PgConnection,
        feed: Feed,
        fetched_feed: FetchedFeed,
    ) -> Result<(), FeedSyncError> {
        if fetched_feed.items.is_empty() {
            return Ok(());
        }

        let last_item_in_db_option = feed_items::get_latest_item(db_connection, self.feed_id);
        let last_fetched_item = fetched_feed.items[0].clone();

        match last_item_in_db_option {
            None => {
                self.create_feed_items(db_connection, feed, fetched_feed)?;
            }
            Some(last_item_in_db) => {
                if last_fetched_item.publication_date >= last_item_in_db.publication_date
                    && last_fetched_item.link != last_item_in_db.link
                {
                    self.create_feed_items(db_connection, feed, fetched_feed)?;
                } else {
                    self.set_synced_at(
                        db_connection,
                        feed,
                        fetched_feed.title,
                        fetched_feed.description,
                    )?;
                }
            }
        }

        Ok(())
    }

    fn create_feed_items(
        &self,
        db_connection: &mut PgConnection,
        feed: Feed,
        fetched_feed: FetchedFeed,
    ) -> Result<(), FeedSyncError> {
        if let Err(err) = feed_items::create(db_connection, &feed, fetched_feed.items) {
            self.format_sync_error(err)
        } else {
            if let Some(last_item) = feed_items::get_latest_item(db_connection, self.feed_id) {
                telegram::set_subscriptions_has_updates(
                    db_connection,
                    feed.id,
                    last_item.created_at,
                )?;
            }

            self.set_synced_at(
                db_connection,
                feed,
                fetched_feed.title,
                fetched_feed.description,
            )
        }
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
        db_connection: &mut PgConnection,
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
        db_connection: &mut PgConnection,
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
        db_connection: &mut PgConnection,
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
    use super::FeedSyncError;
    use super::SyncFeedJob;
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
        let mut connection = db::establish_test_connection();

        connection.test_transaction::<(), (), _>(|connection| {
            let feed = feeds::create(connection, link, "rss".to_string()).unwrap();
            let sync_job = SyncFeedJob { feed_id: feed.id };

            sync_job.execute(connection).unwrap();

            let created_items = feed_items::find(connection, feed.id).unwrap();

            assert_eq!(created_items.len(), 9);

            let updated_feed = feeds::find(connection, feed.id).unwrap();
            assert!(updated_feed.synced_at.is_some());
            assert!(updated_feed.title.is_some());
            assert!(updated_feed.description.is_some());

            Ok(())
        })
    }

    #[test]
    fn it_returns_error_feed_is_not_found() {
        let mut connection = db::establish_test_connection();
        let sync_job = SyncFeedJob { feed_id: 5 };

        let result = sync_job.execute(&mut connection);

        assert_eq!(
            Err(FeedSyncError::DbError {
                msg: "Feed not found :(".to_string()
            }),
            result
        );
    }
}
