use super::Command;
use super::Message;
use super::Response;
use diesel::PgConnection;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/get_template";

#[derive(TypedBuilder)]
pub struct GetTemplate {
    message: Message,
    args: String,
}

impl GetTemplate {
    pub fn run(&self) {
        self.execute(&self.message);
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
    fn response(&self) -> Response {
        let response = match self.fetch_db_connection() {
            Ok(mut connection) => self.get_template(&mut connection),
            Err(error_message) => error_message,
        };

        Response::Simple(response)
    }
}
