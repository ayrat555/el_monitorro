use crate::db::feed_items;
use fang::typetag;
use fang::FangError;
use fang::Queueable;
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

    pub fn run(&self) -> Result<(), FangError> {
        let conn = &mut crate::db::pool().get()?;

        if let Err(error) =
            feed_items::delete_old_feed_items(conn, self.feed_id, MESSAGES_LIMIT_PER_FEED)
        {
            log::error!(
                "Failed to delete old feed items for {}: {:?}",
                self.feed_id,
                error
            );
        };

        Ok(())
    }
}

#[typetag::serde]
impl Runnable for RemoveOldItemsJob {
    fn run(&self, _queue: &dyn Queueable) -> Result<(), FangError> {
        self.run()
    }

    fn uniq(&self) -> bool {
        true
    }

    fn task_type(&self) -> String {
        super::JOB_TYPE.to_string()
    }
}
