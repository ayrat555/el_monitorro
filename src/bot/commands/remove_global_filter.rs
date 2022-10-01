use super::Command;
use super::Message;
use crate::bot::telegram_client::Api;
use crate::db::telegram;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/remove_global_filter";

#[derive(TypedBuilder)]
pub struct RemoveGlobalFilter {
    db_pool: Pool<ConnectionManager<PgConnection>>,
    api: Api,
    message: Message,
}

impl RemoveGlobalFilter {
    pub fn run(&self) {
        self.execute(&self.api, &self.message);
    }

    fn remove_global_filter(&self, db_connection: &mut PgConnection) -> String {
        let chat = match telegram::find_chat(db_connection, self.message.chat.id) {
            Some(chat) => chat,
            None => return "You don't have any subcriptions".to_string(),
        };

        match telegram::set_global_filter(db_connection, &chat, None) {
            Ok(_) => "The global filter was removed".to_string(),
            Err(_) => "Failed to update the filter".to_string(),
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for RemoveGlobalFilter {
    fn response(&self) -> String {
        match self.fetch_db_connection(&self.db_pool) {
            Ok(mut connection) => self.remove_global_filter(&mut connection),
            Err(error_message) => error_message,
        }
    }
}
