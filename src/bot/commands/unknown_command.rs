use super::Command;
use super::Message;
use crate::bot::telegram_client::Api;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;

static UNKNOWN_COMMAND_PRIVATE: &str = "Unknown command. Use /help to show available commands";
static UNKNOWN_COMMAND_GROUP: &str = "Unknown command. Use /help to show available commands.\n\
  Remove admin access from the bot in this group otherwise it will be replying to every message.";

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
        if message.chat().type_field() == "private" {
            UNKNOWN_COMMAND_PRIVATE.to_string()
        } else {
            UNKNOWN_COMMAND_GROUP.to_string()
        }
    }

    fn execute(&self, db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        info!(
            "{:?} wrote: {}",
            message.chat().id(),
            message.text().unwrap()
        );

        if message.chat().type_field() == "private"
            || message.chat().type_field() == "group"
            || message.chat().type_field() == "supergroup"
        {
            let text = self.response(db_pool, &message);

            self.reply_to_message(api, message, text);
        }
    }

    fn command(&self) -> &str {
        Self::command()
    }
}
