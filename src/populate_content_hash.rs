use crate::db::feed_items;
use crate::models::feed_item::FeedItem;
use fang::typetag;
use fang::Error as FangError;
use fang::PgConnection;
use fang::Runnable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PopulateContentHashJob {}

impl Default for PopulateContentHashJob {
    fn default() -> Self {
        Self::new()
    }
}

impl PopulateContentHashJob {
    pub fn new() -> Self {
        PopulateContentHashJob {}
    }
}

#[typetag::serde]
impl Runnable for PopulateContentHashJob {
    fn run(&self, connection: &PgConnection) -> Result<(), FangError> {
        let mut current_feed_items: Vec<FeedItem>;
        let mut page = 1;

        log::info!("Started populating content hash");

        loop {
            current_feed_items =
                match feed_items::find_feed_items_without_content_hash(connection, page, 100) {
                    Ok(items) => {
                        for feed_item in &items {
                            if let Err(error) = feed_items::set_content_hash(connection, feed_item)
                            {
                                let description = format!("{:?}", error);

                                log::info!("Failed populating content hash {}", description);
                                return Err(FangError { description });
                            }
                        }

                        items
                    }
                    Err(error) => {
                        let description = format!("{:?}", error);

                        return Err(FangError { description });
                    }
                };

            page += 1;

            if current_feed_items.is_empty() {
                break;
            }
        }

        log::info!("Finished populating content hash");

        Ok(())
    }

    fn task_type(&self) -> String {
        "deliver".to_string()
    }
}
