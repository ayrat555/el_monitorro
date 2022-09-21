use super::Command;
use super::Message;
use crate::bot::telegram_client::Api;
use crate::db::telegram;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;

static COMMAND: &str = "/list_subscriptions";

pub struct ListSubscriptions {}

impl ListSubscriptions {
    pub fn execute(db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        Self {}.execute(db_pool, api, message);
    }

    fn list_subscriptions(&self, db_connection: &mut PgConnection, message: &Message) -> String {
        match telegram::find_feeds_by_chat_id(db_connection, message.chat.id) {
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
    fn response(
        &self,
        db_pool: Pool<ConnectionManager<PgConnection>>,
        message: &Message,
        _api: &Api,
    ) -> String {
        match self.fetch_db_connection(db_pool) {
            Ok(mut connection) => self.list_subscriptions(&mut connection, message),
            Err(error_message) => error_message,
        }
    }

    fn command(&self) -> &str {
        Self::command()
    }
}
// pub fn set_list_subcriptions_menu_keyboard(
//     message: Message,
//     feed_id: String,
//     _feed_url: String,
// ) -> SendMessageParams {
//     // let text = message.text.unwrap();

//     let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

//     let mut row1: Vec<InlineKeyboardButton> = Vec::new();
//     let mut row2: Vec<InlineKeyboardButton> = Vec::new();
//     let mut row3: Vec<InlineKeyboardButton> = Vec::new();
//     let mut row4: Vec<InlineKeyboardButton> = Vec::new();
//     let mut row5: Vec<InlineKeyboardButton> = Vec::new();
//     // let mut row6: Vec<InlineKeyboardButton> = Vec::new();
//     let mut row7: Vec<InlineKeyboardButton> = Vec::new();

//     let unsubscribe = InlineKeyboardButton::builder()
//         .text("Unsubscribe")
//         .callback_data(format!("/unsubscribe {}", feed_id))
//         .build();
//     let remove_filter = InlineKeyboardButton::builder()
//         .text("Remove filter ")
//         .callback_data(format!("/remove_filter {}", feed_id))
//         .build();
//     let get_filter = InlineKeyboardButton::builder()
//         .text("Get filter ")
//         .callback_data(format!("/get_filter {}", feed_id))
//         .build();
//     let set_template = InlineKeyboardButton::builder()
//         .text("Set template")
//         .callback_data(format!("set_template {}", feed_id))
//         .build();
//     let remove_template = InlineKeyboardButton::builder()
//         .text("Remove template")
//         .callback_data(format!("/remove_template {}", feed_id))
//         .build();
//     let get_template = InlineKeyboardButton::builder()
//         .text("Get template")
//         .callback_data(format!("/get_template {}", feed_id))
//         .build();
//     let set_default_template = InlineKeyboardButton::builder()
//         .text("Set default template")
//         .callback_data(format!("set_default_template {}", feed_id))
//         .build();
//     let _disable_enable_preview = InlineKeyboardButton::builder()
//         .text("Disable/Enable Preview")
//         .callback_data(format!("disable_enable {}", feed_id))
//         .build();
//     let back_to_menu = InlineKeyboardButton::builder()
//         .text("Back to menu 🔙 ")
//         .callback_data("/list_subscriptions")
//         .build();
//     row1.push(set_template);
//     row2.push(set_default_template);
//     row3.push(get_filter);
//     row3.push(remove_filter);
//     row4.push(get_template);
//     row4.push(remove_template);
//     row5.push(unsubscribe);
//     // row6.push(disable_enable_preview);
//     row7.push(back_to_menu);

//     keyboard.push(row1);
//     keyboard.push(row2);
//     keyboard.push(row3);
//     keyboard.push(row4);
//     keyboard.push(row5);
//     // keyboard.push(row6);
//     keyboard.push(row7);

//     let inline_keyboard = InlineKeyboardMarkup::builder()
//         .inline_keyboard(keyboard)
//         .build();
//     SendMessageParams::builder()
//         .chat_id(message.chat.id)
//         .text("select your option")
//         .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
//         .build()
// }
// pub fn select_feed_url_keyboard_list_subscriptions(
//     message: Message,
//     _feeds: std::str::Split<'_, &str>,
//     feed_ids: std::str::Split<'_, &str>,
//     db_pool: Pool<ConnectionManager<PgConnection>>,
// ) -> SendMessageParams {
//     let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

//     for feed in feed_ids.clone() {
//         let feed_id: i64 = feed.parse().unwrap();
//         // println!("feed id of select feed url keyboard {}",);
//         let mut row: Vec<InlineKeyboardButton> = Vec::new();
//         let name = format!("{} ", get_feed_url_by_id(db_pool.clone(), feed_id));
//         let unsubscribe_inlinekeyboard = InlineKeyboardButton::builder()
//             .text(name.clone())
//             .callback_data(format!("list_subscriptions {}", feed)) //used letter s to identify the callback ,callback data support no of characters
//             .build();

//         row.push(unsubscribe_inlinekeyboard);
//         keyboard.push(row);
//     }

//     let inline_keyboard = InlineKeyboardMarkup::builder()
//         .inline_keyboard(keyboard)
//         .build();

//     SendMessageParams::builder()
//         .chat_id(message.chat.id)
//         .text("Select feed url to be modify")
//         .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
//         .build()
// }

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
                let feed = feeds::create(connection, link.to_string(), "rss".to_string()).unwrap();

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

            let result = ListSubscriptions {}.list_subscriptions(connection, &message);

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

            let result = ListSubscriptions {}.list_subscriptions(connection, &message);

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

            let result = ListSubscriptions {}.list_subscriptions(connection, &message);

            assert_eq!("You don't have any subscriptions", result);

            Ok(())
        });
    }
}
