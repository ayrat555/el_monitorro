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

    fn info(&self, db_connection: &mut PgConnection, _message: &Message) -> String {
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

        let mut result_message = format!(
            "the number of feeds is {}\n\
             the number of chats is {} \n",
            total_feeds, total_chats
        );

        for kind in ["private", "group", "supergroup", "channel"] {
            let result = match telegram::count_chats_of_type(db_connection, kind) {
                Ok(res) => res,
                Err(err) => {
                    log::error!("Failed to fetch {} chats count {:?}", kind, err);
                    return "Failed to fetch chats count".to_string();
                }
            };

            result_message = format!("{}\n{} chats - {}", result_message, kind, result);
        }

        result_message
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for Info {
    fn execute(&self, db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        match Config::admin_telegram_id() {
            None => UnknownCommand::execute(db_pool, api, message),
            Some(id) => {
                if id == message.chat.id {
                    info!(
                        "{:?} wrote: {}",
                        message.chat.id,
                        message.text.as_ref().unwrap()
                    );

                    let text = self.response(db_pool, &message, &api);

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
        _api: &Api,
    ) -> String {
        match self.fetch_db_connection(db_pool) {
            Ok(mut connection) => self.info(&mut connection, message),
            Err(error_message) => error_message,
        }
    }

    fn command(&self) -> &str {
        Self::command()
    }
}
