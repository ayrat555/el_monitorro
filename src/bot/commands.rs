use crate::bot::telegram_client::Api;
use crate::bot::telegram_client::Error;
use crate::db::telegram::NewTelegramChat;
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

pub mod help;
pub mod subscribe;
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
}
