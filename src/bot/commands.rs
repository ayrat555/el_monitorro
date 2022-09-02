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
use frankenstein::Message;

pub mod get_filter;
pub mod get_global_filter;
pub mod get_global_template;
pub mod get_template;
pub mod get_timezone;
pub mod help;
pub mod info;
pub mod list_subscriptions;
pub mod remove_filter;
pub mod remove_global_filter;
pub mod remove_global_template;
pub mod remove_template;
pub mod set_content_fields;
pub mod set_filter;
pub mod set_global_filter;
pub mod set_global_template;
pub mod set_template;
pub mod set_timezone;
pub mod start;
pub mod subscribe;
pub mod unknown_command;
pub mod unsubscribe;

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

    fn execute(&self, db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        info!(
            "{:?} wrote: {}",
            message.chat.id,
            message.text.as_ref().unwrap()
        );

        let text = self.response(db_pool, &message, &api);

        self.reply_to_message(api, message, text)
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

        if filter_words.len() > 7 {
            return Err("The number of filter words is limited by 7".to_string());
        }

        Ok(filter_words)
    }
}
