use super::Command;
use super::Message;
use crate::db::feeds;
use crate::db::telegram;
use crate::db::telegram::NewTelegramSubscription;
use crate::models::telegram_subscription::TelegramSubscription;
use crate::sync::reader;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::Connection;
use diesel::PgConnection;
use std::env;
use url::Url;

static COMMAND: &str = "/subscribe";

pub struct Subscribe {}

#[derive(Debug, PartialEq)]
enum SubscriptionError {
    DbError(diesel::result::Error),
    InvalidUrl,
    UrlIsNotFeed,
    SubscriptionAlreadyExists,
    SubscriptionCountLimit,
}

impl From<diesel::result::Error> for SubscriptionError {
    fn from(error: diesel::result::Error) -> Self {
        SubscriptionError::DbError(error)
    }
}

impl Subscribe {
    fn subscribe(&self, db_connection: &PgConnection, message: &Message, url: String) -> String {
        match self.create_subscription(db_connection, message, url.clone()) {
            Ok(_subscription) => format!("Successfully subscribed to {}", url),
            Err(SubscriptionError::DbError(_)) => {
                "Something went wrong with the bot's storage".to_string()
            }
            Err(SubscriptionError::InvalidUrl) => "Invalid url".to_string(),
            Err(SubscriptionError::UrlIsNotFeed) => "Url is not a feed".to_string(),
            Err(SubscriptionError::SubscriptionAlreadyExists) => {
                "The subscription already exists".to_string()
            }
            Err(SubscriptionError::SubscriptionCountLimit) => {
                "You exceeded the number of subscriptions".to_string()
            }
        }
    }

    fn create_subscription(
        &self,
        db_connection: &PgConnection,
        message: &Message,
        url: String,
    ) -> Result<TelegramSubscription, SubscriptionError> {
        let feed_type = self.validate_rss_url(&url)?;

        db_connection.transaction::<TelegramSubscription, SubscriptionError, _>(|| {
            let chat = telegram::create_chat(db_connection, message.chat().into()).unwrap();
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
    fn validate_rss_url(&self, rss_url: &str) -> Result<String, SubscriptionError> {
        match Url::parse(rss_url) {
            Ok(_) => match reader::validate_rss_url(rss_url) {
                Ok(feed_type) => Ok(feed_type),
                _ => Err(SubscriptionError::UrlIsNotFeed),
            },
            _ => Err(SubscriptionError::InvalidUrl),
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

    fn sub_limit() -> i64 {
        let result = env::var("SUBSCRIPTION_LIMIT").unwrap_or_else(|_| "20".to_string());

        result.parse().unwrap()
    }

    pub fn command() -> &'static str {
        COMMAND
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
                self.subscribe(&connection, message, argument)
            }
            Err(error_message) => error_message,
        }
    }

    fn command(&self) -> &str {
        Self::command()
    }
}
