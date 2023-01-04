use super::Close;
use super::Command;
use super::Message;
use super::Response;
use crate::db::telegram;
use diesel::PgConnection;
use frankenstein::InlineKeyboardButton;
use frankenstein::InlineKeyboardMarkup;
use frankenstein::ReplyMarkup;
use frankenstein::SendMessageParams;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/list_subscriptions";

#[derive(TypedBuilder)]
pub struct ListSubscriptions {
    message: Message,
}

impl ListSubscriptions {
    pub fn run(&self) {
        self.execute(&self.message);
    }

    fn list_subscriptions(&self, db_connection: &mut PgConnection) -> String {
        match telegram::find_feeds_by_chat_id(db_connection, self.message.chat.id) {
            Err(_) => "Couldn't fetch your subscriptions".to_string(),
            Ok(feeds) => {
                if feeds.is_empty() {
                    "You don't have any subscriptions".to_string()
                } else {
                    feeds
                        .into_iter()
                        .map(|feed| feed.link)
                        .collect::<Vec<String>>()
                        .join("\n")
                }
            }
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for ListSubscriptions {
    fn response(&self) -> Response {
        let subscriptions_list = match self.fetch_db_connection() {
            Ok(mut connection) => self.list_subscriptions(&mut connection),
            Err(error_message) => error_message,
        };

        let feed_id = match self.fetch_db_connection() {
            Ok(mut connection) => list_feed_id(&mut connection, &self.message),
            Err(_error_message) => "error fetching data".to_string(),
        };

        let feeds_names = subscriptions_list.split(',').clone();
        let feed_ids = feed_id.split(',').clone();

        let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

        for feed in feeds_names {
            for feed_id in feed_ids.clone() {
                let mut row: Vec<InlineKeyboardButton> = Vec::new();
                let name = if feed == "You don't have any subscriptions"
                    || feed == "error fetching data"
                {
                    feed.to_string()
                } else {
                    format!("{} ", feed)
                };

                let unsubscribe_inlinekeyboard = InlineKeyboardButton::builder()
                    .text(name.clone())
                    .callback_data(format!("list_subscriptions {}", feed_id))
                    .build();

                row.push(unsubscribe_inlinekeyboard);
                keyboard.push(row);
            }
        }
        keyboard.push(Close::button_row());
        let inline_keyboard = InlineKeyboardMarkup::builder()
            .inline_keyboard(keyboard)
            .build();

        let send_message_params = SendMessageParams::builder()
            .chat_id(self.message.chat.id)
            .text("Select a feed url ")
            .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
            .build();

        Response::Params(send_message_params)
    }
}

fn list_feed_id(db_connection: &mut PgConnection, message: &Message) -> String {
    match telegram::find_feeds_by_chat_id(db_connection, message.chat.id) {
        Err(_) => "Couldn't fetch your subscriptions".to_string(),
        Ok(feeds) => {
            if feeds.is_empty() {
                "You don't have any subscriptions".to_string()
            } else {
                feeds
                    .into_iter()
                    .map(|feed| feed.id.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            }
        }
    }
}

#[cfg(test)]
mod list_subscriptions_tests {
    use super::ListSubscriptions;
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
    fn fetches_subscriptions() {
        let mut connection = db::establish_test_connection();

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

            for link in ["link1", "link2"] {
                let feed = feeds::create(connection, link, "rss".to_string()).unwrap();

                let new_subscription = NewTelegramSubscription {
                    feed_id: feed.id,
                    chat_id: chat.id,
                };

                telegram::create_subscription(connection, new_subscription).unwrap();
            }

            let chat = Chat::builder().id(42).type_field(ChatType::Private).build();
            let message = Message::builder()
                .message_id(1)
                .date(1_u64)
                .chat(chat)
                .build();

            let result = ListSubscriptions::builder()
                .message(message)
                .build()
                .list_subscriptions(connection);

            assert_eq!("link1\nlink2", result);

            Ok(())
        });
    }

    #[test]
    fn returns_error_if_no_subscriptiops() {
        let mut connection = db::establish_test_connection();

        connection.test_transaction::<(), (), _>(|connection| {
            let new_chat = NewTelegramChat {
                id: 42,
                kind: "private".to_string(),
                username: Some("Username".to_string()),
                first_name: Some("First".to_string()),
                last_name: Some("Last".to_string()),
                title: None,
            };
            telegram::create_chat(connection, new_chat).unwrap();
            let chat = Chat::builder().id(42).type_field(ChatType::Private).build();
            let message = Message::builder()
                .message_id(1)
                .date(1_u64)
                .chat(chat)
                .build();

            let result = ListSubscriptions::builder()
                .message(message)
                .build()
                .list_subscriptions(connection);

            assert_eq!("You don't have any subscriptions", result);

            Ok(())
        });
    }

    #[test]
    fn returns_error_if_chat_does_not_exist() {
        let mut connection = db::establish_test_connection();

        connection.test_transaction::<(), (), _>(|connection| {
            let chat = Chat::builder().id(42).type_field(ChatType::Private).build();
            let message = Message::builder()
                .message_id(1)
                .date(1_u64)
                .chat(chat)
                .build();

            let result = ListSubscriptions::builder()
                .message(message)
                .build()
                .list_subscriptions(connection);

            assert_eq!("You don't have any subscriptions", result);

            Ok(())
        });
    }
}
