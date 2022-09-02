use super::Command;
use super::Message;
use crate::bot::telegram_client::Api;
use crate::db::telegram;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;

static COMMAND: &str = "/remove_filter";

pub struct RemoveFilter {}

impl RemoveFilter {
    pub fn execute(db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        Self {}.execute(db_pool, api, message);
    }

    pub fn remove_filter(
        &self,
        db_connection: &PgConnection,
        message: &Message,
        feed_url: String,
    ) -> String {
        let subscription =
            match self.find_subscription(&mut db_connection, message.chat.id, feed_url) {
                Err(message) => return message,
                Ok(subscription) => subscription,
            };

        match telegram::set_filter(&mut db_connection, &subscription, None) {
            Ok(_) => "The filter was removed".to_string(),
            Err(_) => "Failed to update the filter".to_string(),
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for RemoveFilter {
    fn response(
        &self,
        db_pool: Pool<ConnectionManager<PgConnection>>,
        message: &Message,
        _api: &Api,
    ) -> String {
        match self.fetch_db_connection(db_pool) {
            Ok(connection) => {
                let text = message.text.as_ref().unwrap();
                let argument = self.parse_argument(text);
                self.remove_filter(&connection, message, argument)
            }
            Err(error_message) => error_message,
        }
    }

    fn command(&self) -> &str {
        Self::command()
    }
}
