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

pub fn set_template_keyboard(message: &Message) -> SendMessageParams {
    let chat_id: i64 = message.chat.id;
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let mut row: Vec<InlineKeyboardButton> = Vec::new();
    let mut row2: Vec<InlineKeyboardButton> = Vec::new();
    let mut row3: Vec<InlineKeyboardButton> = Vec::new();

    let create_link_item_description = InlineKeyboardButton::builder()
        .text("Make bot item descriptions as the link")
        .switch_inline_query_current_chat(
            "/set_template \"put_feed_url_here\" {{create_link bot_item_description bot_item_link }}",
        )
        .build();
    let create_link_item_name = InlineKeyboardButton::builder()
        .text("Make bot item name as the link")
        .switch_inline_query_current_chat(
            "/set_template \"put_feed_url_here\" {{create_link bot_item_name bot_item_link }}",
        )
        .build();
    let create_link_custom_name = InlineKeyboardButton::builder()
        .text("Make custom name as the link")
        .switch_inline_query_current_chat(
            "/set_template \"put_feed_url_here\" {{create_link \"custom_name\" bot_item_link }}",
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
