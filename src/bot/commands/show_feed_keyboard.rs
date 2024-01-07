use super::Close;
use super::Command;
use super::GetFilter;
use super::GetTemplate;
use super::ListSubscriptionsKeyboard;
use super::RemoveFilter;
use super::RemoveTemplate;
use super::Response;
use super::SetFilter;
use super::SetTemplate;
use super::Unsubscribe;
use crate::db::feeds;
use diesel::PgConnection;
use frankenstein::ChatType;
use frankenstein::InlineKeyboardButton;
use frankenstein::InlineKeyboardMarkup;
use frankenstein::Message;
use frankenstein::ReplyMarkup;
use frankenstein::SendMessageParams;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/feed_keyboard";

#[derive(TypedBuilder)]
pub struct ShowFeedKeyboard {
    message: Message,
    feed_url_or_external_id: String,
}

impl ShowFeedKeyboard {
    pub fn run(&self) {
        self.execute(
            &self.message,
            &format!("{} {}", Self::command(), self.feed_url_or_external_id),
        );
    }

    pub fn command() -> &'static str {
        COMMAND
    }

    fn feed_keyboard(&self, db_connection: &mut PgConnection) -> SendMessageParams {
        let subscription = match self.find_subscription(
            db_connection,
            self.message.chat.id,
            &self.feed_url_or_external_id,
        ) {
            Err(error_message) => {
                return SendMessageParams::builder()
                    .chat_id(self.message.chat.id)
                    .text(error_message)
                    .build();
            }
            Ok(subscription) => subscription,
        };

        let feed = feeds::find(db_connection, subscription.feed_id).unwrap();

        let mut buttons: Vec<Vec<InlineKeyboardButton>> = Vec::new();

        let rows = if let ChatType::Private = self.message.chat.type_field {
            [
                vec![
                    ("Show Filter", GetFilter::command()),
                    ("Set Filter", SetFilter::command()),
                    ("Remove Filter", RemoveFilter::command()),
                ],
                vec![
                    ("Show Template", GetTemplate::command()),
                    ("Set Template", SetTemplate::command()),
                    ("Remove Template", RemoveTemplate::command()),
                ],
                vec![("Unsubscribe", Unsubscribe::command())],
            ]
        } else {
            [
                vec![
                    ("Show Filter", GetFilter::command()),
                    ("Remove Filter", RemoveFilter::command()),
                ],
                vec![
                    ("Show Template", GetTemplate::command()),
                    ("Remove Template", RemoveTemplate::command()),
                ],
                vec![("Unsubscribe", Unsubscribe::command())],
            ]
        };

        for command_row in rows {
            let mut row: Vec<InlineKeyboardButton> = Vec::new();

            for (text, command) in command_row {
                let button = InlineKeyboardButton::builder()
                    .text(text)
                    .callback_data(format!("{} {}", command, subscription.external_id))
                    .build();

                row.push(button);
            }

            buttons.push(row);
        }
        let mut row: Vec<InlineKeyboardButton> = Vec::new();

        let button = InlineKeyboardButton::builder()
            .text("◀ Back")
            .callback_data(ListSubscriptionsKeyboard::command())
            .build();

        row.push(button);

        buttons.push(row);

        buttons.push(Close::button_row());

        let keyboard = InlineKeyboardMarkup::builder()
            .inline_keyboard(buttons)
            .build();

        let mut params = SendMessageParams::builder()
            .chat_id(self.message.chat.id)
            .text(feed.link)
            .reply_markup(ReplyMarkup::InlineKeyboardMarkup(keyboard))
            .build();

        params.message_thread_id = self.message.message_thread_id;

        params
    }
}

impl Command for ShowFeedKeyboard {
    fn response(&self) -> Response {
        match self.fetch_db_connection() {
            Ok(mut connection) => {
                let params = self.feed_keyboard(&mut connection);

                Response::Params(Box::new(params))
            }
            Err(error_message) => Response::Simple(error_message),
        }
    }

    fn send_message(&self, send_message_params: SendMessageParams) {
        self.send_message_and_remove(send_message_params, &self.message);
    }
}
