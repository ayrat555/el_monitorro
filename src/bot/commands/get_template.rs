use crate::bot::telegram_client::Api;

use super::Command;
use super::Message;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/get_template";

#[derive(TypedBuilder)]
pub struct GetTemplate {
    message: Message,
    args: String,
}

impl GetTemplate {
    pub fn run(&self, db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        self.execute(db_pool, api, message);
    }

    fn get_template(&self, db_connection: &mut PgConnection) -> String {
        match self.find_subscription(db_connection, self.message.chat.id, &self.args) {
            Err(message) => message,
            Ok(subscription) => match subscription.template {
                None => "You did not set a template for this subcription".to_string(),
                Some(template) => template,
            },
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for GetTemplate {
    fn response(&self) -> String {
        match self.fetch_db_connection() {
            Ok(mut connection) => self.get_template(&mut connection),
            Err(error_message) => error_message,
        }
    }
}
