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

static COMMAND: &str = "/set_global_filter";

pub struct SetGlobalFilter {}

impl SetGlobalFilter {
    pub fn execute(db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        Self {}.execute(db_pool, api, message);
    }

    fn set_global_template(
        &self,
        db_connection: &mut PgConnection,
        message: &Message,
        filter: String,
    ) -> String {
        let chat = match telegram::find_chat(db_connection, message.chat.id) {
            Some(chat) => chat,
            None => return "You don't have any subcriptions".to_string(),
        };

        if filter.is_empty() {
            return "Filter can not be empty".to_string();
        }

        let filter_words = match self.parse_filter(&filter) {
            Err(message) => return message,
            Ok(words) => words,
        };

        match telegram::set_global_filter(db_connection, &chat, Some(filter_words.clone())) {
            Ok(_) => format!(
                "The global filter was updated:\n\n{}",
                filter_words.join(", ")
            ),
            Err(_) => "Failed to update the filter".to_string(),
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for SetGlobalFilter {
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
                self.set_global_template(&mut connection, message, argument)
            }
            Err(error_message) => error_message,
        }
    }

    fn command(&self) -> &str {
        Self::command()
    }
}

pub fn set_global_filter_keyboard(message: Message, _feed_url: String) -> SendMessageParams {
    let _text = message.text.unwrap();
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let mut row: Vec<InlineKeyboardButton> = Vec::new();

    let substring = InlineKeyboardButton::builder()
        .text("Click here")
        .switch_inline_query_current_chat("/set_global_filter  filter_words")
        .build();

    row.push(substring);

    keyboard.push(row);

    let inline_keyboard = InlineKeyboardMarkup::builder()
        .inline_keyboard(keyboard)
        .build();
    SendMessageParams::builder()
        .chat_id(message.chat.id)
        .text("send your filter words")
        .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
        .build()
}
