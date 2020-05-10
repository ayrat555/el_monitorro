use crate::db;
use crate::db::telegram;
use crate::models::telegram_subscription::TelegramSubscription;
use diesel::result::Error;

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

        loop {
            current_subscriptions = telegram::fetch_subscriptions(&db_connection, page, 1000)?;

            page += 1;

            if current_subscriptions.is_empty() {
                break;
            }

            total_number += current_subscriptions.len();

            for subscription in current_subscriptions {
                tokio::spawn(deliver_updates(subscription));
            }
        }

        Ok(())
    }
}

async fn deliver_updates(subscription: TelegramSubscription) {}
