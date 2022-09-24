use super::Command;
use super::Message;
use crate::bot::telegram_client::Api;
use crate::db::feeds;
use crate::db::telegram;
use crate::db::telegram::NewTelegramSubscription;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;

static COMMAND: &str = "/unsubscribe";
static CALLBACK: &str = "unsubscribe";
pub struct Unsubscribe {}

enum DeleteSubscriptionError {
    FeedNotFound,
    ChatNotFound,
    SubscriptionNotFound,
    DbError,
}

impl Unsubscribe {
    pub fn execute(db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        Self {}.execute(db_pool, api, message);
    }

    fn unsubscribe(
        &self,
        db_connection: &mut PgConnection,
        message: &Message,
        url: String,
    ) -> String {
        match self.delete_subscription(db_connection, message, url.clone()) {
            Ok(_) => format!("Successfully unsubscribed from {}", url),
            Err(DeleteSubscriptionError::DbError) => format!("Failed to unsubscribe from {}", url),
            _ => "The subscription does not exist".to_string(),
        }
    }

    fn delete_subscription(
        &self,
        db_connection: &mut PgConnection,
        message: &Message,
        link: String,
    ) -> Result<(), DeleteSubscriptionError> {
        let feed = match feeds::find_by_link(db_connection, link) {
            Some(feed) => feed,
            None => return Err(DeleteSubscriptionError::FeedNotFound),
        };

        let chat = match telegram::find_chat(db_connection, message.chat.id) {
            Some(chat) => chat,
            None => return Err(DeleteSubscriptionError::ChatNotFound),
        };

        let telegram_subscription = NewTelegramSubscription {
            chat_id: chat.id,
            feed_id: feed.id,
        };

        let _subscription = match telegram::find_subscription(db_connection, telegram_subscription)
        {
            Some(subscription) => subscription,
            None => return Err(DeleteSubscriptionError::SubscriptionNotFound),
        };

        match telegram::remove_subscription(db_connection, telegram_subscription) {
            Ok(_) => Ok(()),
            _ => Err(DeleteSubscriptionError::DbError),
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
    pub fn callback() -> &'static str {
        CALLBACK
    }
}

impl Command for Unsubscribe {
    fn response(
        &self,
        db_pool: Pool<ConnectionManager<PgConnection>>,
        message: &Message,
        _api: &Api,
    ) -> String {
        match self.fetch_db_connection(db_pool) {
            Ok(mut connection) => {
                let text = message.text.as_ref().unwrap();
                let argument = self.parse_argument(text);
                self.unsubscribe(&mut connection, message, argument)
            }
            Err(error_message) => error_message,
        }
    }

    fn command(&self) -> &str {
        Self::command()
    }
}

#[cfg(test)]
mod unsubscribe_tests {
    use super::Unsubscribe;
    use crate::db;
    use crate::db::feeds;
    use crate::db::telegram;
    use crate::db::telegram::NewTelegramChat;
    use crate::db::telegram::NewTelegramSubscription;
    use diesel::connection::Connection;
    use frankenstein::Chat;
    use frankenstein::ChatType;
    use frankenstein::Message;

    #[test]
    fn removes_subscription() {
        let mut connection = db::establish_test_connection();
        let link = "Link88".to_string();

        connection.test_transaction::<(), (), _>(|connection| {
            let new_chat = NewTelegramChat {
                id: 42,
                kind: "private".to_string(),
                username: Some("Username".to_string()),
                first_name: Some("First".to_string()),
                last_name: Some("Last".to_string()),
                title: None,
            };
            let chat = telegram::create_chat(connection, new_chat).unwrap();
            let feed = feeds::create(connection, link.clone(), "rss".to_string()).unwrap();

            let new_subscription = NewTelegramSubscription {
                feed_id: feed.id,
                chat_id: chat.id,
            };

            telegram::create_subscription(connection, new_subscription).unwrap();

            let result = telegram::fetch_chats_with_subscriptions(connection, 1, 1).unwrap();

            assert_eq!(result.len(), 1);
            assert_eq!(result[0], chat.id);

            let chat = Chat::builder().id(42).type_field(ChatType::Private).build();
            let message = Message::builder()
                .message_id(1)
                .date(1_u64)
                .chat(chat)
                .build();

            let result = Unsubscribe {}.unsubscribe(connection, &message, link.clone());

            assert_eq!(format!("Successfully unsubscribed from {}", link), result);

            let result = telegram::fetch_chats_with_subscriptions(connection, 1, 1).unwrap();

            assert_eq!(result.len(), 0);

            Ok(())
        });
    }

    #[test]
    fn returns_error_if_subscription_does_not_exist() {
        let mut connection = db::establish_test_connection();
        let link = "Link88".to_string();

        connection.test_transaction::<(), (), _>(|connection| {
            let chat = Chat::builder().id(42).type_field(ChatType::Private).build();
            let message = Message::builder()
                .message_id(1)
                .date(1_u64)
                .chat(chat)
                .build();

            let result = Unsubscribe {}.unsubscribe(connection, &message, link.clone());

            assert_eq!("The subscription does not exist", result);

            let result = telegram::fetch_chats_with_subscriptions(connection, 1, 1).unwrap();

            assert_eq!(result.len(), 0);

            Ok(())
        });
    }
}
