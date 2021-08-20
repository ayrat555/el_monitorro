use super::Command;
use super::Message;
use crate::bot::telegram_client::Api;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;

static UNKNOWN_COMMAND_GROUP: &str = "Remove admin access from the bot in this group otherwise it will be replying to every message.";
static UNKNOWN_COMMAND_PRIVATE: &str = "Unknown command. Use /help to show available commands";

static COMMAND: &str = "";

pub struct UnknownCommand {}

impl UnknownCommand {
    pub fn execute(db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        Self {}.execute(db_pool, api, message);
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for UnknownCommand {
    fn response(
        &self,
        _db_pool: Pool<ConnectionManager<PgConnection>>,
        message: &Message,
    ) -> String {
        match message.chat().type_field().as_str() {
            "private" => UNKNOWN_COMMAND_PRIVATE.to_string(),
            "group" | "supergroup" => {
                if message.text().unwrap().starts_with('/') {
                    "".to_string()
                } else {
                    UNKNOWN_COMMAND_GROUP.to_string()
                }
            }
            &_ => "".to_string(),
        }
    }

    fn execute(&self, db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        if message.chat().type_field() != "channel" {
            info!(
                "{:?} wrote: {}",
                message.chat().id(),
                message.text().unwrap()
            );
        }

        let text = self.response(db_pool, &message);

        if !text.is_empty() {
            self.reply_to_message(api, message, text);
        }
    }

    fn command(&self) -> &str {
        Self::command()
    }
}
