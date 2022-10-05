use super::Command;
use super::Message;
use crate::bot::telegram_client::Api;
use crate::db::telegram;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/get_global_filter";

#[derive(TypedBuilder)]
pub struct GetGlobalFilter {
    message: Message,
}

impl GetGlobalFilter {
    pub fn run(&self, db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        self.execute(db_pool, api, message);
    }

    fn get_global_template(&self, db_connection: &mut PgConnection) -> String {
        match telegram::find_chat(db_connection, self.message.chat.id) {
            None => "You don't have the global filter set".to_string(),
            Some(chat) => match chat.filter_words {
                None => "You don't have the global filter set".to_string(),
                Some(filter_words) => {
                    format!("Your global filter is \n {}", filter_words.join(", "))
                }
            },
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for GetGlobalFilter {
    fn response(&self) -> String {
        match self.fetch_db_connection() {
            Ok(mut connection) => self.get_global_template(&mut connection),
            Err(error_message) => error_message,
        }
    }
}
