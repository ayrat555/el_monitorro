use super::unknown_command::UnknownCommand;
use super::Command;
use super::Message;
use crate::bot::telegram_client::Api;
use crate::config::Config;
use crate::db::feeds;
use crate::db::telegram;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;

static COMMAND: &str = "/info";

pub struct Info {}

impl Info {
    pub fn execute(db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        Self {}.execute(db_pool, api, message);
    }

    fn info(&self, db_connection: &PgConnection, _message: &Message) -> String {
        let total_feeds = match feeds::count_feeds_with_subscriptions(db_connection) {
            Ok(res) => res,
            Err(err) => {
                log::error!("Failed to fetch total feeds count {:?}", err);
                return "Failed to fetch total feeds count".to_string();
            }
        };

        let total_chats = match telegram::count_chats_with_subscriptions(db_connection) {
            Ok(res) => res,
            Err(err) => {
                log::error!("Failed to fetch total chats count {:?}", err);
                return "Failed to fetch total chats count".to_string();
            }
        };

        format!(
            "the number of feeds is {}\n\
             the number of chats is {} \n",
            total_feeds, total_chats
        )
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for Info {
    fn execute(&self, db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        info!(
            "{:?} wrote: {}",
            message.chat().id(),
            message.text().unwrap()
        );
        match Config::admin_telegram_id() {
            None => UnknownCommand::execute(db_pool, api, message),
            Some(id) => {
                if id == message.chat().id() {
                    let text = self.response(db_pool, &message);

                    self.reply_to_message(api, message, text)
                } else {
                    UnknownCommand::execute(db_pool, api, message)
                }
            }
        }
    }

    fn response(
        &self,
        db_pool: Pool<ConnectionManager<PgConnection>>,
        message: &Message,
    ) -> String {
        match self.fetch_db_connection(db_pool) {
            Ok(connection) => self.info(&connection, message),
            Err(error_message) => error_message,
        }
    }

    fn command(&self) -> &str {
        Self::command()
    }
}
