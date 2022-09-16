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

static COMMAND: &str = "/set_template";

pub struct SetTemplate {}

impl SetTemplate {
    pub fn execute(db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        Self {}.execute(db_pool, api, message);
    }

    fn set_template(
        &self,
        api: &Api,
        db_connection: &mut PgConnection,
        message: &Message,
        params: String,
    ) -> String {
        let vec: Vec<&str> = params.splitn(2, ' ').collect();

        if vec.len() != 2 {
            return "Wrong number of parameters".to_string();
        }

        if vec[1].is_empty() {
            return "Template can not be empty".to_string();
        }

        let feed_url = vec[0].to_string();
        let template = vec[1];

        let subscription = match self.find_subscription(db_connection, message.chat.id, feed_url) {
            Err(message) => return message,
            Ok(subscription) => subscription,
        };

        let example = match render_template_example(template) {
            Ok(example) => format!("Your messages will look like:\n\n{}", example),
            Err(_) => return "The template is invalid".to_string(),
        };

        if api.send_text_message(message.chat.id, example).is_err() {
            return "The template is invalid".to_string();
        }

        match telegram::set_template(db_connection, &subscription, Some(template.to_string())) {
            Ok(_) => "The template was updated".to_string(),
            Err(_) => "Failed to update the template".to_string(),
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for SetTemplate {
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
                self.set_template(api, &mut connection, message, argument)
            }
            Err(error_message) => error_message,
        }
    }

    fn command(&self) -> &str {
        Self::command()
    }
}

pub fn set_template_keyboard(message: Message, command: String) -> SendMessageParams {
    let chat_id: i64 = message.chat.id;
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();
    let data = command.replace("feed1", "");
    println!("list subscriptions feed url data ==={}", data);
    let mut row: Vec<InlineKeyboardButton> = Vec::new();
    let mut row1: Vec<InlineKeyboardButton> = Vec::new();
    let mut row2: Vec<InlineKeyboardButton> = Vec::new();
    let mut row3: Vec<InlineKeyboardButton> = Vec::new();

    let create_link_item_description = InlineKeyboardButton::builder()
        .text("Make bot item descriptions as the link")
        .callback_data(format!("/set_template_des {}", data))
        .build();
    let create_link_item_name = InlineKeyboardButton::builder()
        .text("Make bot item name as the link")
        .callback_data(format!("/set_template_itm {}", data))
        .build();
    let create_link_custom_name = InlineKeyboardButton::builder()
        .text("Make custom name as the link")
        .callback_data(format!("/set_template_cst {}", data))
        .build();

    row1.push(create_link_item_description);
    row2.push(create_link_item_name);
    row3.push(create_link_custom_name);

    keyboard.push(row);
    keyboard.push(row1);
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
pub fn set_template_menu_keyboard(message: Message, command: String) -> SendMessageParams {
    let text = message.text.unwrap();
    let data = command.replace("feed_for_template", "");
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let mut row: Vec<InlineKeyboardButton> = Vec::new();
    let mut row2: Vec<InlineKeyboardButton> = Vec::new();
    let mut row3: Vec<InlineKeyboardButton> = Vec::new();
    let mut row4: Vec<InlineKeyboardButton> = Vec::new();

    let substring = InlineKeyboardButton::builder()
        .text("Limit number of characters of feed message")
        .callback_data(format!("substring {}", data))
        .build();
    let bold = InlineKeyboardButton::builder()
        .text("Set your feed message bold ")
        .callback_data(format!("bold_set_template {}", data))
        .build();
    let italic = InlineKeyboardButton::builder()
        .text("Set your feed message italic")
        .callback_data(format!("italic_set_template {}", data))
        .build();
    let create_link = InlineKeyboardButton::builder()
        .text("Create link to feed site")
        .callback_data(format!("create_link {}", data))
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
pub fn select_feed_url(message: Message, data: String) -> SendMessageParams {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let mut row: Vec<InlineKeyboardButton> = Vec::new();

    let subscriptions_list = InlineKeyboardButton::builder()
        .text(data.clone())
        .callback_data(format!("feed_for_template {}", data))
        .build();

    row.push(subscriptions_list);

    keyboard.push(row);

    let inline_keyboard = InlineKeyboardMarkup::builder()
        .inline_keyboard(keyboard)
        .build();

    SendMessageParams::builder()
        .chat_id(message.chat.id)
        .text("select feed url to be modifies")
        .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
        .build()
}
pub fn set_template_bold_keyboard(message: Message, command: String) -> SendMessageParams {
    let text = message.text.unwrap();
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let mut row1: Vec<InlineKeyboardButton> = Vec::new();
    let mut row2: Vec<InlineKeyboardButton> = Vec::new();
    let mut row3: Vec<InlineKeyboardButton> = Vec::new();

    let bold_bot_description = InlineKeyboardButton::builder()
        .text("Make bot item description 𝐛𝐨𝐥𝐝")
        .callback_data(format!("/set_template_bold_des {}", command))
        .build();
    let bold_bot_item_name = InlineKeyboardButton::builder()
        .text("Make bot item name 𝐛𝐨𝐥𝐝")
        .callback_data(format!("/set_template_bold_item {}", command))
        .build();
    let back_to_menu = InlineKeyboardButton::builder()
        .text("Back to menu 🔙 ")
        .callback_data("back to set_template menu")
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
pub fn set_template_italic_keyboard(message: Message, command: String) -> SendMessageParams {
    let text = message.text.unwrap();
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let mut row1: Vec<InlineKeyboardButton> = Vec::new();
    let mut row2: Vec<InlineKeyboardButton> = Vec::new();
    let mut row3: Vec<InlineKeyboardButton> = Vec::new();

    let italic_bot_description = InlineKeyboardButton::builder()
        .text("Make bot item description 𝘪𝘵𝘢𝘭𝘪𝘤")
        .callback_data(format!("/set_template_italic_des {}", command))
        .build();
    let italic_bot_item_name = InlineKeyboardButton::builder()
        .text("Make bot item name 𝘪𝘵𝘢𝘭𝘪𝘤")
        .callback_data(format!("/set_template_italic_item {}", command))
        .build();
    let back_to_menu = InlineKeyboardButton::builder()
        .text("Back to menu 🔙 ")
        .callback_data("back to set_template menu")
        .build();

    row1.push(italic_bot_description);
    row2.push(italic_bot_item_name);
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
