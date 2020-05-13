use crate::db::feeds;
use crate::db::telegram;
use crate::db::telegram::{NewTelegramChat, NewTelegramSubscription};
use crate::models::telegram_subscription::TelegramSubscription;
use diesel::{Connection, PgConnection};
use rss::Channel;
use url::Url;

#[derive(Debug, PartialEq)]
pub enum SubscriptionError {
    DbError(diesel::result::Error),
    InvalidRssUrl,
    UrlIsNotRss,
    RssUrlNotProvided,
    SubscriptionAlreadyExists,
    SubscriptionCountLimit,
    TelegramError,
}

pub enum DeleteSubscriptionError {
    FeedNotFound,
    ChatNotFound,
    SubscriptionNotFound,
    DbError,
}

impl From<diesel::result::Error> for SubscriptionError {
    fn from(error: diesel::result::Error) -> Self {
        SubscriptionError::DbError(error)
    }
}

pub fn find_feeds_by_chat_id(db_connection: &PgConnection, chat_id: i64) -> String {
    match telegram::find_feeds_by_chat_id(db_connection, chat_id) {
        Err(_) => "Couldn't fetch your subscriptions".to_string(),
        Ok(feeds) => {
            let response = feeds
                .into_iter()
                .map(|feed| feed.link)
                .collect::<Vec<String>>()
                .join("\n");
            if response == "" {
                "You don't have any subscriptions".to_string()
            } else {
                response
            }
        }
    }
}

pub fn delete_subscription(
    db_connection: &PgConnection,
    chat_id: i64,
    link: String,
) -> Result<(), DeleteSubscriptionError> {
    let feed = match feeds::find_by_link(db_connection, link) {
        Some(feed) => feed,
        None => return Err(DeleteSubscriptionError::FeedNotFound),
    };

    let chat = match telegram::find_chat(db_connection, chat_id) {
        Some(chat) => chat,
        None => return Err(DeleteSubscriptionError::ChatNotFound),
    };

    let telegram_subscription = NewTelegramSubscription {
        chat_id: chat.id,
        feed_id: feed.id,
    };

    let _subscription = match telegram::find_subscription(db_connection, telegram_subscription) {
        Some(subscription) => subscription,
        None => return Err(DeleteSubscriptionError::SubscriptionNotFound),
    };

    match telegram::remove_subscription(db_connection, telegram_subscription) {
        Ok(_) => Ok(()),
        _ => Err(DeleteSubscriptionError::DbError),
    }
}

