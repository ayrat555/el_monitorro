use super::Command;
use super::Message;
use crate::db::telegram;
use crate::deliver::render_template_example;
use diesel::PgConnection;
use typed_builder::TypedBuilder;

use frankenstein::InlineKeyboardButton;
use frankenstein::InlineKeyboardMarkup;
use frankenstein::ReplyMarkup;
use frankenstein::SendMessageParams;

static COMMAND: &str = "/set_global_template";

#[derive(Builder)]
pub struct SetGlobalTemplate {
    message: Message,
    args: String,
}

impl SetGlobalTemplate {
    pub fn run(&self) {
        self.execute(&self.message);
    }

    fn set_global_template(&self, db_connection: &mut PgConnection) -> String {
        if self.args.is_empty() {
            return "Template can not be empty".to_string();
        }

        let chat = match telegram::find_chat(db_connection, self.message.chat.id) {
            Some(chat) => chat,
            None => return "You don't have any subcriptions".to_string(),
        };

        let example = match render_template_example(&self.args) {
            Ok(example) => format!("Your messages will look like:\n\n{}", example),
            Err(_) => return "The template is invalid".to_string(),
        };

        if self
            .api()
            .send_text_message(self.message.chat.id, example)
            .is_err()
        {
            return "The template is invalid".to_string();
        }

        match telegram::set_global_template(db_connection, &chat, Some(self.args.clone())) {
            Ok(_) => "The global template was updated".to_string(),
            Err(_) => "Failed to update the template".to_string(),
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for SetGlobalTemplate {
    fn response(&self) -> String {
        match self.fetch_db_connection() {
            Ok(mut connection) => self.set_global_template(&mut connection),
            Err(error_message) => error_message,
        }
    }
}

pub fn set_global_template_keyboard(message: &Message) -> SendMessageParams {
    let chat_id: i64 = message.chat.id;
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let mut row: Vec<InlineKeyboardButton> = Vec::new();
    let mut row2: Vec<InlineKeyboardButton> = Vec::new();
    let mut row3: Vec<InlineKeyboardButton> = Vec::new();

    let create_link_item_description = InlineKeyboardButton::builder()
        .text("Make bot item descriptions as the link to the feed page")
        .switch_inline_query_current_chat(
            "/set_global_template {{create_link bot_item_description bot_item_link }}",
        )
        .build();
    let create_link_item_name = InlineKeyboardButton::builder()
        .text("Make bot item name as the link to the feed page")
        .switch_inline_query_current_chat(
            "/set_global_template {{create_link bot_item_name bot_item_link }}",
        )
        .build();
    let create_link_custom_name = InlineKeyboardButton::builder()
        .text("Make custom name as the link to the feed page")
        .switch_inline_query_current_chat(
            "/set_global_template {{create_link \"custom_name\" bot_item_link }}",
        )
        .build();

    row.push(create_link_item_description);
    row2.push(create_link_item_name);
    row3.push(create_link_custom_name);

    keyboard.push(row);
    keyboard.push(row2);
    keyboard.push(row3);

    let inline_keyboard = InlineKeyboardMarkup::builder()
        .inline_keyboard(keyboard)
        .build();

    SendMessageParams::builder()
        .chat_id(chat_id)
        .text("Use this options to set your template")
        .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
        .build()
}
