use super::Command;
use super::Message;
use crate::bot::telegram_client::Api;
use crate::db::telegram;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use frankenstein::InlineKeyboardButton;
use frankenstein::InlineKeyboardMarkup;
use frankenstein::ReplyMarkup;
use frankenstein::SendMessageParams;
use typed_builder::TypedBuilder;
static COMMAND: &str = "/remove_template";

#[derive(TypedBuilder)]
pub struct RemoveTemplate {
    message: Message,
    args: String,
}

impl RemoveTemplate {
    pub fn run(&self, db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        self.execute(db_pool, api, message);
    }
    fn remove_template(&self, db_connection: &mut PgConnection) -> String {
        let subscription =
            match self.find_subscription(db_connection, self.message.chat.id, &self.args) {
                Err(message) => return message,
                Ok(subscription) => subscription,
            };

        match telegram::set_template(db_connection, &subscription, None) {
            Ok(_) => "The template was removed".to_string(),
            Err(_) => "Failed to update the template".to_string(),
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for RemoveTemplate {
    fn response(&self) -> String {
        match self.fetch_db_connection() {
            Ok(mut connection) => self.remove_template(&mut connection),
            Err(error_message) => error_message,
        }
    }
}
pub fn remove_template_keyboard(message: Message, feed_url: String) -> SendMessageParams {
    let text = message.text.unwrap();

    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let mut row: Vec<InlineKeyboardButton> = Vec::new();

    let substring = InlineKeyboardButton::builder()
        .text("Remove template")
        .callback_data(format!("/remove_temp {}", feed_url))
        .build();

    row.push(substring);

    keyboard.push(row);

    let inline_keyboard = InlineKeyboardMarkup::builder()
        .inline_keyboard(keyboard)
        .build();
    SendMessageParams::builder()
        .chat_id(message.chat.id)
        .text(text)
        .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
        .build()
}
