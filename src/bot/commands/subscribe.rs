use super::Command;
use super::Message;
use crate::db::feeds;
use crate::db::telegram;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use std::env;

static COMMAND: &str = "/subscribe";

struct Subscribe {}

#[derive(Debug, PartialEq)]
pub enum SubscriptionError {
    DbError(diesel::result::Error),
    InvalidUrl,
    UrlIsNotFeed,
    RssUrlNotProvided,
    SubscriptionAlreadyExists,
    SubscriptionCountLimit,
    TelegramError,
}

impl Subscribe {
    fn create_subscription(&self, db_connection: &PgConnection, url: String) -> String {
        let feed_type = self.validate_rss_url(&url)?;

        db_connection.transaction::<TelegramSubscription, SubscriptionError, _>(|| {
            let chat = telegram::create_chat(db_connection, new_chat).unwrap();
            let feed = feeds::create(db_connection, url, feed_type).unwrap();

            let new_telegram_subscription = NewTelegramSubscription {
                chat_id: chat.id,
                feed_id: feed.id,
            };

            self.check_if_subscription_exists(db_connection, new_telegram_subscription)?;
            self.check_number_of_subscriptions(db_connection, chat.id)?;

            let subscription =
                telegram::create_subscription(db_connection, new_telegram_subscription).unwrap();

            Ok(subscription)
        })
    }

    fn check_if_subscription_exists(
        &self,
        connection: &PgConnection,
        subscription: NewTelegramSubscription,
    ) -> Result<(), SubscriptionError> {
        match telegram::find_subscription(connection, subscription) {
            None => Ok(()),
            Some(_) => Err(SubscriptionError::SubscriptionAlreadyExists),
        }
    }

    fn check_number_of_subscriptions(
        &self,
        connection: &PgConnection,
        chat_id: i64,
    ) -> Result<(), SubscriptionError> {
        let result = telegram::count_subscriptions_for_chat(connection, chat_id);

        if result <= Self::sub_limit() {
            Ok(())
        } else {
            Err(SubscriptionError::SubscriptionCountLimit)
        }
    }

    pub fn sub_limit() -> i64 {
        let result = env::var("SUBSCRIPTION_LIMIT").unwrap_or_else(|_| "20".to_string());

        result.parse().unwrap()
    }
}

impl Command for Subscribe {
    fn response(
        &self,
        db_pool: Pool<ConnectionManager<PgConnection>>,
        message: &Message,
    ) -> String {
        match self.fetch_db_connection(db_pool) {
            Ok(connection) => {
                let text = message.text().unwrap();
                let argument = self.parse_argument(&text);
                self.create_subscription(connection, argument)
            }
            Err(error_message) => error_message,
        }
    }

    fn command(&self) -> &str {
        COMMAND
    }
}
