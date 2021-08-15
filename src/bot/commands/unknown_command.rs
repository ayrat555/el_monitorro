use super::Command;
use super::Message;
use crate::bot::telegram_client::Api;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;

static UNKNOWN_COMMAND: &str = "Unknown command. Use /help to show available commands";

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
        _message: &Message,
    ) -> String {
        UNKNOWN_COMMAND.to_string()
    }

    fn execute(&self, db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        if message.chat().type_field() == "private" {
            let text = self.response(db_pool, &message);

            self.reply_to_message(api, message, text);
        }
    }

    fn command(&self) -> &str {
        Self::command()
    }
}
