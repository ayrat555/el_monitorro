use super::Close;
use super::Command;
use super::Response;
use crate::db::telegram;
use diesel::PgConnection;
use frankenstein::InlineKeyboardButton;
use frankenstein::InlineKeyboardMarkup;
use frankenstein::Message;
use frankenstein::ReplyMarkup;
use frankenstein::SendMessageParams;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/list_subscriptions";

#[derive(TypedBuilder)]
pub struct ListSubscriptionsKeyboard {
    message: Message,
}

impl ListSubscriptionsKeyboard {
    pub fn run(&self) {
        self.execute(&self.message, Self::command());
    }

    fn feeds_keyboard_params(&self, db_connection: &mut PgConnection) -> SendMessageParams {
        let feeds = match telegram::find_feeds_by_chat_id(db_connection, self.message.chat.id) {
            Ok(feeds) => feeds,
            Err(_) => {
                return SendMessageParams::builder()
                    .chat_id(self.message.chat.id)
                    .text("Failed to get your subscriptions")
                    .build()
            }
        };

        let message = if feeds.is_empty() {
            "You don't have any subscriptions".to_string()
        } else {
            "Select a feed:".to_string()
        };

        let mut buttons: Vec<Vec<InlineKeyboardButton>> = Vec::new();

        for feed in feeds {
            let mut row: Vec<InlineKeyboardButton> = Vec::new();
            let subscription = self
                .find_subscription_by_chat_id_and_feed_id(
                    db_connection,
                    self.message.chat.id,
                    feed.id,
                )
                .unwrap();

            let feed_button = InlineKeyboardButton::builder()
                .text(feed.link.clone())
                .callback_data(format!("/feed_keyboard {}", subscription.external_id))
                .build();

            row.push(feed_button);
            buttons.push(row);
        }

        buttons.push(Close::button_row());

        let keyboard = InlineKeyboardMarkup::builder()
            .inline_keyboard(buttons)
            .build();

        let mut params = SendMessageParams::builder()
            .chat_id(self.message.chat.id)
            .text(message)
            .reply_markup(ReplyMarkup::InlineKeyboardMarkup(keyboard))
            .build();

        params.message_thread_id = self.message.message_thread_id;

        params
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for ListSubscriptionsKeyboard {
    fn response(&self) -> Response {
        match self.fetch_db_connection() {
            Ok(mut connection) => {
                let params = self.feeds_keyboard_params(&mut connection);

                Response::Params(Box::new(params))
            }
            Err(error_message) => Response::Simple(error_message),
        }
    }

    fn send_message(&self, send_message_params: SendMessageParams) {
        self.send_message_and_remove(send_message_params, &self.message);
    }
}
