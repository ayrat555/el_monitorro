use super::Command;
use super::Message;
use crate::bot::telegram_client::Api;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/get_filter";

#[derive(TypedBuilder)]
pub struct GetFilter {
    db_pool: Pool<ConnectionManager<PgConnection>>,
    api: Api,
    message: Message,
    args: String,
}

impl GetFilter {
    pub fn run(&self) {
        self.execute(&self.api, &self.message);
    }

    fn get_filter(&self, db_connection: &mut PgConnection) -> String {
        match self.find_subscription(db_connection, self.message.chat.id, self.args) {
            Err(message) => message,
            Ok(subscription) => match subscription.filter_words {
                None => "You did not set a filter for this subcription".to_string(),
                Some(filter_words) => filter_words.join(", "),
            },
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for GetFilter {
    fn response(&self) -> String {
        match self.fetch_db_connection(&self.db_pool) {
            Ok(mut connection) => self.get_filter(&mut connection),

            Err(error_message) => error_message,
        }
    }
}
