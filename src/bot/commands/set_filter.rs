use super::Command;
use super::Message;
use crate::bot::telegram_client::Api;
use crate::db::telegram;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;

static COMMAND: &str = "/set_filter";

pub struct SetFilter {}

impl SetFilter {
    pub fn execute(db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        Self {}.execute(db_pool, api, message);
    }

    pub fn set_filter(
        &self,
        db_connection: &PgConnection,
        message: &Message,
        params: String,
    ) -> String {
        let vec: Vec<&str> = params.split(' ').collect();

        if vec.len() != 2 {
            return "Wrong number of parameters".to_string();
        }

        if vec[1].is_empty() {
            return "Filter can not be empty".to_string();
        }

        let subscription =
            match self.find_subscription(db_connection, message.chat.id, vec[0].to_string()) {
                Err(message) => return message,
                Ok(subscription) => subscription,
            };

        let filter_words: Vec<String> =
            vec[1].split(',').map(|s| s.trim().to_lowercase()).collect();

        if filter_words.len() > 7 {
            return "The number of filter words is limited by 7".to_string();
        }

        match telegram::set_filter(db_connection, &subscription, Some(filter_words.clone())) {
            Ok(_) => format!("The filter was updated:\n\n{}", filter_words.join(", ")),
            Err(_) => "Failed to update the filter".to_string(),
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for SetFilter {
    fn response(
        &self,
        db_pool: Pool<ConnectionManager<PgConnection>>,
        message: &Message,
    ) -> String {
        match self.fetch_db_connection(db_pool) {
            Ok(connection) => {
                let text = message.text.as_ref().unwrap();
                let argument = self.parse_argument(&text);
                self.set_filter(&connection, message, argument)
            }
            Err(error_message) => error_message,
        }
    }

    fn command(&self) -> &str {
        Self::command()
    }
}
