use super::Command;
use super::Message;
use super::Response;
use crate::db::telegram;
use diesel::PgConnection;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/get_global_template";

#[derive(TypedBuilder)]
pub struct GetGlobalTemplate {
    message: Message,
}

impl GetGlobalTemplate {
    pub fn run(&self) {
        self.execute(&self.message, Self::command());
    }

    fn get_global_template(&self, db_connection: &mut PgConnection) -> String {
        match telegram::find_chat(db_connection, self.message.chat.id) {
            None => "You don't have the global template set".to_string(),
            Some(chat) => match chat.template {
                None => "You don't have the global template set".to_string(),
                Some(value) => format!("Your global template is \n {value}"),
            },
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for GetGlobalTemplate {
    fn response(&self) -> Response {
        let response = match self.fetch_db_connection() {
            Ok(mut connection) => self.get_global_template(&mut connection),
            Err(error_message) => error_message,
        };

        Response::Simple(response)
    }
}
