use crate::db;
use crate::db::{feed_items, feeds};
use crate::sync::rss_reader::{ReadRSS, RssReader};
use dotenv::dotenv;
use izta::job::Job;
use izta::process_jobs;
use izta::runner::Runner;
use izta::task::task_req::TaskReq;
use log::error;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedSyncJob {
    feed_id: i32,
}

#[derive(Debug, Fail, Serialize, Deserialize)]
pub enum FeedSyncError {
    #[fail(display = "failed to sync a feed: {}", msg)]
    FeedError { msg: String },
    #[fail(display = "failed to insert data: {}", msg)]
    DbError { msg: String },
}

impl FeedSyncJob {
    pub fn new(feed_id: i32) -> Self {
        FeedSyncJob { feed_id }
    }

    pub fn execute(&self) -> Result<(), FeedSyncError> {
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

pub fn enqueue_job(number: i32) {
    dotenv().ok();
    let database_url =
        env::var("DATABASE_URL").expect("No DATABASE_URL environment variable found");
    let runner = Runner::new(process_jobs!(FeedSyncJob), &database_url, "tasks", vec![]);

    let task_req = TaskReq::new(FeedSyncJob::new(number));
    runner.add_task(&task_req);
}

impl Job for FeedSyncJob {
    type R = ();
    type E = FeedSyncError;

    // All jobs must have a UUID
    const UUID: &'static str = "74f3a15b-75c0-4889-9546-63b02ff304e4";

    const MAX_ATTEMPTS: usize = 3;

    // Job logic - return an `Err` for errors and `Ok` if successful.
    fn run(&self) -> Result<Self::R, Self::E> {
        self.execute()
    }
}

pub fn start_runner() {
    dotenv().ok();
    let database_url =
        env::var("DATABASE_URL").expect("No DATABASE_URL environment variable found");
    let runner = Runner::new(process_jobs!(FeedSyncJob), &database_url, "tasks", vec![]);

    runner.start();
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

        let feed = feeds::create(&connection, link).unwrap();
        let sync_job = FeedSyncJob { feed_id: feed.id };

        sync_job.execute().unwrap();

        let created_items = feed_items::find(&connection, feed.id).unwrap();

        assert_eq!(created_items.len(), 3);

        let updated_feed = feeds::find_one(&connection, feed.id).unwrap();
        assert!(updated_feed.synced_at.is_some());
        assert!(updated_feed.title.is_some());
        assert!(updated_feed.description.is_some());
    }
}
