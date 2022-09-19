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
        println!(
            "text in response setglobal template {}",
            message.text.as_ref().unwrap()
        );
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

pub fn set_global_template_keyboard(message: Message) -> SendMessageParams {
    let text = message.text.unwrap();
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let mut row: Vec<InlineKeyboardButton> = Vec::new();
    let mut row2: Vec<InlineKeyboardButton> = Vec::new();
    let mut row3: Vec<InlineKeyboardButton> = Vec::new();
    let mut row4: Vec<InlineKeyboardButton> = Vec::new();

    let substring = InlineKeyboardButton::builder()
        .text("Limit number of characters of feed message")
        .callback_data("global_substring")
        .build();
    let bold = InlineKeyboardButton::builder()
        .text("Set your feed message bold ")
        .callback_data("global_bold")
        .build();
    let italic = InlineKeyboardButton::builder()
        .text("Set your feed message italic")
        .callback_data("global_italic")
        .build();
    let create_link = InlineKeyboardButton::builder()
        .text("Create link to feed site")
        .callback_data("global_create_link")
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
        .chat_id(message.chat.id)
        .text(text)
        .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
        .build()
}

pub fn set_global_template_substring_keyboard(message: Message) -> SendMessageParams {
    let text = message.text.unwrap();
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let mut row1: Vec<InlineKeyboardButton> = Vec::new();
    let mut row2: Vec<InlineKeyboardButton> = Vec::new();
    let mut row3: Vec<InlineKeyboardButton> = Vec::new();

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
    let back_to_menu = InlineKeyboardButton::builder()
        .text("Back to menu 🔙 ")
        .callback_data("back to menu")
        .build();

    row1.push(substring_bot_description);
    row2.push(substring_bot_name);
    row3.push(back_to_menu);

    keyboard.push(row1);
    keyboard.push(row2);
    keyboard.push(row3);
    let inline_keyboard = InlineKeyboardMarkup::builder()
        .inline_keyboard(keyboard)
        .build();
    SendMessageParams::builder()
        .chat_id(message.chat.id)
        .text(text)
        .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
        .build()
}

pub fn set_global_template_bold_keyboard(message: Message) -> SendMessageParams {
    let text = message.text.unwrap();
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let mut row1: Vec<InlineKeyboardButton> = Vec::new();
    let mut row2: Vec<InlineKeyboardButton> = Vec::new();
    let mut row3: Vec<InlineKeyboardButton> = Vec::new();

    let bold_bot_description = InlineKeyboardButton::builder()
        .text("Make bot item description 𝐛𝐨𝐥𝐝")
        .callback_data("/set_global_template {{bold bot_item_description }}")
        .build();
    let bold_bot_item_name = InlineKeyboardButton::builder()
        .text("Make bot item name 𝐛𝐨𝐥𝐝")
        .callback_data("/set_global_template {{bold bot_item_name }}")
        .build();
    let back_to_menu = InlineKeyboardButton::builder()
        .text("Back to menu 🔙 ")
        .callback_data("back to menu")
        .build();

    row1.push(bold_bot_description);
    row2.push(bold_bot_item_name);
    row3.push(back_to_menu);

    keyboard.push(row1);
    keyboard.push(row2);
    keyboard.push(row3);

    let inline_keyboard = InlineKeyboardMarkup::builder()
        .inline_keyboard(keyboard)
        .build();

    SendMessageParams::builder()
        .chat_id(message.chat.id)
        .text(text)
        .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
        .build()
}

pub fn set_global_template_italic_keyboard(message: Message) -> SendMessageParams {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let text = message.text.unwrap();
    println!(
        "text inside set global templ italic keyboard ======{:?}",
        text
    );
    let mut row: Vec<InlineKeyboardButton> = Vec::new();
    let mut row2: Vec<InlineKeyboardButton> = Vec::new();
    let mut row3: Vec<InlineKeyboardButton> = Vec::new();

    let italic_bot_item_description = InlineKeyboardButton::builder()
        .text("Make bot item description 𝘪𝘵𝘢𝘭𝘪𝘤")
        .callback_data("/set_global_template {{italic bot_item_description }}")
        .build();
    let italic_bot_item_name = InlineKeyboardButton::builder()
        .text("Make bot item name 𝘪𝘵𝘢𝘭𝘪𝘤")
        .callback_data("/set_global_template {{italic bot_item_name }}")
        .build();
    let back_to_menu = InlineKeyboardButton::builder()
        .text("Back to menu 🔙 ")
        .callback_data("back to menu")
        .build();

    row.push(italic_bot_item_description);
    row2.push(italic_bot_item_name);
    row3.push(back_to_menu);

    keyboard.push(row);
    keyboard.push(row2);
    keyboard.push(row3);

    let inline_keyboard = InlineKeyboardMarkup::builder()
        .inline_keyboard(keyboard)
        .build();
    SendMessageParams::builder()
        .chat_id(message.chat.id)
        .text(text)
        .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
        .build()
}

pub fn set_global_template_create_link_keyboard(message: Message) -> SendMessageParams {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let text = message.text.unwrap();

    let mut row1: Vec<InlineKeyboardButton> = Vec::new();
    let mut row2: Vec<InlineKeyboardButton> = Vec::new();
    let mut row3: Vec<InlineKeyboardButton> = Vec::new();
    let mut row4: Vec<InlineKeyboardButton> = Vec::new();

    let create_link_bot_item_description = InlineKeyboardButton::builder()
        .text("Make bot item description as link")
        .callback_data("/set_global_template create_link_description")
        .build();
    let create_link_bot_item_name = InlineKeyboardButton::builder()
        .text("Make bot item name as link")
        .callback_data("/set_global_template create_link_item_name")
        .build();
    let create_link_custom_name = InlineKeyboardButton::builder()
        .text("Make custom name as link")
        .switch_inline_query_current_chat(
            "/set_global_template {{create_link \"custom name\" bot_item_link}}",
        )
        .build();
    let back_to_menu = InlineKeyboardButton::builder()
        .text("Back to menu 🔙 ")
        .callback_data("back to menu")
        .build();

    row1.push(create_link_bot_item_description);
    row2.push(create_link_bot_item_name);
    row3.push(create_link_custom_name);
    row4.push(back_to_menu);

    keyboard.push(row1);
    keyboard.push(row2);
    keyboard.push(row3);
    keyboard.push(row4);

    let inline_keyboard = InlineKeyboardMarkup::builder()
        .inline_keyboard(keyboard)
        .build();
    SendMessageParams::builder()
        .chat_id(message.chat.id)
        .text(text)
        .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
        .build()
}
