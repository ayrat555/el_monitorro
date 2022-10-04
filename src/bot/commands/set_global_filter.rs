use super::Command;
use super::Message;
use crate::db::telegram;
use diesel::PgConnection;
<<<<<<< HEAD
use frankenstein::InlineKeyboardButton;
use frankenstein::InlineKeyboardMarkup;
use frankenstein::ReplyMarkup;
use frankenstein::SendMessageParams;
=======
use typed_builder::TypedBuilder;
>>>>>>> master

static COMMAND: &str = "/set_global_filter";

#[derive(TypedBuilder)]
pub struct SetGlobalFilter {
    message: Message,
    args: String,
}

impl SetGlobalFilter {
    pub fn run(&self) {
        self.execute(&self.message);
    }

    fn set_global_template(&self, db_connection: &mut PgConnection) -> String {
        let chat = match telegram::find_chat(db_connection, self.message.chat.id) {
            Some(chat) => chat,
            None => return "You don't have any subcriptions".to_string(),
        };

        if self.args.is_empty() {
            return "Filter can not be empty".to_string();
        }

        let filter_words = match self.parse_filter(&self.args) {
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
    fn response(&self) -> String {
        match self.fetch_db_connection() {
            Ok(mut connection) => self.set_global_template(&mut connection),
            Err(error_message) => error_message,
        }
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
