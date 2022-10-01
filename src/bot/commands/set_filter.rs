use super::Command;
use super::Message;
use crate::bot::telegram_client::Api;
use crate::db::telegram;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/set_filter";

#[derive(TypedBuilder)]
pub struct SetFilter {
    db_pool: Pool<ConnectionManager<PgConnection>>,
    api: Api,
    message: Message,
    args: String,
}

impl SetFilter {
    pub fn run(&self) {
        self.execute(&self.api, &self.message);
    }

    pub fn set_filter(&self, db_connection: &mut PgConnection) -> String {
        let vec: Vec<&str> = self.args.splitn(2, ' ').collect();

        if vec.len() != 2 {
            return "Wrong number of parameters".to_string();
        }

        if vec[1].is_empty() {
            return "Filter can not be empty".to_string();
        }

        let filter_words = match self.parse_filter(vec[1]) {
            Err(message) => return message,
            Ok(words) => words,
        };

        let subscription =
            match self.find_subscription(db_connection, self.message.chat.id, vec[0].to_string()) {
                Err(message) => return message,
                Ok(subscription) => subscription,
            };

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
    fn response(&self) -> String {
        match self.fetch_db_connection(&self.db_pool) {
            Ok(mut connection) => self.set_filter(&mut connection),
            Err(error_message) => error_message,
        }
    }
}
