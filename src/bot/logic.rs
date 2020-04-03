use crate::db::feeds;
use crate::db::telegram;
use crate::db::telegram::{NewTelegramChat, NewTelegramSubscription};
use crate::models::telegram_subscription::TelegramSubscription;
use diesel::{Connection, PgConnection};
use url::Url;

#[derive(Debug, PartialEq)]
pub enum SubscriptionError {
    DbError(diesel::result::Error),
    InvalidRssUrl,
    SubscriptionAlreadyExists,
    SubscriptionCountLimit,
}

impl From<diesel::result::Error> for SubscriptionError {
    fn from(error: diesel::result::Error) -> Self {
        SubscriptionError::DbError(error)
    }
}

pub fn create_subscription(
    db_connection: &PgConnection,
    new_chat: NewTelegramChat,
    rss_url: &str,
) -> Result<TelegramSubscription, SubscriptionError> {
    validate_rss_url(rss_url)?;

    db_connection.transaction::<TelegramSubscription, SubscriptionError, _>(|| {
        let chat = telegram::create_chat(db_connection, new_chat).unwrap();
        let feed = feeds::create(db_connection, rss_url.to_string()).unwrap();

        let new_telegram_subscription = NewTelegramSubscription {
            chat_id: chat.id,
            feed_id: feed.id,
        };

        check_if_subscription_exists(db_connection, new_telegram_subscription)?;
        check_number_of_subscriptions(db_connection, chat.id)?;

        let subscription =
            telegram::create_subscription(db_connection, new_telegram_subscription).unwrap();

        Ok(subscription)
    })
}

fn validate_rss_url(rss_url: &str) -> Result<(), SubscriptionError> {
    match Url::parse(rss_url) {
        Ok(_) => Ok(()),
        _ => Err(SubscriptionError::InvalidRssUrl),
    }
}

fn check_if_subscription_exists(
    connection: &PgConnection,
    subscription: NewTelegramSubscription,
) -> Result<(), SubscriptionError> {
    match telegram::find_subscription(connection, subscription) {
        None => Ok(()),
        Some(_) => Err(SubscriptionError::SubscriptionAlreadyExists),
    }
}

fn check_number_of_subscriptions(
    connection: &PgConnection,
    chat_id: i64,
) -> Result<(), SubscriptionError> {
    match telegram::count_subscriptions_for_chat(connection, chat_id) {
        0 | 1 | 2 => Ok(()),
        _ => Err(SubscriptionError::SubscriptionCountLimit),
    }
}

#[cfg(test)]
mod tests {
    use crate::db;
    use crate::db::feeds;
    use crate::db::telegram;
    use crate::db::telegram::NewTelegramChat;
    use diesel::connection::Connection;

    #[test]
    fn create_subscription_creates_new_subscription() {
        let db_connection = db::establish_connection();
        let new_chat = NewTelegramChat {
            id: 42,
            kind: "private".to_string(),
            title: None,
            username: Some("Username".to_string()),
            first_name: Some("First".to_string()),
            last_name: Some("Last".to_string()),
        };

        db_connection.test_transaction::<(), super::SubscriptionError, _>(|| {
            let subscription =
                super::create_subscription(&db_connection, new_chat, "https:/google.com").unwrap();

            assert!(feeds::find(&db_connection, subscription.feed_id).is_some());
            assert!(telegram::find_chat(&db_connection, subscription.chat_id).is_some());

            Ok(())
        });
    }

    #[test]
    fn create_subscription_fails_to_create_chat_when_rss_url_is_invalid() {
        let db_connection = db::establish_connection();
        let new_chat = NewTelegramChat {
            id: 42,
            kind: "private".to_string(),
            title: None,
            username: Some("Username".to_string()),
            first_name: Some("First".to_string()),
            last_name: Some("Last".to_string()),
        };

        db_connection.test_transaction::<(), super::SubscriptionError, _>(|| {
            let result = super::create_subscription(&db_connection, new_chat, "11");
            assert_eq!(result.err(), Some(super::SubscriptionError::InvalidRssUrl));

            Ok(())
        });
    }

    #[test]
    fn create_subscription_fails_to_create_a_subscription_if_it_already_exists() {
        let db_connection = db::establish_connection();
        let new_chat = NewTelegramChat {
            id: 42,
            kind: "private".to_string(),
            title: None,
            username: Some("Username".to_string()),
            first_name: Some("First".to_string()),
            last_name: Some("Last".to_string()),
        };

        db_connection.test_transaction::<(), super::SubscriptionError, _>(|| {
            let subscription =
                super::create_subscription(&db_connection, new_chat.clone(), "https:/google.com")
                    .unwrap();

            assert!(feeds::find(&db_connection, subscription.feed_id).is_some());
            assert!(telegram::find_chat(&db_connection, subscription.chat_id).is_some());

            let result = super::create_subscription(&db_connection, new_chat, "https:/google.com");
            assert_eq!(
                result.err(),
                Some(super::SubscriptionError::SubscriptionAlreadyExists)
            );

            Ok(())
        });
    }

    #[test]
    fn create_subscription_fails_to_create_a_subscription_if_it_already_has_3_suscriptions() {
        let db_connection = db::establish_connection();
        let new_chat = NewTelegramChat {
            id: 42,
            kind: "private".to_string(),
            title: None,
            username: Some("Username".to_string()),
            first_name: Some("First".to_string()),
            last_name: Some("Last".to_string()),
        };

        db_connection.test_transaction::<(), super::SubscriptionError, _>(|| {
            super::create_subscription(&db_connection, new_chat.clone(), "https:/google.com")
                .unwrap();
            super::create_subscription(&db_connection, new_chat.clone(), "https:/google1.com")
                .unwrap();
            super::create_subscription(&db_connection, new_chat.clone(), "https:/google2.com")
                .unwrap();

            let result = super::create_subscription(&db_connection, new_chat, "https:/google3.com");

            assert_eq!(
                result.err(),
                Some(super::SubscriptionError::SubscriptionCountLimit)
            );

            Ok(())
        });
    }
}
