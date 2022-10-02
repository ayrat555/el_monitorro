use super::unknown_command::UnknownCommand;
use super::Command;
use super::Message;
use crate::config::Config;
use crate::db::feeds;
use crate::db::telegram;
use diesel::PgConnection;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/info";

#[derive(TypedBuilder)]
pub struct Info {
    message: Message,
}

impl Info {
    pub fn run(&self) {
        self.execute(&self.message);
    }

    fn info(&self, db_connection: &mut PgConnection) -> String {
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

    fn unknown_command(&self) {
        UnknownCommand::builder()
            .message(self.message.clone())
            .args(self.message.text.clone().unwrap())
            .build()
            .run();
    }
}

impl Command for Info {
    fn execute(&self, message: &Message) {
        match Config::admin_telegram_id() {
            None => self.unknown_command(),
            Some(id) => {
                if id == message.chat.id {
                    info!(
                        "{:?} wrote: {}",
                        message.chat.id,
                        message.text.as_ref().unwrap()
                    );

                    let text = self.response();

                    self.reply_to_message(message, text)
                } else {
                    self.unknown_command();
                }
            }
        }
    }

    fn response(&self) -> String {
        match self.fetch_db_connection() {
            Ok(mut connection) => self.info(&mut connection),
            Err(error_message) => error_message,
        }
    }
}
