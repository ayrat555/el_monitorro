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
use std::env;

pub mod get_filter;
pub mod get_timezone;
pub mod help;
pub mod list_subscriptions;
pub mod set_filter;
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
