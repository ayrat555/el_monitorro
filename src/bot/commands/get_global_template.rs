use super::Command;
use super::Message;
use crate::bot::telegram_client::Api;
use crate::db::telegram;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;

static COMMAND: &str = "/get_global_template";

pub struct GetGlobalTemplate {}

impl GetGlobalTemplate {
    pub fn execute(db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        Self {}.execute(db_pool, api, message);
    }

    fn get_global_template(&self, db_connection: &mut PgConnection, message: &Message) -> String {
        match telegram::find_chat(db_connection, message.chat.id) {
            None => "You don't have the global template set".to_string(),
            Some(chat) => match chat.template {
                None => "You don't have the global template set".to_string(),
                Some(value) => format!("Your global template is \n {}", value),
            },
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for GetGlobalTemplate {
    fn response(
        &self,
        db_pool: Pool<ConnectionManager<PgConnection>>,
        message: &Message,
        _api: &Api,
    ) -> String {
        match self.fetch_db_connection(db_pool) {
            Ok(mut connection) => self.get_global_template(&mut connection, message),
            Err(error_message) => error_message,
        }
    }

    fn command(&self) -> &str {
        Self::command()
    }
}
