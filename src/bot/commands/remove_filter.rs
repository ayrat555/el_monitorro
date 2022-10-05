use super::Command;
use super::Message;
use crate::bot::telegram_client::Api;
use crate::db::telegram;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/remove_filter";

#[derive(TypedBuilder)]
pub struct RemoveFilter {
    message: Message,
    args: String,
}

impl RemoveFilter {
    pub fn run(&self, db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        self.execute(db_pool, api, message);
    }

    pub fn remove_filter(&self, db_connection: &mut PgConnection) -> String {
        let subscription =
            match self.find_subscription(db_connection, self.message.chat.id, &self.args) {
                Err(message) => return message,
                Ok(subscription) => subscription,
            };

        match telegram::set_filter(db_connection, &subscription, None) {
            Ok(_) => "The filter was removed".to_string(),
            Err(_) => "Failed to update the filter".to_string(),
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for RemoveFilter {
    fn response(&self) -> String {
        match self.fetch_db_connection() {
            Ok(mut connection) => self.remove_filter(&mut connection),
            Err(error_message) => error_message,
        }
    }
}
