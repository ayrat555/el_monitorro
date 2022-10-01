use super::Command;
use super::Message;
use crate::bot::telegram_client::Api;
use crate::db::telegram;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/remove_template";

#[derive(TypedBuilder)]
pub struct RemoveTemplate {
    db_pool: Pool<ConnectionManager<PgConnection>>,
    api: Api,
    message: Message,
    args: String,
}

impl RemoveTemplate {
    pub fn run(&self) {
        self.execute(&self.api, &self.message);
    }

    fn remove_template(&self, db_connection: &mut PgConnection) -> String {
        let subscription =
            match self.find_subscription(db_connection, self.message.chat.id, self.args) {
                Err(message) => return message,
                Ok(subscription) => subscription,
            };

        match telegram::set_template(db_connection, &subscription, None) {
            Ok(_) => "The template was removed".to_string(),
            Err(_) => "Failed to update the template".to_string(),
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for RemoveTemplate {
    fn response(&self) -> String {
        match self.fetch_db_connection(&self.db_pool) {
            Ok(mut connection) => self.remove_template(&mut connection),
            Err(error_message) => error_message,
        }
    }
}
