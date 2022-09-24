use crate::bot::commands::list_subscriptions_inline_keyboard::select_feed_url_keyboard_list_subscriptions;
use crate::bot::commands::set_global_template_inline_keyboard::set_global_template_keyboard;
use crate::bot::commands::set_template_inline_keyboard::select_feed_url_keyboard;
use crate::bot::commands::subscribe_inline_keyboard::set_subscribe_keyboard;
use crate::bot::commands::unsubscribe_inline_keyboard::set_unsubscribe_keyboard;
use crate::bot::telegram_client::Api;
use crate::config::Config;
use crate::db::feeds;
use crate::db::telegram;
use crate::db::telegram::NewTelegramChat;
use crate::db::telegram::NewTelegramSubscription;
use crate::models::Feed;
use crate::models::TelegramSubscription;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::r2d2::PooledConnection;
use diesel::PgConnection;
use frankenstein::Chat;
use frankenstein::ChatType;
use frankenstein::DeleteMessageParams;
use frankenstein::Message;
use frankenstein::TelegramApi;

pub mod get_filter;
pub mod get_global_filter;
pub mod get_global_template;
pub mod get_template;
pub mod get_timezone;
pub mod help;
pub mod info;
pub mod list_subscriptions;
pub mod list_subscriptions_inline_keyboard;
pub mod remove_filter;
pub mod remove_global_filter;
pub mod remove_global_template;
pub mod remove_template;
pub mod set_content_fields;
pub mod set_filter;
pub mod set_global_filter;
pub mod set_global_template;
pub mod set_global_template_inline_keyboard;
pub mod set_template;
pub mod set_template_inline_keyboard;
pub mod set_timezone;
pub mod start;
pub mod subscribe;
pub mod subscribe_inline_keyboard;
pub mod unknown_command;
pub mod unsubscribe;
pub mod unsubscribe_inline_keyboard;

const BOT_NAME: &str = "@el_monitorro_bot "; //replace with your bot name and add a space at end

impl From<Chat> for NewTelegramChat {
    fn from(chat: Chat) -> Self {
        let kind = match chat.type_field {
            ChatType::Private => "private",
            ChatType::Group => "group",
            ChatType::Supergroup => "supergroup",
            ChatType::Channel => "channel",
        };

        NewTelegramChat {
            id: chat.id as i64,
            kind: kind.to_string(),
            username: chat.username,
            first_name: chat.first_name,
            last_name: chat.last_name,
            title: chat.title,
        }
    }
}

pub trait Command {
    fn response(
        &self,
        db_pool: Pool<ConnectionManager<PgConnection>>,
        message: &Message,
        api: &Api,
    ) -> String;

    fn execute(
        &self,
        db_pool: Pool<ConnectionManager<PgConnection>>,
        api: Api,
        mut message: Message,
    ) {
        let command = message.text.as_ref().unwrap().to_string();
        let response_message = command.replace(BOT_NAME, "");

        info!("{:?} wrote: {}", message.chat.id, response_message,);

        let data = match self.fetch_db_connection(db_pool.clone()) {
            Ok(mut connection) => self.list_subscriptions(&mut *connection, message.clone()),
            Err(_error_message) => "error fetching data".to_string(),
        };

        let feed_id = match self.fetch_db_connection(db_pool.clone()) {
            Ok(mut connection) => self.list_feed_id(&mut *connection, &message),
            Err(_error_message) => "error fetching data".to_string(),
        };

        let feeds = data.split("`'\n'`");
        let feeds_ids = feed_id.split("`','`").clone();
        let text = self.response(db_pool.clone(), &message, &api);
        let delete_message_params = DeleteMessageParams::builder()
            .chat_id(message.chat.id)
            .message_id(message.message_id)
            .build();

        if command == "/subscribe" {
            let send_message_params = set_subscribe_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else if command == "/unsubscribe" {
            let send_message_params = set_unsubscribe_keyboard(message, feeds, feed_id);
            api.send_message(&send_message_params).unwrap();
        } else if command == "/set_global_template" {
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params = set_global_template_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else if command == "/list_subscriptions" {
            if data == "You don't have any subscriptions" {
                self.reply_to_message(api, message, text);
            } else {
                let send_message_params =
                    select_feed_url_keyboard_list_subscriptions(message, feeds, feeds_ids, db_pool);
                api.send_message(&send_message_params).unwrap();
            }
        } else if command == "/set_template" {
            if data == "You don't have any subscriptions" {
                message.text = Some("/list_subscriptions".to_string());
                self.reply_to_message(api, message, data);
            } else {
                let send_message_params =
                    select_feed_url_keyboard(message, feeds, feeds_ids, db_pool);
                api.send_message(&send_message_params).unwrap();
            }
        } else {
            self.reply_to_message(api, message, text);
        }
    }

    fn reply_to_message(&self, api: Api, message: Message, text: String) {
        if let Err(error) =
            api.reply_with_text_message(message.chat.id, text, Some(message.message_id))
        {
            error!("Failed to reply to update {:?} {:?}", error, message);
        }
    }
    fn command(&self) -> &str;

    fn parse_argument(&self, full_command: &str) -> String {
        let command = self.command();
        let handle = Config::telegram_bot_handle();
        let command_with_handle = format!("{}@{}", command, handle);

        if full_command.starts_with(&command_with_handle) {
            full_command
                .replace(&command_with_handle, "")
                .replace(BOT_NAME, "")
                .trim()
                .to_string()
        } else {
            full_command
                .replace(command, "")
                .replace(BOT_NAME, "")
                .trim()
                .to_string()
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
        db_connection: &mut PgConnection,
        chat_id: i64,
        feed_url: String,
    ) -> Result<TelegramSubscription, String> {
        let not_exists_error = Err("Subscription does not exist".to_string());
        let feed = self.find_feed(db_connection, feed_url)?;

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

    fn find_feed(
        &self,
        db_connection: &mut PgConnection,
        feed_url: String,
    ) -> Result<Feed, String> {
        match feeds::find_by_link(db_connection, feed_url) {
            Some(feed) => Ok(feed),
            None => Err("Feed does not exist".to_string()),
        }
    }

    fn parse_filter(&self, params: &str) -> Result<Vec<String>, String> {
        let filter_words: Vec<String> =
            params.split(',').map(|s| s.trim().to_lowercase()).collect();

        if filter_words.len() > 20 {
            return Err("The number of filter words is limited by 20".to_string());
        }

        Ok(filter_words)
    }
    fn list_subscriptions(&self, db_connection: &mut PgConnection, message: Message) -> String {
        match telegram::find_feeds_by_chat_id(db_connection, message.chat.id) {
            Err(_) => "Couldn't fetch your subscriptions".to_string(),
            Ok(feeds) => {
                if feeds.is_empty() {
                    "You don't have any subscriptions".to_string()
                } else {
                    feeds
                        .into_iter()
                        .map(|feed| feed.link)
                        .collect::<Vec<String>>()
                        .join("\n")
                }
            }
        }
    }
    fn list_feed_id(&self, db_connection: &mut PgConnection, message: &Message) -> String {
        match telegram::find_feeds_by_chat_id(db_connection, message.chat.id) {
            Err(_) => "Couldn't fetch your subscriptions".to_string(),
            Ok(feeds) => {
                if feeds.is_empty() {
                    "You don't have any subscriptions".to_string()
                } else {
                    feeds
                        .into_iter()
                        .map(|feed| feed.id.to_string())
                        .collect::<Vec<String>>()
                        .join(",")
                }
            }
        }
    }
}
