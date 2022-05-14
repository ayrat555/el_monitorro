use super::Command;
use super::Message;
use crate::bot::telegram_client::Api;
use crate::config::Config;
use crate::db::feeds;
use crate::db::telegram;
use crate::db::telegram::NewTelegramSubscription;
use crate::models::telegram_subscription::TelegramSubscription;
use crate::sync::reader;
use crate::sync::SyncFeedJob;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::Connection;
use diesel::PgConnection;
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
    pub fn execute(db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        Self {}.execute(db_pool, api, message);
    }

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
            let chat = telegram::create_chat(db_connection, message.chat.clone().into()).unwrap();
            let feed = feeds::create(db_connection, url, feed_type).unwrap();

            let new_telegram_subscription = NewTelegramSubscription {
                chat_id: chat.id,
                feed_id: feed.id,
            };

            self.check_if_subscription_exists(db_connection, new_telegram_subscription)?;
            self.check_number_of_subscriptions(db_connection, chat.id)?;

            let subscription =
                telegram::create_subscription(db_connection, new_telegram_subscription).unwrap();

            SyncFeedJob::new(feed.id).enqueue(&db_connection).unwrap();

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

        if result < Self::sub_limit() {
            Ok(())
        } else {
            Err(SubscriptionError::SubscriptionCountLimit)
        }
    }

    fn sub_limit() -> i64 {
        Config::subscription_limit()
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
                let text = message.text.as_ref().unwrap();
                let argument = self.parse_argument(text);
                self.subscribe(&connection, message, argument)
            }
            Err(error_message) => error_message,
        }
    }

    fn command(&self) -> &str {
        Self::command()
    }
}

#[cfg(test)]
mod subscribe_tests {
    use super::Subscribe;
    use crate::db;
    use crate::db::feeds;
    use crate::db::telegram;
    use diesel::connection::Connection;
    use frankenstein::Chat;
    use frankenstein::ChatType;
    use frankenstein::Message;
    use mockito::mock;

    #[test]
    fn creates_new_subscription() {
        let db_connection = db::establish_test_connection();
        let message = create_message();

        let path = "/feed";
        let response = feed_example();
        let _m = mock("GET", path)
            .with_status(200)
            .with_body(response)
            .create();
        let feed_url = format!("{}{}", mockito::server_url(), path);

        db_connection.test_transaction::<(), (), _>(|| {
            let result = Subscribe {}.subscribe(&db_connection, &message, feed_url.clone());

            assert_eq!(result, format!("Successfully subscribed to {}", feed_url));

            let subscriptions = telegram::fetch_subscriptions(&db_connection, 1, 1000).unwrap();

            assert_eq!(1, subscriptions.len());
            assert_eq!(message.chat.id, subscriptions[0].chat_id);
            assert!(feeds::find_by_link(&db_connection, feed_url).is_some());

            Ok(())
        });
    }

    #[test]
    fn create_subscription_fails_to_create_chat_when_rss_url_is_invalid() {
        let db_connection = db::establish_test_connection();
        let message = create_message();

        db_connection.test_transaction::<(), (), _>(|| {
            let result = Subscribe {}.subscribe(&db_connection, &message, "11".to_string());

            assert_eq!(result, "Invalid url".to_string());

            let subscriptions = telegram::fetch_subscriptions(&db_connection, 1, 1000).unwrap();
            assert_eq!(0, subscriptions.len());

            Ok(())
        });
    }

    #[test]
    fn create_subscription_fails_to_create_chat_when_rss_url_is_not_rss() {
        let db_connection = db::establish_test_connection();
        let message = create_message();

        let path = "/not_feed";
        let _m = mock("GET", path)
            .with_status(200)
            .with_body("hello")
            .create();
        let feed_url = format!("{}{}", mockito::server_url(), path);

        db_connection.test_transaction::<(), (), _>(|| {
            let result = Subscribe {}.subscribe(&db_connection, &message, feed_url);

            assert_eq!(result, "Url is not a feed".to_string());

            let subscriptions = telegram::fetch_subscriptions(&db_connection, 1, 1000).unwrap();
            assert_eq!(0, subscriptions.len());

            Ok(())
        });
    }

    #[test]
    fn create_subscription_fails_to_create_a_subscription_if_it_already_exists() {
        let db_connection = db::establish_test_connection();
        let message = create_message();

        let path = "/feed";
        let response = feed_example();
        let _m = mock("GET", path)
            .with_status(200)
            .with_body(response)
            .create();
        let feed_url = format!("{}{}", mockito::server_url(), path);

        db_connection.test_transaction::<(), super::SubscriptionError, _>(|| {
            Subscribe {}.subscribe(&db_connection, &message, feed_url.clone());

            let result = Subscribe {}.subscribe(&db_connection, &message, feed_url);

            assert_eq!(result, "The subscription already exists".to_string());

            let subscriptions = telegram::fetch_subscriptions(&db_connection, 1, 1000).unwrap();
            assert_eq!(1, subscriptions.len());

            Ok(())
        });
    }

    #[test]
    fn create_subscription_fails_to_create_a_subscription_if_it_already_has_20_suscriptions() {
        let db_connection = db::establish_test_connection();
        let message = create_message();

        let response = feed_example();

        let path1 = "/feed1";
        let _m1 = mock("GET", path1)
            .with_status(200)
            .with_body(&response)
            .create();

        let path2 = "/feed2";
        let _m2 = mock("GET", path2)
            .with_status(200)
            .with_body(&response)
            .create();

        let path3 = "/feed3";
        let _m3 = mock("GET", path3)
            .with_status(200)
            .with_body(&response)
            .create();

        let feed_url1 = format!("{}{}", mockito::server_url(), path1);
        let feed_url2 = format!("{}{}", mockito::server_url(), path2);
        let feed_url3 = format!("{}{}", mockito::server_url(), path3);

        std::env::set_var("SUBSCRIPTION_LIMIT", "2");

        db_connection.test_transaction::<(), super::SubscriptionError, _>(|| {
            for rss_url in [feed_url1, feed_url2] {
                let result = Subscribe {}.subscribe(&db_connection, &message, rss_url.clone());

                assert_eq!(format!("Successfully subscribed to {}", rss_url), result);
            }

            let result = Subscribe {}.subscribe(&db_connection, &message, feed_url3.clone());

            assert_eq!("You exceeded the number of subscriptions", result);

            Ok(())
        });
    }

    fn create_message() -> Message {
        let chat = Chat::builder().id(1).type_field(ChatType::Private).build();
        Message::builder()
            .message_id(1)
            .date(1_u64)
            .chat(chat)
            .build()
    }

    fn feed_example() -> String {
        std::fs::read_to_string("./tests/support/rss_feed_example.xml").unwrap()
    }
}
