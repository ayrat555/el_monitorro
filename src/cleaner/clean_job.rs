use super::RemoveOldItemsJob;
use crate::db::feeds;
use fang::typetag;
use fang::Error as FangError;
use fang::PgConnection;
use fang::Queue;
use fang::Runnable;
use serde::{Deserialize, Serialize};

const FEEDS_PER_PAGE: i64 = 500;

#[derive(Serialize, Deserialize)]
pub struct CleanJob {}

impl Default for CleanJob {
    fn default() -> Self {
        Self::new()
    }
}

impl CleanJob {
    pub fn new() -> Self {
        CleanJob {}
    }

    pub fn execute(&self, connection: &PgConnection) -> Result<(), FangError> {
        self.delete_feeds_without_subscriptions(connection);

        let mut current_feed_ids: Vec<i64>;
        let mut page = 1;
        let mut total_number = 0;

        loop {
            current_feed_ids = match feeds::load_feed_ids(connection, page, FEEDS_PER_PAGE) {
                Err(err) => {
                    let description = format!("{:?}", err);
                    return Err(FangError { description });
                }
                Ok(ids) => ids,
            };

            page += 1;

            if current_feed_ids.is_empty() {
                break;
            }

            total_number += current_feed_ids.len();

            for feed_id in current_feed_ids {
                Queue::push_task_query(connection, &RemoveOldItemsJob::new(feed_id)).unwrap();
            }
        }

        log::info!(
            "Finished enqueuing feeds for deletion of old feed items. Total Number:  {}",
            total_number
        );

        Ok(())
    }

    fn delete_feeds_without_subscriptions(&self, conn: &PgConnection) {
        log::info!("Started removing feeds without subscriptions");

        match feeds::delete_feeds_without_subscriptions(conn) {
            Ok(count) => log::info!("Removed {} feeds without subscriptions", count),
            Err(error) => log::error!("Failed to remove feeds without subscriptions {:?}", error),
        };
    }
}

#[typetag::serde]
impl Runnable for CleanJob {
    fn run(&self, connection: &PgConnection) -> Result<(), FangError> {
        self.execute(connection)
    }

    fn task_type(&self) -> String {
        super::JOB_TYPE.to_string()
    }
}
