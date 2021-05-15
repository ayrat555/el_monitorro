use crate::db;
use crate::db::{feed_items, feeds};
use diesel::result::Error;
use diesel::PgConnection;

pub struct CleanJob {}

impl Default for CleanJob {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CleanJobError {
    pub msg: String,
}

impl From<Error> for CleanJobError {
    fn from(error: Error) -> Self {
        let msg = format!("{:?}", error);

        CleanJobError { msg }
    }
}

impl CleanJob {
    pub fn new() -> Self {
        CleanJob {}
    }

    pub async fn execute(&self) -> Result<(), CleanJobError> {
        let semaphored_connection = db::get_semaphored_connection().await;
        let db_connection = semaphored_connection.connection;
        let mut current_feed_ids: Vec<i64>;
        let mut page = 1;
        let mut total_number = 0;

        delete_feeds_without_subscriptions(&db_connection);

        loop {
            current_feed_ids = feeds::load_feed_ids(&db_connection, page, 1000)?;

            page += 1;

            if current_feed_ids.is_empty() {
                break;
            }

            total_number += current_feed_ids.len();

            for feed_id in current_feed_ids {
                tokio::spawn(remove_old_feed_items(feed_id));
            }
        }

        log::info!(
            "Finished enqueuing feeds for deletion of old feed items. Total Number:  {}",
            total_number
        );

        Ok(())
    }
}

pub async fn remove_old_feed_items(feed_id: i64) {
    let semaphored_connection = db::get_semaphored_connection().await;
    let db_connection = semaphored_connection.connection;

    if let Err(error) = feed_items::delete_old_feed_items(&db_connection, feed_id, 1000) {
        log::error!(
            "Failed to delete old feed items for {}: {:?}",
            feed_id,
            error
        );
    }
}

fn delete_feeds_without_subscriptions(conn: &PgConnection) {
    log::info!("Started removing feeds without subscriptions");

    match feeds::delete_feeds_without_subscriptions(conn) {
        Ok(count) => log::info!("Removed {} feeds without subscriptions", count),
        Err(error) => log::error!("Failed to remove feeds without subscriptions {:?}", error),
    };
}
