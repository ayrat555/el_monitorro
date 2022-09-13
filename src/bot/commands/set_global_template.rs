use super::Command;
use super::Message;
use crate::bot::telegram_client::Api;
use crate::db::telegram;
use crate::deliver::render_template_example;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;

use frankenstein::InlineKeyboardButton;
use frankenstein::InlineKeyboardMarkup;
use frankenstein::ReplyMarkup;
use frankenstein::SendMessageParams;

static COMMAND: &str = "/set_global_template";

pub struct SetGlobalTemplate {}

impl SetGlobalTemplate {
    pub fn execute(db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        Self {}.execute(db_pool, api, message);
    }

    fn set_global_template(
        &self,
        api: &Api,
        db_connection: &mut PgConnection,
        message: &Message,
        template: String,
    ) -> String {
        if template.is_empty() {
            "".to_string();
        }

        let chat = match telegram::find_chat(db_connection, message.chat.id) {
            Some(chat) => chat,
            None => return "You don't have any subcriptions".to_string(),
        };

        let example = match render_template_example(&template) {
            Ok(example) => format!("Your messages will look like:\n\n{}", example),
            Err(_) => return "The template is invalid".to_string(),
        };

        if api.send_text_message(message.chat.id, example).is_err() {
            return "The template is invalid".to_string();
        }

        match telegram::set_global_template(db_connection, &chat, Some(template)) {
            Ok(_) => "The global template was updated".to_string(),
            Err(_) => "Failed to update the template".to_string(),
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for SetGlobalTemplate {
    fn response(
        &self,
        db_pool: Pool<ConnectionManager<PgConnection>>,
        message: &Message,
        api: &Api,
    ) -> String {
        match self.fetch_db_connection(db_pool) {
            Ok(mut connection) => {
                let text = message.text.as_ref().unwrap();
                let argument = self.parse_argument(text);
                self.set_global_template(api, &mut connection, message, argument)
            }
            Err(error_message) => error_message,
        }
    }

    fn command(&self) -> &str {
        Self::command()
    }
}
pub fn set_global_template_keyboard(message: &Message) -> SendMessageParams {
    let chat_id: i64 = message.chat.id;
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let mut row: Vec<InlineKeyboardButton> = Vec::new();
    let mut row2: Vec<InlineKeyboardButton> = Vec::new();
    let mut row3: Vec<InlineKeyboardButton> = Vec::new();
    let mut row4: Vec<InlineKeyboardButton> = Vec::new();

    let substring = InlineKeyboardButton::builder()
        .text("Limit number of characters of feed message")
        .switch_inline_query_current_chat("/set_global_template substring")
        .build();
    let bold = InlineKeyboardButton::builder()
        .text("Set your feed message bold ")
        .switch_inline_query_current_chat("/set_global_template bold")
        .build();
    let italic = InlineKeyboardButton::builder()
        .text("Set your feed message italic")
        .switch_inline_query_current_chat("/set_global_template italic")
        .build();
    let create_link = InlineKeyboardButton::builder()
        .text("Create link to feed site")
        .switch_inline_query_current_chat("/set_global_template create_link")
        .build();

    row.push(substring);
    row2.push(bold);
    row3.push(italic);
    row4.push(create_link);

    keyboard.push(row);
    keyboard.push(row2);
    keyboard.push(row3);
    keyboard.push(row4);

    let inline_keyboard = InlineKeyboardMarkup::builder()
        .inline_keyboard(keyboard)
        .build();

    SendMessageParams::builder()
        .chat_id(chat_id)
        .text("Use this options to set your template")
        .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
        .build()
}

pub fn set_global_template_substring_keyboard(message: &Message) -> SendMessageParams {
    let chat_id: i64 = message.chat.id;
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let mut row: Vec<InlineKeyboardButton> = Vec::new();
    let mut row2: Vec<InlineKeyboardButton> = Vec::new();

    let substring_bot_description = InlineKeyboardButton::builder()
        .text("Limit bot item description characters")
        .switch_inline_query_current_chat(
            "/set_global_template {{substring bot_item_description 100 }}",
        )
        .build();
    let substring_bot_name = InlineKeyboardButton::builder()
        .text("Limit bot item name characters")
        .switch_inline_query_current_chat("/set_global_template {{substring bot_item_name 100 }}")
        .build();

    row.push(substring_bot_description);
    row2.push(substring_bot_name);

    keyboard.push(row);
    keyboard.push(row2);

    let inline_keyboard = InlineKeyboardMarkup::builder()
        .inline_keyboard(keyboard)
        .build();

    SendMessageParams::builder()
        .chat_id(chat_id)
        .text("Use this options to set your template")
        .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
        .build()
}

pub fn set_global_template_bold_keyboard(message: &Message) -> SendMessageParams {
    let chat_id: i64 = message.chat.id;
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let mut row: Vec<InlineKeyboardButton> = Vec::new();
    let mut row2: Vec<InlineKeyboardButton> = Vec::new();
    let mut row3: Vec<InlineKeyboardButton> = Vec::new();

    let bold_bot_description = InlineKeyboardButton::builder()
        .text("Make bot item description bold")
        .callback_data("bold")
        .build();
    let bold_bot_item_name = InlineKeyboardButton::builder()
        .text("Make bot item name bold")
        .switch_inline_query_current_chat("/set_global_template {{bold bot_item_name }}")
        .build();
    let back = InlineKeyboardButton::builder()
        .text("Back to main menu")
        .callback_data("back to menu")
        .build();
    row.push(bold_bot_description);
    row2.push(bold_bot_item_name);
    row3.push(back);
    keyboard.push(row);
    keyboard.push(row2);
    keyboard.push(row3);

    let inline_keyboard = InlineKeyboardMarkup::builder()
        .inline_keyboard(keyboard)
        .build();
    //  CallbackQuery::builder()
    SendMessageParams::builder()
        .chat_id(chat_id)
        .text("Use this options to set your template")
        .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
        .build()
}

pub fn set_global_template_italic_keyboard(message: &Message) -> SendMessageParams {
    let chat_id: i64 = message.chat.id;
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let mut row: Vec<InlineKeyboardButton> = Vec::new();
    let mut row2: Vec<InlineKeyboardButton> = Vec::new();

    let italic_bot_item_description = InlineKeyboardButton::builder()
        .text("Make bot item description italic")
        .switch_inline_query_current_chat("/set_global_template {{italic bot_item_description }}")
        .build();
    let italic_bot_item_name = InlineKeyboardButton::builder()
        .text("Make bot item name italic")
        .switch_inline_query_current_chat("/set_global_template {{italic bot_item_name }}")
        .build();

    row.push(italic_bot_item_description);
    row2.push(italic_bot_item_name);

    keyboard.push(row);
    keyboard.push(row2);

    let inline_keyboard = InlineKeyboardMarkup::builder()
        .inline_keyboard(keyboard)
        .build();

    SendMessageParams::builder()
        .chat_id(chat_id)
        .text("Use this options to set your template")
        .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
        .build()
}

pub fn set_global_template_create_link_keyboard(message: &Message) -> SendMessageParams {
    let chat_id: i64 = message.chat.id;
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let mut row: Vec<InlineKeyboardButton> = Vec::new();
    let mut row2: Vec<InlineKeyboardButton> = Vec::new();
    let mut row3: Vec<InlineKeyboardButton> = Vec::new();

    let create_link_item_description = InlineKeyboardButton::builder()
        .text("Make bot item descriptions as the link")
        .switch_inline_query_current_chat(
            "/set_global_template {{create_link bot_item_description bot_item_link }}",
        )
        .build();
    let create_link_item_name = InlineKeyboardButton::builder()
        .text("Make bot item name as the link")
        .switch_inline_query_current_chat(
            "/set_global_template {{create_link bot_item_name bot_item_link }}",
        )
        .build();
    let create_link_custom_name = InlineKeyboardButton::builder()
        .text("Make custom name as the link")
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
