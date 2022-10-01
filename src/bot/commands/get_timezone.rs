use super::Command;
use super::Message;
use crate::bot::telegram_client::Api;
use crate::db::telegram;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/get_timezone";

#[derive(TypedBuilder)]
pub struct GetTimezone {
    db_pool: Pool<ConnectionManager<PgConnection>>,
    api: Api,
    message: Message,
}

impl GetTimezone {
    pub fn run(&self) {
        self.execute(&self.api, &self.message);
    }

    fn get_timezone(&self, db_connection: &mut PgConnection) -> String {
        match telegram::find_chat(db_connection, self.message.chat.id) {
            None => "You don't have timezone set".to_string(),
            Some(chat) => match chat.utc_offset_minutes {
                None => "You don't have timezone set".to_string(),
                Some(value) => format!("Your timezone offset is {} minutes", value),
            },
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for GetTimezone {
    fn response(&self) -> String {
        match self.fetch_db_connection(self.db_pool) {
            Ok(mut connection) => self.get_timezone(&mut connection),
            Err(error_message) => error_message,
        }
    }
}
