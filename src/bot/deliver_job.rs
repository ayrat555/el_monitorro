use crate::bot::api;
use crate::db;
use crate::db::telegram;
use crate::models::telegram_subscription::TelegramSubscription;
use diesel::result::Error;
use tokio::time;

pub struct DeliverJob {}

pub struct DeliverJobError {
    msg: String,
}

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

    if !feed_items.is_empty() {
        let messages = feed_items.into_iter().map(|item| {
            format!(
                "{}\n\n{}\n\n{}\n\n",
                item.title.unwrap_or("".to_string()),
                item.publication_date,
                item.link.unwrap_or("".to_string())
            )
        });

        for message in messages {
            match api::send_message(subscription.chat_id, message).await {
                Ok(_) => (),
                Err(error) => {
                    log::error!("Failed to deliver updates: {}", error);
                    return Err(DeliverJobError {
                        msg: format!("Failed to send updates : {}", error),
                    });
                }
            };
        }

        match telegram::set_subscription_delivered_at(&connection, &subscription) {
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