pub fn create_subscription(
    db_connection: &PgConnection,
    new_chat: NewTelegramChat,
    rss_url: Option<String>,
) -> Result<TelegramSubscription, SubscriptionError> {
    if rss_url.is_none() {
        return Err(SubscriptionError::RssUrlNotProvided);
    }

    let url = rss_url.unwrap();

    validate_rss_url(&url)?;

    db_connection.transaction::<TelegramSubscription, SubscriptionError, _>(|| {
        let chat = telegram::create_chat(db_connection, new_chat).unwrap();
        let feed = feeds::create(db_connection, url).unwrap();

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
        Ok(_) => match Channel::from_url(rss_url) {
            Ok(_) => Ok(()),
            _ => Err(SubscriptionError::UrlIsNotRss),
        },
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
        0 | 1 | 2 | 3 | 4 => Ok(()),
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
            username: Some("Username".to_string()),
            first_name: Some("First".to_string()),
            last_name: Some("Last".to_string()),
        };

        db_connection.test_transaction::<(), super::SubscriptionError, _>(|| {
            let subscription = super::create_subscription(
                &db_connection,
                new_chat,
                Some("http://feeds.reuters.com/reuters/technologyNews".to_string()),
            )
            .unwrap();

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
            username: Some("Username".to_string()),
            first_name: Some("First".to_string()),
            last_name: Some("Last".to_string()),
        };

        db_connection.test_transaction::<(), super::SubscriptionError, _>(|| {
            let result =
                super::create_subscription(&db_connection, new_chat, Some("11".to_string()));
            assert_eq!(result.err(), Some(super::SubscriptionError::InvalidRssUrl));

            Ok(())
        });
    }

    #[test]
    fn create_subscription_fails_to_create_chat_when_rss_url_is_not_rss() {
        let db_connection = db::establish_connection();
        let new_chat = NewTelegramChat {
            id: 42,
            kind: "private".to_string(),
            username: Some("Username".to_string()),
            first_name: Some("First".to_string()),
            last_name: Some("Last".to_string()),
        };

        db_connection.test_transaction::<(), super::SubscriptionError, _>(|| {
            let result = super::create_subscription(
                &db_connection,
                new_chat,
                Some("http://google.com".to_string()),
            );
            assert_eq!(result.err(), Some(super::SubscriptionError::UrlIsNotRss));

            Ok(())
        });
    }

    #[test]
    fn create_subscription_fails_to_create_a_subscription_if_it_already_exists() {
        let db_connection = db::establish_connection();
        let new_chat = NewTelegramChat {
            id: 42,
            kind: "private".to_string(),
            username: Some("Username".to_string()),
            first_name: Some("First".to_string()),
            last_name: Some("Last".to_string()),
        };

        db_connection.test_transaction::<(), super::SubscriptionError, _>(|| {
            let subscription = super::create_subscription(
                &db_connection,
                new_chat.clone(),
                Some("http://feeds.reuters.com/reuters/technologyNews".to_string()),
            )
            .unwrap();

            assert!(feeds::find(&db_connection, subscription.feed_id).is_some());
            assert!(telegram::find_chat(&db_connection, subscription.chat_id).is_some());

            let result = super::create_subscription(
                &db_connection,
                new_chat,
                Some("http://feeds.reuters.com/reuters/technologyNews".to_string()),
            );
            assert_eq!(
                result.err(),
                Some(super::SubscriptionError::SubscriptionAlreadyExists)
            );

            Ok(())
        });
    }

    #[test]
    fn create_subscription_fails_to_create_a_subscription_if_it_already_has_5_suscriptions() {
        let db_connection = db::establish_connection();
        let new_chat = NewTelegramChat {
            id: 42,
            kind: "private".to_string(),
            username: Some("Username".to_string()),
            first_name: Some("First".to_string()),
            last_name: Some("Last".to_string()),
        };

        db_connection.test_transaction::<(), super::SubscriptionError, _>(|| {
            for rss_url in vec![
                "https://rss.nytimes.com/services/xml/rss/nyt/HomePage.xml",
                "https://www.eurekalert.org/rss/technology_engineering.xml",
                "https://www.sciencedaily.com/rss/matter_energy/engineering.xml",
                "https://www.france24.com/fr/france/rss",
                "http://feeds.reuters.com/reuters/technologyNews",
            ] {
                assert!(super::create_subscription(
                    &db_connection,
                    new_chat.clone(),
                    Some(rss_url.to_string()),
                )
                .is_ok());
            }

            let result = super::create_subscription(
                &db_connection,
                new_chat,
                Some("http://www.engadget.com/rss.xml".to_string()),
            );

            assert_eq!(
                result.err(),
                Some(super::SubscriptionError::SubscriptionCountLimit)
            );

            Ok(())
        });
    }

    #[test]
    fn create_subscription_fails_if_url_is_not_provided() {
        let db_connection = db::establish_connection();
        let new_chat = NewTelegramChat {
            id: 42,
            kind: "private".to_string(),
            username: Some("Username".to_string()),
            first_name: Some("First".to_string()),
            last_name: Some("Last".to_string()),
        };

        db_connection.test_transaction::<(), super::SubscriptionError, _>(|| {
            let result = super::create_subscription(&db_connection, new_chat.clone(), None);

            assert_eq!(
                result.err(),
                Some(super::SubscriptionError::RssUrlNotProvided)
            );

            Ok(())
        })
    }
}
