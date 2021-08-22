use crate::db::feed_items;
use fang::typetag;
use fang::Error as FangError;
use fang::PgConnection;
use fang::Runnable;
use serde::{Deserialize, Serialize};

const MESSAGES_LIMIT_PER_FEED: i64 = 1000;

#[derive(Serialize, Deserialize)]
pub struct RemoveOldItemsJob {
    pub feed_id: i64,
}

impl RemoveOldItemsJob {
    pub fn new(feed_id: i64) -> Self {
        Self { feed_id }
    }

    pub fn run(&self, db_connection: &PgConnection) {
        if let Err(error) =
            feed_items::delete_old_feed_items(db_connection, self.feed_id, MESSAGES_LIMIT_PER_FEED)
        {
            log::error!(
                "Failed to delete old feed items for {}: {:?}",
                self.feed_id,
                error
            );
        };
    }
}

#[typetag::serde]
impl Runnable for RemoveOldItemsJob {
    fn run(&self, connection: &PgConnection) -> Result<(), FangError> {
        self.run(connection);

        Ok(())
    }

    fn task_type(&self) -> String {
        super::JOB_TYPE.to_string()
    }
}
