use crate::bot::telegram_client::Api;
use crate::db::feeds;
use crate::db::telegram;
use crate::db::telegram::NewTelegramChat;
use crate::db::telegram::NewTelegramSubscription;
use crate::models::telegram_subscription::TelegramSubscription;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::r2d2::PooledConnection;
use diesel::PgConnection;
use frankenstein::Chat;
use frankenstein::ChatId;
use frankenstein::Message;
use frankenstein::SendMessageParams;
use frankenstein::TelegramApi;
use handlebars::{to_json, Handlebars};
use regex::Regex;
use serde_json::value::Map;
use std::env;

pub mod get_filter;
pub mod get_template;
pub mod get_timezone;
pub mod help;
pub mod list_subscriptions;
pub mod set_filter;
pub mod set_template;
pub mod set_timezone;
pub mod start;
pub mod subscribe;
pub mod unknown_command;
pub mod unsubscribe;

impl From<Chat> for NewTelegramChat {
    fn from(chat: Chat) -> Self {
        NewTelegramChat {
            id: chat.id() as i64,
            kind: chat.type_field(),
            username: chat.username(),
            first_name: chat.first_name(),
            last_name: chat.last_name(),
            title: chat.title(),
        }
    }
}

pub trait Command {
    fn response(&self, db_pool: Pool<ConnectionManager<PgConnection>>, message: &Message)
        -> String;

    fn execute(&self, db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        info!(
            "{:?} wrote: {}",
            message.chat().id(),
            message.text().unwrap()
        );

        let text = self.response(db_pool, &message);

        self.reply_to_message(api, message, text)
    }

    fn reply_to_message(&self, api: Api, message: Message, text: String) {
        let mut send_message_params =
            SendMessageParams::new(ChatId::Integer(message.chat().id()), text);

        send_message_params.set_reply_to_message_id(Some(message.message_id()));

        if let Err(err) = api.send_message(&send_message_params) {
            error!(
                "Failed to send a message {:?}: {:?}",
                err, send_message_params
            );
        }
    }

    fn command(&self) -> &str;

    fn parse_argument(&self, full_command: &str) -> String {
        let command = self.command();
        let handle = env::var("TELEGRAM_BOT_HANDLE").unwrap_or_else(|_| "".to_string());
        let command_with_handle = format!("{}@{}", command, handle);

        if full_command.starts_with(&command_with_handle) {
            full_command
                .replace(&command_with_handle, "")
                .trim()
                .to_string()
        } else {
            full_command.replace(command, "").trim().to_string()
        }
    }

    fn fetch_db_connection(
        &self,
        db_pool: Pool<ConnectionManager<PgConnection>>,
    ) -> Result<PooledConnection<ConnectionManager<PgConnection>>, String> {
        match db_pool.get() {
            Ok(connection) => Ok(connection),
            Err(err) => {
                error!("Failed to fetch a connection from the pool {:?}", err);

                Err("Failed to process your command. Please contact @Ayrat555".to_string())
            }
        }
    }

    fn find_subscription(
        &self,
        db_connection: &PgConnection,
        chat_id: i64,
        feed_url: String,
    ) -> Result<TelegramSubscription, String> {
        let not_exists_error = Err("Subscription does not exist".to_string());
        let feed = match feeds::find_by_link(db_connection, feed_url) {
            Some(feed) => feed,
            None => return not_exists_error,
        };

        let chat = match telegram::find_chat(db_connection, chat_id) {
            Some(chat) => chat,
            None => return not_exists_error,
        };

        let telegram_subscription = NewTelegramSubscription {
            chat_id: chat.id,
            feed_id: feed.id,
        };

        match telegram::find_subscription(db_connection, telegram_subscription) {
            Some(subscription) => Ok(subscription),
            None => not_exists_error,
        }
    }
}

pub trait Template {
    fn parse_template(&self, template: &str) -> String {
        let allowed_fields = vec![
            "bot_feed_name",
            "bot_item_name",
            "bot_date",
            "bot_feed_link",
            "bot_item_link",
            "bot_item_description",
        ];
        let separators = vec!["bot_new_line", "bot_space"];
        let all_words = [&allowed_fields[..], &separators[..]].concat();
        let regex_string = all_words.join("|");
        let regex = Regex::new(&regex_string).unwrap();

        let mut result = "".to_string();

        for part in self.split_keep(&regex, template) {
            if allowed_fields.iter().any(|&i| i == part) {
                let new_part = format!("{{{{{}}}}}", part);
                result.push_str(&new_part);
            } else if part == "bot_space" {
                result.push(' ');
            } else if part == "bot_new_line" {
                result.push('\n');
            } else {
                result.push_str(part);
            }
        }

        result
    }

    fn split_keep<'a>(&self, r: &Regex, text: &'a str) -> Vec<&'a str> {
        let mut result = Vec::new();
        let mut last = 0;
        for (index, matched) in text.match_indices(r) {
            if last != index {
                result.push(&text[last..index]);
            }
            result.push(matched);
            last = index + matched.len();
        }
        if last < text.len() {
            result.push(&text[last..]);
        }
        result
    }

    fn parse_template_and_send_example(
        &self,
        raw_template: String,
    ) -> Result<(String, String), String> {
        let mut data = Map::new();
        data.insert("bot_feed_name".to_string(), to_json("feed_name"));
        data.insert("bot_item_name".to_string(), to_json("item_name"));
        data.insert("bot_date".to_string(), to_json("date"));
        data.insert("bot_feed_link".to_string(), to_json("feed_link"));
        data.insert("bot_item_link".to_string(), to_json("item_link"));
        data.insert(
            "bot_item_description".to_string(),
            to_json("item_description"),
        );

        let reg = Handlebars::new();
        let template = self.parse_template(&raw_template);

        match reg.render_template(&template, &data) {
            Err(_) => Err("Failed to update the template".to_string()),
            Ok(result) => Ok((template, result)),
        }
    }
}
