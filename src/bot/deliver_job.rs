use crate::bot::api;
use crate::db;
use crate::db::telegram;
use crate::models::feed_item::FeedItem;
use crate::models::telegram_subscription::TelegramSubscription;
use chrono::offset::FixedOffset;
use chrono::{DateTime, Utc};

use diesel::result::Error;
use tokio::time;

pub struct DeliverJob {}

pub struct DeliverJobError {
    msg: String,
}

static BLOCKED_ERROR: &str = "Forbidden: bot was blocked by the user";

impl From<Error> for DeliverJobError {
    fn from(error: Error) -> Self {
        let msg = format!("{:?}", error);

        DeliverJobError { msg }
    }
}

impl DeliverJob {
    pub fn new() -> Self {
        DeliverJob {}
    }

    pub fn execute(&self) -> Result<(), DeliverJobError> {
        let db_connection = db::establish_connection();
        let mut current_subscriptions: Vec<TelegramSubscription>;
        let mut page = 1;
        let mut total_number = 0;

        log::info!("Started delivering feed items");

        loop {
            current_subscriptions = telegram::fetch_subscriptions(&db_connection, page, 1000)?;

            page += 1;

            if current_subscriptions.is_empty() {
                break;
            }

            total_number += current_subscriptions.len();

            for subscription in current_subscriptions {
                tokio::spawn(deliver_subscription_updates(subscription));
            }
        }

        log::info!(
            "Started checking delivery for {} subscriptions",
            total_number
        );

        Ok(())
    }
}

async fn deliver_subscription_updates(
    subscription: TelegramSubscription,
) -> Result<(), DeliverJobError> {
    let connection = db::establish_connection();
    let feed_items = telegram::find_undelivered_feed_items(&connection, &subscription)?;
    let undelivered_count = telegram::count_undelivered_feed_items(&connection, &subscription);
    let chat_id = subscription.chat_id;

    if feed_items.len() < undelivered_count as usize {
        let message = format!(
            "You have {} unread items, below {} last items",
            undelivered_count,
            feed_items.len()
        );

        match api::send_message(chat_id, message).await {
            Ok(_) => (),
            Err(error) => {
                let error_message = format!("{}", error);

                log::error!("Failed to deliver updates: {} {}", chat_id, error_message);

                if error_message == BLOCKED_ERROR {
                    match telegram::remove_chat(&connection, chat_id) {
                        Ok(_) => log::info!("Successfully removed chat {}", chat_id),
                        Err(error) => log::error!("Failed to remove a chat {}", error),
                    }
                };

                return Err(DeliverJobError {
                    msg: format!("Failed to send updates : {}", error),
                });
            }
        }
    }

    if !feed_items.is_empty() {
        let chat = telegram::find_chat(&connection, chat_id).unwrap();

        let offset = match chat.utc_offset_minutes {
            None => FixedOffset::west(0),
            Some(value) => {
                if value > 0 {
                    FixedOffset::east(value * 60)
                } else {
                    FixedOffset::west(value * 60)
                }
            }
        };

        let mut messages = feed_items
            .iter()
            .map(|item| {
                let date = item.publication_date.with_timezone(&offset);
                format!("{}\n\n{}\n\n{}\n\n", item.title, date, item.link)
            })
            .collect::<Vec<String>>();

        messages.reverse();

        for message in messages.into_iter() {
            match api::send_message(chat_id, message).await {
                Ok(_) => (),
                Err(error) => {
                    let error_message = format!("{}", error);

                    log::error!("Failed to deliver updates: {}", error_message);

                    if error_message == BLOCKED_ERROR {
                        match telegram::remove_chat(&connection, chat_id) {
                            Ok(_) => log::info!("Successfully removed chat {}", chat_id),
                            Err(error) => log::error!("Failed to remove a chat {}", error),
                        }
                    };

                    return Err(DeliverJobError {
                        msg: format!("Failed to send updates : {}", error),
                    });
                }
            };
        }

        match telegram::set_subscription_last_delivered_at(
            &connection,
            &subscription,
            get_max_publication_date(feed_items),
        ) {
            Ok(_) => (),
            Err(error) => {
                log::error!("Failed to set last_delivered_at: {}", error);
                return Err(DeliverJobError {
                    msg: format!("Failed to set last_delivered_at : {}", error),
                });
            }
        }
    }

    Ok(())
}

pub async fn deliver_updates() {
    let mut interval = time::interval(std::time::Duration::from_secs(60));
    loop {
        interval.tick().await;
        match DeliverJob::new().execute() {
            Err(error) => log::error!("Failed to send updates: {}", error.msg),
            Ok(_) => (),
        }
    }
}

fn get_max_publication_date(items: Vec<FeedItem>) -> DateTime<Utc> {
    items
        .into_iter()
        .max_by(|item1, item2| item1.publication_date.cmp(&item2.publication_date))
        .unwrap()
        .publication_date
}

#[cfg(test)]
mod tests {
    use crate::db;
    use crate::models::feed_item::FeedItem;
    use chrono::{DateTime, Utc};

    #[test]
    fn get_max_publication_date_finds_max_publication_date_in_feed_items_vector() {
        let feed_item1 = FeedItem {
            feed_id: 1,
            title: "".to_string(),
            description: None,
            link: "dsd".to_string(),
            author: None,
            guid: None,
            publication_date: DateTime::parse_from_rfc2822("Wed, 13 May 2020 15:54:02 EDT")
                .unwrap()
                .into(),
            created_at: db::current_time(),
            updated_at: db::current_time(),
        };

        let feed_item2 = FeedItem {
            feed_id: 1,
            title: "".to_string(),
            description: None,
            link: "dsd1".to_string(),
            author: None,
            guid: None,

            publication_date: DateTime::parse_from_rfc2822("Wed, 13 May 2020 13:54:02 EDT")
                .unwrap()
                .into(),
            created_at: db::current_time(),
            updated_at: db::current_time(),
        };

        let feed_items = vec![feed_item1, feed_item2];
        let result = super::get_max_publication_date(feed_items);

        let expected_result: DateTime<Utc> =
            DateTime::parse_from_rfc2822("Wed, 13 May 2020 15:54:02 EDT")
                .unwrap()
                .into();

        assert!(result == expected_result);
    }
}
